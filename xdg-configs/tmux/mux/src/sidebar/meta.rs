use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::time::Duration;

use ratatui::prelude::*;

use crate::palette::{BLUE, MAUVE, PEACH, SUBTEXT0};
use crate::tmux::tmux;

use super::claude::{AgentCtx, query_agent_scrapes, query_claude_ages};
use super::hooks;
use super::pi::query_pi_agents;

// ── Process info ─────────────────────────────────────────────

fn build_process_info() -> (HashMap<u32, u32>, HashMap<u32, String>, HashMap<u32, f32>) {
    let out = Command::new("ps")
        .args(["-axo", "pid=,ppid=,pcpu=,comm="])
        .stderr(Stdio::null())
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut parent_of: HashMap<u32, u32> = HashMap::new();
    let mut name_of: HashMap<u32, String> = HashMap::new();
    let mut cpu_of: HashMap<u32, f32> = HashMap::new();
    for line in out.lines() {
        let mut it = line.split_whitespace();
        if let (Some(p1), Some(p2), Some(p3)) = (it.next(), it.next(), it.next())
            && let (Ok(pid), Ok(ppid)) = (p1.parse::<u32>(), p2.parse::<u32>())
        {
            let comm = it.collect::<Vec<_>>().join(" ");
            let basename = comm.rsplit('/').next().unwrap_or(&comm).to_string();
            parent_of.insert(pid, ppid);
            name_of.insert(pid, basename);
            if let Ok(cpu) = p3.parse::<f32>() {
                cpu_of.insert(pid, cpu.max(0.0));
            }
        }
    }
    (parent_of, name_of, cpu_of)
}

fn build_children(parent_of: &HashMap<u32, u32>) -> HashMap<u32, Vec<u32>> {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&child, &parent) in parent_of {
        children.entry(parent).or_default().push(child);
    }
    children
}

fn query_session_cpu(
    all_panes: &[PaneInfo],
    parent_of: &HashMap<u32, u32>,
    cpu_of: &HashMap<u32, f32>,
) -> HashMap<String, f32> {
    let children = build_children(parent_of);
    let mut result: HashMap<String, f32> = HashMap::new();
    let mut seen_by_session: HashMap<String, HashSet<u32>> = HashMap::new();

    for pane in all_panes {
        let seen = seen_by_session.entry(pane.session.clone()).or_default();
        let mut stack = vec![pane.pid];

        while let Some(pid) = stack.pop() {
            if !seen.insert(pid) {
                continue;
            }
            *result.entry(pane.session.clone()).or_default() +=
                cpu_of.get(&pid).copied().unwrap_or(0.0);
            if let Some(kids) = children.get(&pid) {
                stack.extend(kids);
            }
        }
    }

    result
}

// ── Agent detection ──────────────────────────────────────────

/// Low-saturation purple for opencode's identity color.
const OPENCODE_COLOR: Color = Color::Rgb(0x9A, 0x8F, 0xBF);
/// Sky blue matching codex's usage bar provider color.
const CODEX_AGENT_COLOR: Color = Color::Rgb(0x74, 0xC7, 0xEC);
/// Opencode-adjacent blue-lavender for Pi.
const PI_AGENT_COLOR: Color = Color::Rgb(0x82, 0x97, 0xD6);

const AGENTS: &[(&str, Color)] = &[
    ("claude", PEACH),
    ("codex", CODEX_AGENT_COLOR),
    ("opencode", OPENCODE_COLOR),
    ("pi", PI_AGENT_COLOR),
    ("aider", MAUVE),
    ("cursor-agent", BLUE),
    ("gemini", BLUE),
];

pub(super) fn agent_color(name: &str) -> Color {
    AGENTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| *c)
        .unwrap_or(SUBTEXT0)
}

/// Brand glyph for a known agent (Font Awesome 7 Brands / Nerd Font). None
/// means "no glyph assigned" — caller falls back to the textual name.
pub(super) fn agent_glyph(name: &str) -> Option<&'static str> {
    match name {
        "claude" => Some("\u{e861}"),
        "codex" => Some("\u{e7cf}"),
        "opencode" => Some("\u{f0b16}"),
        "pi" => Some("\u{e22c}"),
        _ => None,
    }
}

pub(super) struct PaneInfo {
    pub(super) session: String,
    pub(super) pane_id: String,
    pub(super) pid: u32,
}

/// Returns (session, pane_id, agent_name) for every agent found across all panes.
fn query_agents(
    all_panes: &[PaneInfo],
    parent_of: &HashMap<u32, u32>,
    name_of: &HashMap<u32, String>,
) -> Vec<(String, String, String)> {
    let children = build_children(parent_of);

    let mut result: Vec<(String, String, String)> = Vec::new();

    for pane in all_panes {
        let mut stack = vec![pane.pid];
        let mut visited: HashSet<u32> = HashSet::new();
        // Track which pids we've claimed for an agent so we skip their subtrees.
        let mut skip_subtree: HashSet<u32> = HashSet::new();

        while let Some(pid) = stack.pop() {
            if !visited.insert(pid) {
                continue;
            }
            // If this pid belongs to an agent subtree we already claimed, skip it
            // (but still walk siblings — skip_subtree only blocks the *children*).
            if skip_subtree.contains(&pid) {
                continue;
            }
            if let Some(name) = name_of.get(&pid) {
                let lower = name.to_ascii_lowercase();
                let agent_match = AGENTS.iter().find(|(a, _)| lower == *a);
                if let Some((agent_name, _)) = agent_match {
                    let key = (pane.session.clone(), pane.pane_id.clone());
                    // Dedup: same (session, pane_id, agent_name) should appear once.
                    let already = result
                        .iter()
                        .any(|(s, p, n)| s == &key.0 && p == &key.1 && n == *agent_name);
                    if !already {
                        result.push((
                            pane.session.clone(),
                            pane.pane_id.clone(),
                            (*agent_name).to_string(),
                        ));
                    }
                    // Mark children of this agent pid as skip so nested tools
                    // aren't double-detected as separate agents.
                    if let Some(kids) = children.get(&pid) {
                        skip_subtree.extend(kids);
                    }
                    // Don't push children onto main stack either.
                    continue;
                }
            }
            if let Some(kids) = children.get(&pid) {
                stack.extend(kids);
            }
        }
    }

    result
}

// ── Rich metadata ────────────────────────────────────────────

#[derive(Clone)]
pub(super) struct AgentInstance {
    pub(super) name: String,
    pub(super) pane_id: String,
    pub(super) gerund: Option<String>,
    pub(super) ctx: Option<AgentCtx>,
    pub(super) age: Option<Duration>,
    pub(super) asking: bool,
}

#[derive(Default, Clone)]
pub(super) struct SessionMeta {
    pub(super) branch: String,
    pub(super) diff: Option<DiffStat>,
    pub(super) cpu_pct: f32,
    pub(super) agents: Vec<AgentInstance>,
    pub(super) attention: bool,
    pub(super) status: String,
    pub(super) progress: Option<u8>,
}

#[derive(Default, Clone, Copy)]
pub(super) struct DiffStat {
    pub(super) added: u32,
    pub(super) removed: u32,
}

fn git_branch(dir: &str) -> String {
    Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args(["-C", dir, "rev-parse", "--abbrev-ref", "HEAD"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn git_diff_stat(dir: &str) -> Option<DiffStat> {
    let out = Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args(["-C", dir, "diff", "HEAD", "--numstat"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    let raw = String::from_utf8_lossy(&out.stdout);
    let mut stat = DiffStat::default();
    for line in raw.lines() {
        let mut parts = line.split('\t');
        let added = parts.next().and_then(|s| s.parse::<u32>().ok());
        let removed = parts.next().and_then(|s| s.parse::<u32>().ok());
        if let (Some(added), Some(removed)) = (added, removed) {
            stat.added = stat.added.saturating_add(added);
            stat.removed = stat.removed.saturating_add(removed);
        }
    }

    (stat.added > 0 || stat.removed > 0).then_some(stat)
}

fn min_duration(a: Option<Duration>, b: Option<Duration>) -> Option<Duration> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Returns (meta_map, tmux_call_count).
pub(super) fn query_session_meta(sessions: &[String]) -> (HashMap<String, SessionMeta>, u32) {
    let mut tmux_calls = 0u32;

    // Batch list-panes + list-sessions into one tmux invocation.
    // A sentinel line separates the two outputs so a tab in a session name
    // cannot cause a panes row to be misidentified as a sessions row.
    const META_DELIM: &str = "\x1e<<MUX_META_DELIM>>\x1e";
    let combined = tmux(&[
        "list-panes",
        "-a",
        "-F",
        "#{session_name}\t#{window_active}\t#{pane_active}\t#{pane_current_path}\t#{pane_pid}\t#{pane_id}",
        ";",
        "display-message",
        "-p",
        META_DELIM,
        ";",
        "list-sessions",
        "-F",
        "#{session_name}\t#{@attention}\t#{@sidebar_status}\t#{@sidebar_progress}",
    ]);
    tmux_calls += 1;

    let mut cwds: HashMap<String, String> = HashMap::new();
    let mut pane_cwds: HashMap<(String, String), String> = HashMap::new();
    let mut all_panes: Vec<PaneInfo> = Vec::new();
    let mut attn: HashMap<String, bool> = HashMap::new();
    let mut statuses: HashMap<String, String> = HashMap::new();
    let mut progresses: HashMap<String, u8> = HashMap::new();

    let (panes_section, sessions_section) =
        combined.split_once(META_DELIM).unwrap_or((&combined, ""));

    for line in panes_section.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 6 {
            continue;
        }
        let session = parts[0].to_string();
        let window_active = parts[1] == "1";
        let pane_active = parts[2] == "1";
        let cwd = parts[3];
        let pid_str = parts[4];
        let pane_id = parts[5].to_string();

        if let Ok(pid) = pid_str.parse::<u32>() {
            all_panes.push(PaneInfo {
                session: session.clone(),
                pane_id: pane_id.clone(),
                pid,
            });
            pane_cwds.insert((session.clone(), pane_id), cwd.to_string());

            if window_active && pane_active {
                cwds.insert(session.clone(), cwd.to_string());
            }
        }
    }

    for line in sessions_section.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() || parts[0].is_empty() {
            continue;
        }
        let name = parts[0];
        if parts.len() > 1 && parts[1] == "1" {
            attn.insert(name.to_string(), true);
        }
        if parts.len() > 2 && !parts[2].is_empty() {
            statuses.insert(name.to_string(), parts[2].to_string());
        }
        if parts.len() > 3
            && let Ok(p) = parts[3].parse::<u8>()
        {
            progresses.insert(name.to_string(), p.min(100));
        }
    }

    let (parent_of, name_of, cpu_of) = build_process_info();
    let session_cpu = query_session_cpu(&all_panes, &parent_of, &cpu_of);
    let pi_agents = query_pi_agents(&all_panes);
    let agent_hits = query_agents(&all_panes, &parent_of, &name_of);

    let scrape_targets: Vec<(String, String, String)> = agent_hits
        .iter()
        .filter(|(session, pane_id, name)| {
            name != "pi" || !pi_agents.contains_key(&(session.clone(), pane_id.clone()))
        })
        .cloned()
        .collect();

    let (scrape_map, scrape_calls) = query_agent_scrapes(&scrape_targets);
    tmux_calls += scrape_calls;

    let claude_sessions: Vec<String> = agent_hits
        .iter()
        .filter(|(_, _, name)| name == "claude")
        .map(|(s, _, _)| s.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let claude_age_map = query_claude_ages(&claude_sessions, &cwds);

    let mut branch_cache: HashMap<String, String> = HashMap::new();
    let mut diff_cache: HashMap<String, Option<DiffStat>> = HashMap::new();
    for cwd in cwds.values() {
        if !cwd.is_empty() && !branch_cache.contains_key(cwd) {
            branch_cache.insert(cwd.clone(), git_branch(cwd));
            diff_cache.insert(cwd.clone(), git_diff_stat(cwd));
        }
    }

    let mut result = HashMap::new();
    for name in sessions {
        let cwd = cwds.get(name).cloned().unwrap_or_default();
        let branch = branch_cache.get(&cwd).cloned().unwrap_or_default();
        let diff = diff_cache.get(&cwd).copied().flatten();

        let mut session_agents: Vec<AgentInstance> = agent_hits
            .iter()
            .filter(|(s, _, _)| s == name)
            .filter(|(s, pane_id, agent_name)| {
                agent_name != "pi" || !pi_agents.contains_key(&(s.clone(), pane_id.clone()))
            })
            .map(|(s, pane_id, agent_name)| {
                let scrape = scrape_map.get(&(s.clone(), pane_id.clone()));
                let hook = if agent_name == "claude" || agent_name == "codex" {
                    if let Some(cwd) = pane_cwds.get(&(s.clone(), pane_id.clone())) {
                        if agent_name == "claude" {
                            hooks::install(cwd);
                        }
                    }
                    hooks::read_signal(pane_id)
                } else {
                    None
                };
                let hook_age = hook.as_ref().and_then(|h| h.age);
                AgentInstance {
                    name: agent_name.clone(),
                    pane_id: pane_id.clone(),
                    gerund: if hook.as_ref().is_some_and(|h| h.idle) {
                        None
                    } else {
                        scrape
                            .and_then(|sc| sc.gerund.clone())
                            .or_else(|| hook.as_ref().and_then(|h| h.gerund.clone()))
                    },
                    ctx: scrape.and_then(|sc| sc.ctx.clone()),
                    age: if agent_name == "claude" {
                        min_duration(claude_age_map.get(s).copied(), hook_age)
                    } else {
                        hook_age
                    },
                    asking: scrape.is_some_and(|sc| sc.asking)
                        || hook.as_ref().is_some_and(|h| h.asking),
                }
            })
            .collect();
        session_agents.extend(
            all_panes
                .iter()
                .filter(|pane| pane.session == *name)
                .filter_map(|pane| pi_agents.get(&(pane.session.clone(), pane.pane_id.clone())))
                .cloned(),
        );
        let needs_attention =
            *attn.get(name).unwrap_or(&false) || session_agents.iter().any(|agent| agent.asking);

        result.insert(
            name.clone(),
            SessionMeta {
                branch,
                diff,
                cpu_pct: session_cpu.get(name).copied().unwrap_or(0.0),
                agents: session_agents,
                attention: needs_attention,
                status: statuses.get(name).cloned().unwrap_or_default(),
                progress: progresses.get(name).copied(),
            },
        );
    }
    (result, tmux_calls)
}
