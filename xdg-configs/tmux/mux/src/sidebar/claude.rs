use std::collections::HashMap;
use std::time::Duration;

use tracing::error;

use crate::tmux::tmux;

// ── Agent context % via pane statusline scraping ──────────────
// Claude Code's statusline is rendered right in the pane. We capture
// the last lines via `tmux capture-pane` and parse the segmented digit
// + ٪ (U+066A ARABIC PERCENT SIGN) pattern emitted by statusline.py.
// The FIRST ٪ in the statusline is the context percentage.

#[derive(Clone, Default)]
pub(super) struct AgentCtx {
    pub(super) pct: u8,
    pub(super) tokens: String,
}

#[derive(Clone, Default)]
pub(super) struct AgentScrape {
    pub(super) ctx: Option<AgentCtx>,
    /// The gerund verb from the activity line (e.g. "Churning" from "✻ Churning…").
    /// Present only while the agent is actively working.
    pub(super) gerund: Option<String>,
    /// The agent is waiting for user input (question, confirmation, etc.).
    pub(super) asking: bool,
}

/// Returns (scrape_map, tmux_call_count).
/// Each target is (session, pane_id, agent_name); map key is (session, pane_id).
pub(super) fn query_agent_scrapes(
    targets: &[(String, String, String)],
) -> (HashMap<(String, String), AgentScrape>, u32) {
    if targets.is_empty() {
        return (HashMap::new(), 0);
    }

    // Batch all capture-pane calls into a single tmux invocation.
    // We interleave `display-message -p "<<DELIM>>"` between captures
    // so we can split the combined output reliably.
    const DELIM: &str = "\x1e<<MUX_CAPTURE_DELIM>>\x1e";
    let mut args: Vec<String> = Vec::new();
    for (i, (_, pane_id, _)) in targets.iter().enumerate() {
        if i > 0 {
            args.extend([
                ";".into(),
                "display-message".into(),
                "-p".into(),
                DELIM.into(),
                ";".into(),
            ]);
        }
        args.extend([
            "capture-pane".into(),
            "-t".into(),
            pane_id.clone(),
            "-p".into(),
            "-S".into(),
            "-30".into(),
        ]);
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let raw = tmux(&refs);

    let chunks: Vec<&str> = raw.split(DELIM).collect();
    if chunks.len() != targets.len() {
        // Delimiter collision: fall back to sequential per-pane captures.
        error!(
            expected = targets.len(),
            got = chunks.len(),
            "capture-pane delimiter collision, falling back to sequential"
        );
        return scrape_targets_sequential(targets);
    }

    let mut result = HashMap::new();
    for (i, (session, pane_id, agent_name)) in targets.iter().enumerate() {
        let scrape = parse_agent_scrape(agent_name, chunks[i]);
        result.insert((session.clone(), pane_id.clone()), scrape);
    }
    (result, 1)
}

fn scrape_targets_sequential(
    targets: &[(String, String, String)],
) -> (HashMap<(String, String), AgentScrape>, u32) {
    let mut result = HashMap::new();
    for (session, pane_id, agent_name) in targets {
        let text = tmux(&["capture-pane", "-t", pane_id, "-p", "-S", "-30"]);
        let scrape = parse_agent_scrape(agent_name, &text);
        result.insert((session.clone(), pane_id.clone()), scrape);
    }
    (result, targets.len() as u32)
}

fn parse_agent_scrape(agent_name: &str, text: &str) -> AgentScrape {
    match agent_name {
        "claude" => {
            let ctx = parse_context(text);
            AgentScrape {
                asking: ctx.is_none() && is_claude_asking(text),
                gerund: parse_gerund(text),
                ctx,
            }
        }
        "codex" => {
            let ctx = parse_codex_ctx(text);
            AgentScrape {
                asking: ctx.is_none() && is_codex_asking(text),
                gerund: parse_codex_activity(text),
                ctx,
            }
        }
        "opencode" => {
            let ctx = parse_opencode_ctx(text);
            AgentScrape {
                asking: ctx.is_none() && is_opencode_asking(text),
                gerund: parse_opencode_activity(text),
                ctx,
            }
        }
        _ => AgentScrape::default(),
    }
}

/// Claude: "Enter to select" visible + no context statusline.
fn is_claude_asking(text: &str) -> bool {
    text.lines()
        .any(|l| l.contains("Enter to select") || l.contains("enter to select"))
}

/// Codex: "to submit answer" visible + no context statusline.
fn is_codex_asking(text: &str) -> bool {
    text.lines().any(|l| l.contains("to submit answer"))
}

/// OpenCode: "select", "submit", and "dismiss" all present + no context.
fn is_opencode_asking(text: &str) -> bool {
    let combined: String = text.lines().collect();
    combined.contains("select")
        && combined.contains("submit")
        && combined.contains("dismiss")
}

/// Extract the gerund verb from an activity line like "✻ Churning…".
/// Returns just the word (e.g. "Churning"), stripping the spinner char and
/// the trailing ellipsis.
fn parse_gerund(text: &str) -> Option<String> {
    // Scan the full captured text bottom-up and return the LAST (lowest) match.
    // Tasks push the gerund line upward by a variable amount, so a fixed
    // bottom-N window misses it. Bottom-up + first-match gives us the most
    // recent active gerund without picking up stale ones higher in scrollback.
    let mut result: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        let mut chars = trimmed.chars();
        let Some(first) = chars.next() else {
            continue;
        };
        if !matches!(
            first,
            '\u{00B7}'  // ·
            | '\u{2022}'  // •
            | '\u{273B}'  // ✻
            | '\u{22C6}'  // ⋆
            | '\u{2726}'  // ✦
            | '\u{2727}'  // ✧
            | '\u{2736}' // ✶
            | '\u{2722}' // ✢
            | '\u{273D}' // ✽
            | '\u{2733}' // ✳
        ) {
            continue;
        }
        if chars.next() != Some(' ') {
            continue;
        }
        // Text after "{spinner} " — e.g. "Churning…" or
        // "Implementing foundation model + constants… (1m 28s · ↓ 2.1k tokens)".
        // The trailing "…" marks active work; past-tense lines lack it.
        let rest: String = chars.collect();
        if !rest.contains('\u{2026}') {
            continue;
        }
        // Extract the first word as the gerund verb, append … for display.
        let word = rest.split_whitespace().next().unwrap_or("");
        if word.len() >= 2 {
            let verb = word.trim_end_matches('\u{2026}');
            result = Some(format!("{verb}…"));
        }
    }
    result
}

/// Codex shows "• Working" (U+2022 + space + "Working") when active.
fn parse_codex_activity(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('\u{2022}') {
            let after = trimmed['\u{2022}'.len_utf8()..].trim_start();
            if after.starts_with("Working") {
                return Some("Working…".to_string());
            }
        }
    }
    None
}

/// Opencode shows "esc" near "interrupt" at the bottom when working.
fn parse_opencode_activity(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let bottom: Vec<&str> = lines.iter().rev().take(10).copied().collect();
    let combined = bottom.join(" ");
    if combined.contains("esc") && combined.contains("interrupt") {
        return Some("Working…".to_string());
    }
    None
}

fn segdigit_value(c: char) -> Option<u32> {
    let n = c as u32;
    if (0x1FBF0..=0x1FBF9).contains(&n) {
        Some(n - 0x1FBF0)
    } else if c.is_ascii_digit() {
        Some(n - 0x30)
    } else {
        None
    }
}

fn parse_context(text: &str) -> Option<AgentCtx> {
    // Scan bottom-up: the real statusline is at the bottom of the pane;
    // diff output higher up may contain stray ٪ characters.
    for line in text.lines().rev() {
        let Some(pct_pos) = line.find('\u{066A}') else {
            continue;
        };

        let before = &line[..pct_pos];
        let mut digits: Vec<u32> = Vec::new();
        for c in before.chars().rev() {
            match segdigit_value(c) {
                Some(d) => digits.push(d),
                None => {
                    if !digits.is_empty() {
                        break;
                    }
                }
            }
        }
        if digits.is_empty() {
            continue;
        }
        digits.reverse();
        let pct = (digits.iter().fold(0u32, |a, d| a * 10 + d)).min(100) as u8;

        let after = &line[pct_pos + '\u{066A}'.len_utf8()..];
        let tokens = after
            .split_whitespace()
            .find(|t| t.contains('/') && t.bytes().any(|b| b.is_ascii_digit()))
            .unwrap_or("")
            .to_string();

        // Require a tokens field (e.g. "139k/1.0m") to distinguish the real
        // statusline from mux usage bars that also contain ٪.
        if tokens.is_empty() {
            continue;
        }
        return Some(AgentCtx { pct, tokens });
    }
    None
}

/// Parse "258K window" from codex statusline — just the window size, no usage.
/// The "used" value in codex isn't context usage (that's a separate progress
/// bar we can't parse), so we only report the total window.
fn parse_codex_ctx(text: &str) -> Option<AgentCtx> {
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("window") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let window_idx = parts.iter().position(|&w| w == "window");
        if let Some(wi) = window_idx
            && wi > 0
        {
            let window = parse_k_number(parts[wi - 1])?;
            return Some(AgentCtx {
                pct: 0,
                tokens: fmt_compact(window),
            });
        }
    }
    None
}

/// Parse "20.1K (2%)" into AgentCtx — scans from the bottom of the pane.
/// Computes total window from used / (pct/100) so we can display "20.1k/1.0m".
fn parse_opencode_ctx(text: &str) -> Option<AgentCtx> {
    for line in text.lines().rev() {
        let trimmed = line.trim();
        if let Some(paren_start) = trimmed.rfind('(')
            && let after = &trimmed[paren_start + 1..]
            && let Some(pct_end) = after.find("%)")
            && let Ok(pct) = after[..pct_end].trim().parse::<u8>()
        {
            let before = trimmed[..paren_start].trim();
            let used_str = before.split_whitespace().last().unwrap_or("");
            if let Some(used) = parse_k_number(used_str) {
                let total = if pct > 0 {
                    used / (pct as f64 / 100.0)
                } else {
                    0.0
                };
                return Some(AgentCtx {
                    pct,
                    tokens: fmt_tokens(used, total),
                });
            }
        }
    }
    None
}

/// Parse numbers like "71.3K", "258K", "1.0M", "1.0m" into raw value.
fn parse_k_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.ends_with('K') || s.ends_with('k') {
        s[..s.len() - 1].parse::<f64>().ok().map(|v| v * 1_000.0)
    } else if s.ends_with('M') || s.ends_with('m') {
        s[..s.len() - 1]
            .parse::<f64>()
            .ok()
            .map(|v| v * 1_000_000.0)
    } else {
        s.parse::<f64>().ok()
    }
}

/// Format a raw token count as compact lowercase: 139000 → "139k", 1050000 → "1.0m".
fn fmt_compact(v: f64) -> String {
    if v >= 950_000.0 {
        format!("{:.1}m", v / 1_000_000.0)
    } else if v >= 950.0 {
        let k = v / 1_000.0;
        if k >= 100.0 {
            format!("{}k", k.round() as u64)
        } else if (k * 10.0).fract().abs() < 0.05 {
            format!("{:.0}k", k)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        format!("{}", v.round() as u64)
    }
}

/// Format used/total as compact lowercase string like "139k/1.0m".
fn fmt_tokens(used: f64, total: f64) -> String {
    format!("{}/{}", fmt_compact(used), fmt_compact(total))
}

// ── Claude activity age ──────────────────────────────────────

pub(super) fn query_claude_ages(
    claude_sessions: &[String],
    cwds: &HashMap<String, String>,
) -> HashMap<String, Duration> {
    let home = std::env::var("HOME").unwrap_or_default();
    let projects_root = std::path::PathBuf::from(&home).join(".claude/projects");
    let now = std::time::SystemTime::now();

    let mut result = HashMap::new();
    for session in claude_sessions {
        let Some(cwd) = cwds.get(session) else {
            continue;
        };
        let dir_name: String = cwd
            .chars()
            .map(|c| if c == '/' || c == '.' { '-' } else { c })
            .collect();
        let project_dir = projects_root.join(&dir_name);
        let Some(mtime) = most_recent_jsonl_mtime(&project_dir) else {
            continue;
        };
        if let Ok(elapsed) = now.duration_since(mtime) {
            result.insert(session.clone(), elapsed);
        }
    }
    result
}

fn most_recent_jsonl_mtime(dir: &std::path::Path) -> Option<std::time::SystemTime> {
    let mut best: Option<std::time::SystemTime> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let Ok(mtime) = entry.metadata().and_then(|m| m.modified()) else {
            continue;
        };
        best = Some(match best {
            None => mtime,
            Some(t) if mtime > t => mtime,
            Some(t) => t,
        });
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gerund_near_bottom() {
        let text = "\
some output
✻ Churning…
18٪ 139k/1.0m";
        assert_eq!(parse_gerund(text), Some("Churning…".to_string()));
    }

    #[test]
    fn gerund_pushed_up_by_tasks() {
        // Tasks sit between the gerund and the statusline, pushing the
        // gerund well above the bottom 12 lines.
        let mut lines = vec!["some output"];
        lines.push("✻ Reading…");
        for _ in 0..20 {
            lines.push("  ├ Task in progress");
        }
        lines.push("");
        lines.push("18٪ 139k/1.0m");
        let text = lines.join("\n");
        assert_eq!(parse_gerund(&text), Some("Reading…".to_string()));
    }

    #[test]
    fn past_tense_ignored() {
        let text = "\
✻ Cogitated for 3m
18٪ 139k/1.0m";
        assert_eq!(parse_gerund(text), None);
    }

    #[test]
    fn latest_gerund_wins() {
        let text = "\
✻ Reading…
some middle output
✻ Writing…
18٪ 139k/1.0m";
        assert_eq!(parse_gerund(text), Some("Writing…".to_string()));
    }

    #[test]
    fn no_gerund_when_idle() {
        let text = "\
> some prompt
18٪ 139k/1.0m";
        assert_eq!(parse_gerund(text), None);
    }

    #[test]
    fn multi_word_gerund_extracts_first_word() {
        // Real example: subagent dispatch shows full description after verb.
        let text = "\
✢ Implementing foundation model + constants… (1m 28s · ↓ 2.1k tokens)
  ◼ Phase 1: Foundation
  ◻ Phase 2: Entity Provenance
18٪ 139k/1.0m";
        assert_eq!(parse_gerund(text), Some("Implementing…".to_string()));
    }

    #[test]
    fn all_spinner_chars_recognized() {
        for spinner in ['·', '•', '✻', '⋆', '✦', '✧', '✶', '✢', '✽', '✳'] {
            let text = format!("{spinner} Thinking…");
            assert_eq!(
                parse_gerund(&text),
                Some("Thinking…".to_string()),
                "spinner {spinner:?} (U+{:04X}) not recognized",
                spinner as u32,
            );
        }
    }
}
