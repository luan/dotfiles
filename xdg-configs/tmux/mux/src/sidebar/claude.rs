use std::collections::HashMap;
use std::time::Duration;

use tracing::error;

use crate::tmux::tmux;

// ── Claude context % via pane statusline scraping ─────────────
// Claude Code's statusline is rendered right in the pane. We capture
// the last lines via `tmux capture-pane` and parse the segmented digit
// + ٪ (U+066A ARABIC PERCENT SIGN) pattern emitted by statusline.py.
// The FIRST ٪ in the statusline is the context percentage.

#[derive(Clone, Default)]
pub(super) struct ClaudeCtx {
    pub(super) pct: u8,
    pub(super) tokens: String,
}

#[derive(Clone, Default)]
pub(super) struct ClaudeScrape {
    pub(super) ctx: Option<ClaudeCtx>,
    pub(super) activity: Option<String>,
}

/// Returns (scrape_map, tmux_call_count).
pub(super) fn query_claude_scrapes(
    claude_sessions: &[String],
) -> (HashMap<String, ClaudeScrape>, u32) {
    if claude_sessions.is_empty() {
        return (HashMap::new(), 0);
    }

    // Batch all capture-pane calls into a single tmux invocation.
    // We interleave `display-message -p "<<DELIM>>"` between captures
    // so we can split the combined output reliably.
    const DELIM: &str = "\x1e<<MUX_CAPTURE_DELIM>>\x1e";
    let mut args: Vec<String> = Vec::new();
    for (i, session) in claude_sessions.iter().enumerate() {
        if i > 0 {
            // Separator: print a delimiter line between captures
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
            session.clone(),
            "-p".into(),
            "-S".into(),
            "-30".into(),
        ]);
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let raw = tmux(&refs);

    let chunks: Vec<&str> = raw.split(DELIM).collect();
    if chunks.len() != claude_sessions.len() {
        // Delimiter collision: a pane contained the sentinel string,
        // corrupting the session-to-chunk mapping. Fall back to
        // sequential per-session captures.
        error!(
            expected = claude_sessions.len(),
            got = chunks.len(),
            "capture-pane delimiter collision, falling back to sequential"
        );
        return scrape_sessions_sequential(claude_sessions);
    }

    let mut result = HashMap::new();
    for (i, session) in claude_sessions.iter().enumerate() {
        insert_scrape(&mut result, session, chunks[i]);
    }
    (result, 1)
}

fn insert_scrape(result: &mut HashMap<String, ClaudeScrape>, session: &str, text: &str) {
    let scrape = ClaudeScrape {
        ctx: parse_context(text),
        activity: parse_activity(text),
    };
    if scrape.ctx.is_some() || scrape.activity.is_some() {
        result.insert(session.to_string(), scrape);
    }
}

fn scrape_sessions_sequential(sessions: &[String]) -> (HashMap<String, ClaudeScrape>, u32) {
    let mut result = HashMap::new();
    for session in sessions {
        let text = tmux(&["capture-pane", "-t", session, "-p", "-S", "-30"]);
        insert_scrape(&mut result, session, &text);
    }
    (result, sessions.len() as u32)
}

fn parse_activity(text: &str) -> Option<String> {
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
        ) {
            continue;
        }
        if chars.next() != Some(' ') {
            continue;
        }
        if trimmed.chars().count() < 5 {
            continue;
        }
        return Some(trimmed.to_string());
    }
    None
}

fn segdigit_value(c: char) -> Option<u32> {
    let n = c as u32;
    if (0x1FBF0..=0x1FBF9).contains(&n) {
        Some(n - 0x1FBF0)
    } else {
        None
    }
}

fn parse_context(text: &str) -> Option<ClaudeCtx> {
    for line in text.lines() {
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

        return Some(ClaudeCtx { pct, tokens });
    }
    None
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
