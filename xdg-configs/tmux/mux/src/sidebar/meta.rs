use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::time::Duration;

use ratatui::prelude::*;

use crate::palette::{BLUE, GREEN, MAUVE, PEACH, SUBTEXT0};
use crate::tmux::tmux;

use super::claude::{ClaudeCtx, query_claude_ages, query_claude_scrapes};

// ── Process info ─────────────────────────────────────────────

fn build_process_info() -> (HashMap<u32, u32>, HashMap<u32, String>) {
    let out = Command::new("ps")
        .args(["-axo", "pid=,ppid=,comm="])
        .stderr(Stdio::null())
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut parent_of: HashMap<u32, u32> = HashMap::new();
    let mut name_of: HashMap<u32, String> = HashMap::new();
    for line in out.lines() {
        let mut it = line.split_whitespace();
        if let (Some(p1), Some(p2)) = (it.next(), it.next())
            && let (Ok(pid), Ok(ppid)) = (p1.parse::<u32>(), p2.parse::<u32>())
        {
            let comm = it.collect::<Vec<_>>().join(" ");
            let basename = comm.rsplit('/').next().unwrap_or(&comm).to_string();
            parent_of.insert(pid, ppid);
            name_of.insert(pid, basename);
        }
    }
    (parent_of, name_of)
}

// ── Agent detection ──────────────────────────────────────────

const AGENTS: &[(&str, Color)] = &[
    ("claude", PEACH),
    ("codex", GREEN),
    ("opencode", BLUE),
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
        "opencode" => Some("\u{f113a}"),
        _ => None,
    }
}

fn query_agents(
    pane_pids: &HashMap<String, u32>,
    parent_of: &HashMap<u32, u32>,
    name_of: &HashMap<u32, String>,
) -> HashMap<String, String> {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&c, &p) in parent_of {
        children.entry(p).or_default().push(c);
    }

    let mut result = HashMap::new();
    for (session, &pane_pid) in pane_pids {
        let mut stack = vec![pane_pid];
        let mut visited = HashSet::new();
        'walk: while let Some(pid) = stack.pop() {
            if !visited.insert(pid) {
                continue;
            }
            if let Some(name) = name_of.get(&pid) {
                let lower = name.to_ascii_lowercase();
                for (agent, _) in AGENTS {
                    if lower == *agent {
                        result.insert(session.clone(), (*agent).to_string());
                        break 'walk;
                    }
                }
            }
            if let Some(kids) = children.get(&pid) {
                stack.extend(kids);
            }
        }
    }
    result
}

// ── Port detection ───────────────────────────────────────────

fn query_ports(
    pane_pids: &HashMap<String, u32>,
    parent_of: &HashMap<u32, u32>,
) -> HashMap<String, Vec<u16>> {
    if pane_pids.is_empty() {
        return HashMap::new();
    }

    let lsof_out = Command::new("lsof")
        .args(["-i", "-P", "-n", "-sTCP:LISTEN", "-F", "pn"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut pid_ports: HashMap<u32, Vec<u16>> = HashMap::new();
    let mut cur_pid = 0u32;
    for line in lsof_out.lines() {
        if let Some(p) = line.strip_prefix('p') {
            cur_pid = p.parse().unwrap_or(0);
        } else if let Some(n) = line.strip_prefix('n')
            && let Some(port_str) = n.rsplit(':').next()
            && let Ok(port) = port_str.parse::<u16>()
        {
            pid_ports.entry(cur_pid).or_default().push(port);
        }
    }

    if pid_ports.is_empty() {
        return HashMap::new();
    }

    let pid_to_session: HashMap<u32, &str> = pane_pids
        .iter()
        .map(|(name, pid)| (*pid, name.as_str()))
        .collect();

    let mut result: HashMap<String, Vec<u16>> = HashMap::new();
    for (pid, ports) in &pid_ports {
        let mut cur = *pid;
        for _ in 0..100 {
            if let Some(&session) = pid_to_session.get(&cur) {
                result.entry(session.to_string()).or_default().extend(ports);
                break;
            }
            match parent_of.get(&cur) {
                Some(&ppid) if ppid != 0 && ppid != cur => cur = ppid,
                _ => break,
            }
        }
    }

    for ports in result.values_mut() {
        ports.sort();
        ports.dedup();
    }
    result
}

// ── Rich metadata ────────────────────────────────────────────

#[derive(Default, Clone)]
pub(super) struct SessionMeta {
    pub(super) branch: String,
    pub(super) agent: String,
    pub(super) claude_ctx: Option<ClaudeCtx>,
    pub(super) claude_age: Option<Duration>,
    pub(super) claude_activity: Option<String>,
    pub(super) attention: bool,
    pub(super) ports: Vec<u16>,
    pub(super) status: String,
    pub(super) progress: Option<u8>,
}

fn git_branch(dir: &str) -> String {
    Command::new("git")
        .args(["-C", dir, "rev-parse", "--abbrev-ref", "HEAD"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
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
        "#{session_name}\t#{window_active}\t#{pane_active}\t#{pane_current_path}\t#{pane_pid}",
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
    let mut pane_pids: HashMap<String, u32> = HashMap::new();
    let mut attn: HashMap<String, bool> = HashMap::new();
    let mut statuses: HashMap<String, String> = HashMap::new();
    let mut progresses: HashMap<String, u8> = HashMap::new();

    let (panes_section, sessions_section) =
        combined.split_once(META_DELIM).unwrap_or((&combined, ""));

    for line in panes_section.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 5 && parts[1] == "1" && parts[2] == "1" {
            cwds.insert(parts[0].to_string(), parts[3].to_string());
            if let Ok(pid) = parts[4].parse::<u32>() {
                pane_pids.insert(parts[0].to_string(), pid);
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

    let (parent_of, name_of) = build_process_info();
    let agents_map = query_agents(&pane_pids, &parent_of, &name_of);

    let claude_sessions: Vec<String> = agents_map
        .iter()
        .filter(|(_, name)| name.as_str() == "claude")
        .map(|(s, _)| s.clone())
        .collect();
    let (claude_scrape_map, scrape_calls) = query_claude_scrapes(&claude_sessions);
    tmux_calls += scrape_calls;
    let claude_age_map = query_claude_ages(&claude_sessions, &cwds);

    let mut branch_cache: HashMap<String, String> = HashMap::new();
    for cwd in cwds.values() {
        if !cwd.is_empty() && !branch_cache.contains_key(cwd) {
            branch_cache.insert(cwd.clone(), git_branch(cwd));
        }
    }

    let ports_map = query_ports(&pane_pids, &parent_of);

    let mut result = HashMap::new();
    for name in sessions {
        let cwd = cwds.get(name).cloned().unwrap_or_default();
        let branch = branch_cache.get(&cwd).cloned().unwrap_or_default();
        result.insert(
            name.clone(),
            SessionMeta {
                branch,
                agent: agents_map.get(name).cloned().unwrap_or_default(),
                claude_ctx: claude_scrape_map.get(name).and_then(|s| s.ctx.clone()),
                claude_age: claude_age_map.get(name).copied(),
                claude_activity: claude_scrape_map.get(name).and_then(|s| s.activity.clone()),
                attention: *attn.get(name).unwrap_or(&false),
                ports: ports_map.get(name).cloned().unwrap_or_default(),
                status: statuses.get(name).cloned().unwrap_or_default(),
                progress: progresses.get(name).copied(),
            },
        );
    }
    (result, tmux_calls)
}
