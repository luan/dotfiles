use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::time::Duration;

use ratatui::prelude::*;

use crate::palette::{BLUE, MAUVE, PEACH, SUBTEXT0};
use crate::tmux::tmux;

use super::claude::{AgentCtx, query_agent_scrapes, query_claude_ages};

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

/// Low-saturation purple for opencode's identity color.
const OPENCODE_COLOR: Color = Color::Rgb(0x9A, 0x8F, 0xBF);
/// Sky blue matching codex's usage bar provider color.
const CODEX_AGENT_COLOR: Color = Color::Rgb(0x74, 0xC7, 0xEC);

const AGENTS: &[(&str, Color)] = &[
    ("claude", PEACH),
    ("codex", CODEX_AGENT_COLOR),
    ("opencode", OPENCODE_COLOR),
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
        _ => None,
    }
}

struct PaneInfo {
    session: String,
    pane_id: String,
    pid: u32,
}

/// Returns (session, pane_id, agent_name) for every agent found across all panes.
fn query_agents(
    all_panes: &[PaneInfo],
    parent_of: &HashMap<u32, u32>,
    name_of: &HashMap<u32, String>,
) -> Vec<(String, String, String)> {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&c, &p) in parent_of {
        children.entry(p).or_default().push(c);
    }

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
    pub(super) agents: Vec<AgentInstance>,
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
    // Active-pane pids for port detection (only the active pane per session).
    let mut active_pane_pids: HashMap<String, u32> = HashMap::new();
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
                pane_id,
                pid,
            });

            if window_active && pane_active {
                cwds.insert(session.clone(), cwd.to_string());
                active_pane_pids.insert(session, pid);
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
    let agent_hits = query_agents(&all_panes, &parent_of, &name_of);

    let (scrape_map, scrape_calls) = query_agent_scrapes(&agent_hits);
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
    for cwd in cwds.values() {
        if !cwd.is_empty() && !branch_cache.contains_key(cwd) {
            branch_cache.insert(cwd.clone(), git_branch(cwd));
        }
    }

    let ports_map = query_ports(&active_pane_pids, &parent_of);

    let mut result = HashMap::new();
    for name in sessions {
        let cwd = cwds.get(name).cloned().unwrap_or_default();
        let branch = branch_cache.get(&cwd).cloned().unwrap_or_default();

        let session_agents: Vec<AgentInstance> = agent_hits
            .iter()
            .filter(|(s, _, _)| s == name)
            .map(|(s, pane_id, agent_name)| {
                let scrape = scrape_map.get(&(s.clone(), pane_id.clone()));
                AgentInstance {
                    name: agent_name.clone(),
                    pane_id: pane_id.clone(),
                    gerund: scrape.and_then(|sc| sc.gerund.clone()),
                    ctx: scrape.and_then(|sc| sc.ctx.clone()),
                    age: if agent_name == "claude" {
                        claude_age_map.get(s).copied()
                    } else {
                        None
                    },
                    asking: scrape.is_some_and(|sc| sc.asking),
                }
            })
            .collect();

        result.insert(
            name.clone(),
            SessionMeta {
                branch,
                agents: session_agents,
                attention: *attn.get(name).unwrap_or(&false),
                ports: ports_map.get(name).cloned().unwrap_or_default(),
                status: statuses.get(name).cloned().unwrap_or_default(),
                progress: progresses.get(name).copied(),
            },
        );
    }
    (result, tmux_calls)
}
