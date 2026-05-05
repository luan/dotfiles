use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ratatui::prelude::*;

use crate::palette::{BLUE, MAUVE, PEACH, SUBTEXT0};
use crate::tmux::tmux;

use super::claude::{AgentCtx, query_agent_scrapes, query_claude_ages};
use super::hooks;
use super::pi::query_pi_agents;

// ── Process info ─────────────────────────────────────────────

#[derive(Clone, Default)]
pub(super) struct ProcessTreeInfo {
    pub(super) name: String,
    pub(super) cpu_pct: f32,
    pub(super) mem_bytes: u64,
}

#[derive(Clone, Default)]
struct ProcSample {
    ppid: u32,
    name: String,
    cpu_pct: f32,
    rss_bytes: u64,
}

fn build_process_info() -> HashMap<u32, ProcSample> {
    let out = Command::new("ps")
        .args(["-axo", "pid=,ppid=,pcpu=,rss=,comm="])
        .stderr(Stdio::null())
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut samples: HashMap<u32, ProcSample> = HashMap::new();
    for line in out.lines() {
        let mut it = line.split_whitespace();
        if let (Some(p1), Some(p2), Some(p3), Some(p4)) =
            (it.next(), it.next(), it.next(), it.next())
            && let (Ok(pid), Ok(ppid)) = (p1.parse::<u32>(), p2.parse::<u32>())
        {
            let comm = it.collect::<Vec<_>>().join(" ");
            let basename = comm.rsplit('/').next().unwrap_or(&comm).to_string();
            samples.insert(
                pid,
                ProcSample {
                    ppid,
                    name: basename,
                    cpu_pct: p3.parse::<f32>().unwrap_or(0.0).max(0.0),
                    rss_bytes: p4.parse::<u64>().unwrap_or(0).saturating_mul(1024),
                },
            );
        }
    }
    samples
}

fn build_children(samples: &HashMap<u32, ProcSample>) -> HashMap<u32, Vec<u32>> {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&pid, sample) in samples {
        children.entry(sample.ppid).or_default().push(pid);
    }
    children
}

pub(super) fn legacy_process_index_allocations_for_bench(process_count: u32) -> usize {
    let samples = synthetic_proc_samples_for_bench(process_count);
    let parent_of: HashMap<u32, u32> = samples
        .iter()
        .map(|(&pid, sample)| (pid, sample.ppid))
        .collect();
    let name_of: HashMap<u32, String> = samples
        .iter()
        .map(|(&pid, sample)| (pid, sample.name.clone()))
        .collect();

    let mut total = parent_of.len() + name_of.len();
    for _ in 0..3 {
        let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
        for (&child, &parent) in &parent_of {
            children.entry(parent).or_default().push(child);
        }
        total += children.len();
    }
    total
}

pub(super) fn shared_process_index_allocations_for_bench(process_count: u32) -> usize {
    let samples = synthetic_proc_samples_for_bench(process_count);
    let children = build_children(&samples);
    // The optimized metadata path reads process names directly from `samples`
    // and shares this single child index across CPU, memory/process, and agent
    // detection passes.
    samples.len() + children.len()
}

fn synthetic_proc_samples_for_bench(process_count: u32) -> HashMap<u32, ProcSample> {
    let mut samples = HashMap::with_capacity(process_count as usize);
    for idx in 1..=process_count {
        let ppid = if idx == 1 { 0 } else { idx / 2 };
        samples.insert(
            idx,
            ProcSample {
                ppid,
                name: if idx % 17 == 0 {
                    "claude".to_string()
                } else if idx % 11 == 0 {
                    "rustc".to_string()
                } else {
                    "zsh".to_string()
                },
                cpu_pct: if idx % 11 == 0 { 75.0 } else { 0.5 },
                rss_bytes: u64::from(idx % 128) * 1024 * 1024,
            },
        );
    }
    samples
}

fn query_session_cpu<'a>(
    all_panes: &'a [PaneInfo],
    children: &HashMap<u32, Vec<u32>>,
    samples: &HashMap<u32, ProcSample>,
) -> HashMap<&'a str, f32> {
    let mut result: HashMap<&'a str, f32> = HashMap::new();
    let mut seen_by_session: HashMap<&'a str, HashSet<u32>> = HashMap::new();

    for pane in all_panes {
        let session = pane.session.as_str();
        let seen = seen_by_session.entry(session).or_default();
        let mut stack = vec![pane.pid];

        while let Some(pid) = stack.pop() {
            if !seen.insert(pid) {
                continue;
            }
            *result.entry(session).or_default() += samples
                .get(&pid)
                .map(|sample| sample.cpu_pct)
                .unwrap_or_default();
            if let Some(kids) = children.get(&pid) {
                stack.extend(kids);
            }
        }
    }

    result
}

fn subtree_usage(
    pid: u32,
    children: &HashMap<u32, Vec<u32>>,
    samples: &HashMap<u32, ProcSample>,
    memo: &mut HashMap<u32, (f32, u64)>,
) -> (f32, u64) {
    if let Some(usage) = memo.get(&pid).copied() {
        return usage;
    }
    let mut cpu = samples.get(&pid).map(|s| s.cpu_pct).unwrap_or_default();
    let mut mem = samples.get(&pid).map(|s| s.rss_bytes).unwrap_or_default();
    if let Some(kids) = children.get(&pid) {
        for &kid in kids {
            let (kid_cpu, kid_mem) = subtree_usage(kid, children, samples, memo);
            cpu += kid_cpu;
            mem = mem.saturating_add(kid_mem);
        }
    }
    memo.insert(pid, (cpu, mem));
    (cpu, mem)
}

fn is_shell_or_tmux_root(name: &str) -> bool {
    matches!(
        name,
        "tmux" | "zsh" | "fish" | "bash" | "sh" | "login" | "mux"
    )
}

fn is_agent_process(name: &str) -> bool {
    AGENTS.iter().any(|(agent, _)| name == *agent)
}

fn is_hot(cpu_pct: f32, mem_bytes: u64) -> bool {
    const HOT_CPU_PCT: f32 = 50.0;
    const HOT_MEM_BYTES: u64 = 1024 * 1024 * 1024;

    cpu_pct > HOT_CPU_PCT || mem_bytes > HOT_MEM_BYTES
}

fn dominant_process_root(
    pid: u32,
    children: &HashMap<u32, Vec<u32>>,
    samples: &HashMap<u32, ProcSample>,
    memo: &mut HashMap<u32, (f32, u64)>,
) -> u32 {
    let mut current = pid;
    loop {
        let (current_cpu, current_mem) = subtree_usage(current, children, samples, memo);
        let Some(kids) = children.get(&current) else {
            return current;
        };
        let Some((&best_child, (best_cpu, best_mem))) = kids
            .iter()
            .map(|kid| (kid, subtree_usage(*kid, children, samples, memo)))
            .filter(|(_, (cpu, mem))| is_hot(*cpu, *mem))
            .max_by(|(_, a), (_, b)| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)))
        else {
            return current;
        };

        // Keep walking through wrapper chains (just → uv → cmake → ninja →
        // swift-driver) but stop when work fans out across many children. This
        // names the dominant child tree instead of an unhelpful top-level
        // parent while avoiding a long list of individual compiler workers.
        let cpu_dominant = current_cpu > 0.0 && best_cpu >= current_cpu * 0.5;
        let mem_dominant = current_mem > 0 && best_mem >= current_mem / 2;
        if is_agent_process(
            samples
                .get(&current)
                .map(|s| s.name.as_str())
                .unwrap_or_default(),
        ) || cpu_dominant
            || mem_dominant
        {
            current = best_child;
        } else {
            return current;
        }
    }
}

fn query_session_memory_and_processes<'a>(
    all_panes: &'a [PaneInfo],
    children: &HashMap<u32, Vec<u32>>,
    samples: &HashMap<u32, ProcSample>,
) -> (
    HashMap<&'a str, u64>,
    HashMap<&'a str, Vec<ProcessTreeInfo>>,
) {
    let mut memo: HashMap<u32, (f32, u64)> = HashMap::new();
    let mut session_mem: HashMap<&str, u64> = HashMap::new();
    let mut seen_by_session: HashMap<&str, HashSet<u32>> = HashMap::new();
    let mut roots_by_session: HashMap<&str, Vec<(u32, ProcessTreeInfo)>> = HashMap::new();
    let mut pids_by_pane: Vec<(&PaneInfo, Vec<u32>)> = Vec::new();

    for pane in all_panes {
        let session = pane.session.as_str();
        let seen = seen_by_session.entry(session).or_default();
        let mut stack = vec![pane.pid];
        let mut pane_pids = HashSet::new();

        while let Some(pid) = stack.pop() {
            if !pane_pids.insert(pid) {
                continue;
            }
            if seen.insert(pid) {
                *session_mem.entry(session).or_default() = session_mem
                    .get(session)
                    .copied()
                    .unwrap_or_default()
                    .saturating_add(samples.get(&pid).map(|s| s.rss_bytes).unwrap_or_default());
            }
            if let Some(kids) = children.get(&pid) {
                stack.extend(kids);
            }
        }
        pids_by_pane.push((pane, pane_pids.into_iter().collect()));
    }

    for (pane, pane_pids) in pids_by_pane {
        let mut claimed = HashSet::new();
        let pane_pid_set: HashSet<u32> = pane_pids.into_iter().collect();
        let mut candidates = Vec::new();
        let mut stack = vec![pane.pid];
        while let Some(pid) = stack.pop() {
            if !pane_pid_set.contains(&pid) {
                continue;
            }
            candidates.push(pid);
            if let Some(kids) = children.get(&pid) {
                stack.extend(kids.iter().rev().copied());
            }
        }
        for pid in candidates {
            if claimed.contains(&pid) || pid == pane.pid {
                continue;
            }
            let Some(sample) = samples.get(&pid) else {
                continue;
            };
            if is_shell_or_tmux_root(&sample.name) {
                continue;
            }
            let (cpu_pct, mem_bytes) = subtree_usage(pid, &children, samples, &mut memo);
            if !is_hot(cpu_pct, mem_bytes) {
                continue;
            }
            let pid = dominant_process_root(pid, &children, samples, &mut memo);
            if claimed.contains(&pid) {
                continue;
            }
            let Some(sample) = samples.get(&pid) else {
                continue;
            };
            let (cpu_pct, mem_bytes) = subtree_usage(pid, &children, samples, &mut memo);
            if is_agent_process(&sample.name) {
                continue;
            }
            roots_by_session
                .entry(pane.session.as_str())
                .or_default()
                .push((
                    pid,
                    ProcessTreeInfo {
                        name: sample.name.clone(),
                        cpu_pct,
                        mem_bytes,
                    },
                ));
            let mut subtree = vec![pid];
            while let Some(cur) = subtree.pop() {
                claimed.insert(cur);
                if let Some(kids) = children.get(&cur) {
                    subtree.extend(kids);
                }
            }
        }
    }

    let processes = roots_by_session
        .into_iter()
        .map(|(session, mut entries)| {
            entries.sort_by(|(_, a), (_, b)| {
                b.mem_bytes
                    .cmp(&a.mem_bytes)
                    .then_with(|| b.cpu_pct.total_cmp(&a.cpu_pct))
            });
            entries.dedup_by_key(|(pid, _)| *pid);
            (session, entries.into_iter().map(|(_, info)| info).collect())
        })
        .collect();

    (session_mem, processes)
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

#[derive(Default)]
struct SessionCwdCandidate {
    active_content: Option<String>,
    active_window_content: Option<String>,
}

impl SessionCwdCandidate {
    fn observe(&mut self, window_active: bool, pane_active: bool, is_sidebar: bool, cwd: &str) {
        if !window_active || is_sidebar || cwd.is_empty() {
            return;
        }

        self.active_window_content
            .get_or_insert_with(|| cwd.to_string());
        if pane_active {
            self.active_content = Some(cwd.to_string());
        }
    }

    fn selected(self) -> Option<String> {
        self.active_content.or(self.active_window_content)
    }
}

/// Returns (session, pane_id, agent_name) for every agent found across all panes.
fn query_agents(
    all_panes: &[PaneInfo],
    children: &HashMap<u32, Vec<u32>>,
    samples: &HashMap<u32, ProcSample>,
) -> Vec<(String, String, String)> {
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
            if let Some(name) = samples.get(&pid).map(|sample| sample.name.as_str()) {
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
    pub(super) pr: Option<PullRequestMeta>,
    pub(super) diff: Option<DiffStat>,
    pub(super) cpu_pct: f32,
    pub(super) mem_bytes: u64,
    pub(super) processes: Vec<ProcessTreeInfo>,
    pub(super) agents: Vec<AgentInstance>,
    pub(super) attention: bool,
    pub(super) status: String,
    pub(super) progress: Option<u8>,
}

#[derive(Clone, Debug)]
pub(super) struct PullRequestMeta {
    pub(super) number: u32,
    pub(super) url: String,
    pub(super) review_state: PullRequestReviewState,
    pub(super) ci_state: PullRequestCiState,
    pub(super) checks: Vec<PullRequestCheck>,
    pub(super) unresolved_comments: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PullRequestReviewState {
    Draft,
    InReview,
    ChangesRequested,
    Approved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PullRequestCiState {
    Passing,
    Failing,
    RunningClean,
    RunningFailed,
}

#[derive(Clone, Debug)]
pub(super) struct PullRequestCheck {
    pub(super) name: String,
    pub(super) status: PullRequestCheckStatus,
    pub(super) elapsed: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PullRequestCheckStatus {
    Running,
    Failing,
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

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn parse_gh_time_secs(raw: &str) -> Option<u64> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.timestamp().max(0) as u64)
}

fn check_name(check: &serde_json::Value) -> String {
    check
        .get("name")
        .or_else(|| check.get("context"))
        .or_else(|| check.get("workflowName"))
        .and_then(|v| v.as_str())
        .unwrap_or("check")
        .to_string()
}

fn check_elapsed(check: &serde_json::Value, now_secs: u64) -> Duration {
    check
        .get("startedAt")
        .or_else(|| check.get("createdAt"))
        .and_then(|v| v.as_str())
        .and_then(parse_gh_time_secs)
        .map(|started| Duration::from_secs(now_secs.saturating_sub(started)))
        .unwrap_or_default()
}

fn is_running_check(check: &serde_json::Value) -> bool {
    let status = check
        .get("status")
        .or_else(|| check.get("state"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    matches!(
        status,
        "ACTION_REQUIRED"
            | "EXPECTED"
            | "IN_PROGRESS"
            | "PENDING"
            | "QUEUED"
            | "REQUESTED"
            | "WAITING"
    )
}

fn is_failing_check(check: &serde_json::Value) -> bool {
    let conclusion = check
        .get("conclusion")
        .or_else(|| check.get("state"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    matches!(
        conclusion,
        "ACTION_REQUIRED" | "CANCELLED" | "ERROR" | "FAILURE" | "STARTUP_FAILURE" | "TIMED_OUT"
    )
}

fn selected_pr_checks(
    status_rollup: &[serde_json::Value],
) -> (PullRequestCiState, Vec<PullRequestCheck>) {
    let now_secs = now_epoch_secs();
    let mut failing = Vec::new();
    let mut running = Vec::new();

    for check in status_rollup {
        if is_failing_check(check) {
            failing.push(PullRequestCheck {
                name: check_name(check),
                status: PullRequestCheckStatus::Failing,
                elapsed: check_elapsed(check, now_secs),
            });
        } else if is_running_check(check) {
            running.push(PullRequestCheck {
                name: check_name(check),
                status: PullRequestCheckStatus::Running,
                elapsed: check_elapsed(check, now_secs),
            });
        }
    }

    running.sort_by(|a, b| b.elapsed.cmp(&a.elapsed).then_with(|| a.name.cmp(&b.name)));

    let ci_state = if !running.is_empty() {
        if failing.is_empty() {
            PullRequestCiState::RunningClean
        } else {
            PullRequestCiState::RunningFailed
        }
    } else if failing.is_empty() {
        PullRequestCiState::Passing
    } else {
        PullRequestCiState::Failing
    };

    let mut selected = failing;
    selected.sort_by(|a, b| b.elapsed.cmp(&a.elapsed).then_with(|| a.name.cmp(&b.name)));
    selected.truncate(2);
    selected.extend(running.into_iter().take(2));
    (ci_state, selected)
}

fn git_config(dir: &str, key: &str) -> String {
    Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args(["-C", dir, "config", "--get", key])
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn graphite_url_from_github_url(github_url: &str, number: u32) -> Option<String> {
    let (owner, repo) = github_owner_repo(github_url)?;
    Some(format!(
        "https://app.graphite.com/github/pr/{owner}/{repo}/{number}"
    ))
}

fn github_owner_repo(github_url: &str) -> Option<(String, String)> {
    let path = github_url
        .strip_prefix("https://github.com/")
        .or_else(|| github_url.strip_prefix("http://github.com/"))?;
    let mut parts = path.split('/');
    Some((parts.next()?.to_string(), parts.next()?.to_string()))
}

fn pr_link_url(dir: &str, github_url: &str, number: u32) -> String {
    if git_config(dir, "agents.git-tool") == "graphite" {
        graphite_url_from_github_url(github_url, number).unwrap_or_else(|| github_url.to_string())
    } else {
        github_url.to_string()
    }
}

fn unresolved_review_thread_count(dir: &str, github_url: &str, number: u32) -> u32 {
    let Some((owner, repo)) = github_owner_repo(github_url) else {
        return 0;
    };
    let query = r#"query($owner:String!,$repo:String!,$number:Int!){repository(owner:$owner,name:$repo){pullRequest(number:$number){reviewThreads(first:100){nodes{isResolved}}}}}"#;
    let Ok(mut child) = Command::new("gh")
        .args([
            "api",
            "graphql",
            "-f",
            &format!("query={query}"),
            "-F",
            &format!("owner={owner}"),
            "-F",
            &format!("repo={repo}"),
            "-F",
            &format!("number={number}"),
        ])
        .env("GH_PROMPT_DISABLED", "1")
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return 0;
    };

    let start = Instant::now();
    let out = loop {
        if child.try_wait().ok().flatten().is_some() {
            break child.wait_with_output().ok();
        }
        if start.elapsed() > Duration::from_millis(1500) {
            let _ = child.kill();
            let _ = child.wait();
            return 0;
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    let Some(out) = out.filter(|o| o.status.success()) else {
        return 0;
    };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return 0;
    };
    json.pointer("/data/repository/pullRequest/reviewThreads/nodes")
        .and_then(|v| v.as_array())
        .map(|nodes| {
            nodes
                .iter()
                .filter(|node| {
                    !node
                        .get("isResolved")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true)
                })
                .count() as u32
        })
        .unwrap_or_default()
}

fn gh_pr_meta(dir: &str, branch: &str) -> Option<PullRequestMeta> {
    if branch.is_empty() || matches!(branch, "HEAD" | "main" | "master" | "trunk") {
        return None;
    }

    let mut child = Command::new("gh")
        .args([
            "pr",
            "view",
            "--json",
            "number,url,isDraft,reviewDecision,statusCheckRollup",
        ])
        .env("GH_PROMPT_DISABLED", "1")
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let start = Instant::now();
    let out = loop {
        if child.try_wait().ok().flatten().is_some() {
            break child.wait_with_output().ok()?;
        }
        if start.elapsed() > Duration::from_millis(1500) {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    if !out.status.success() {
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let number = json.get("number").and_then(|v| v.as_u64())? as u32;
    let github_url = json
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if github_url.is_empty() {
        return None;
    }
    let url = pr_link_url(dir, &github_url, number);

    let review_state = if json
        .get("isDraft")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        PullRequestReviewState::Draft
    } else {
        match json
            .get("reviewDecision")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
        {
            "APPROVED" => PullRequestReviewState::Approved,
            "CHANGES_REQUESTED" => PullRequestReviewState::ChangesRequested,
            _ => PullRequestReviewState::InReview,
        }
    };

    let rollup = json
        .get("statusCheckRollup")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let (ci_state, checks) = selected_pr_checks(&rollup);
    let unresolved_comments = unresolved_review_thread_count(dir, &github_url, number);

    Some(PullRequestMeta {
        number,
        url,
        review_state,
        ci_state,
        checks,
        unresolved_comments,
    })
}

fn cached_gh_pr_meta(dir: &str, branch: &str) -> Option<PullRequestMeta> {
    static CACHE: OnceLock<Mutex<HashMap<String, (Instant, Option<PullRequestMeta>)>>> =
        OnceLock::new();
    const HIT_TTL: Duration = Duration::from_secs(60);
    const MISS_TTL: Duration = Duration::from_secs(10);

    let key = format!("{dir}\x1f{branch}");
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock()
        && let Some((fetched_at, value)) = guard.get(&key)
        && fetched_at.elapsed() < if value.is_some() { HIT_TTL } else { MISS_TTL }
    {
        return value.clone();
    }

    let value = gh_pr_meta(dir, branch);
    if let Ok(mut guard) = cache.lock() {
        guard.insert(key, (Instant::now(), value.clone()));
    }
    value
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

    let mut cwd_candidates: HashMap<String, SessionCwdCandidate> = HashMap::new();
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
        let is_sidebar = false;

        if let Ok(pid) = pid_str.parse::<u32>() {
            all_panes.push(PaneInfo {
                session: session.clone(),
                pane_id: pane_id.clone(),
                pid,
            });
            pane_cwds.insert((session.clone(), pane_id), cwd.to_string());

            cwd_candidates.entry(session).or_default().observe(
                window_active,
                pane_active,
                is_sidebar,
                cwd,
            );
        }
    }

    let cwds: HashMap<String, String> = cwd_candidates
        .into_iter()
        .filter_map(|(session, candidate)| candidate.selected().map(|cwd| (session, cwd)))
        .collect();

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

    let samples = build_process_info();
    let children = build_children(&samples);
    let session_cpu = query_session_cpu(&all_panes, &children, &samples);
    let (session_mem, session_processes) =
        query_session_memory_and_processes(&all_panes, &children, &samples);
    let pi_agents = query_pi_agents(&all_panes);
    let agent_hits = query_agents(&all_panes, &children, &samples);

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
    let mut pr_cache: HashMap<String, Option<PullRequestMeta>> = HashMap::new();
    for cwd in cwds.values() {
        if !cwd.is_empty() && !branch_cache.contains_key(cwd) {
            let branch = git_branch(cwd);
            branch_cache.insert(cwd.clone(), branch.clone());
            diff_cache.insert(cwd.clone(), git_diff_stat(cwd));
            pr_cache.insert(cwd.clone(), cached_gh_pr_meta(cwd, &branch));
        }
    }

    let mut result = HashMap::new();
    for name in sessions {
        let cwd = cwds.get(name).cloned().unwrap_or_default();
        let branch = branch_cache.get(&cwd).cloned().unwrap_or_default();
        let diff = diff_cache.get(&cwd).copied().flatten();
        let pr = pr_cache.get(&cwd).cloned().flatten();

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
                pr,
                diff,
                cpu_pct: session_cpu.get(name.as_str()).copied().unwrap_or(0.0),
                mem_bytes: session_mem.get(name.as_str()).copied().unwrap_or(0),
                processes: session_processes
                    .get(name.as_str())
                    .cloned()
                    .unwrap_or_default(),
                agents: session_agents,
                attention: needs_attention,
                status: statuses.get(name).cloned().unwrap_or_default(),
                progress: progresses.get(name).copied(),
            },
        );
    }
    (result, tmux_calls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cwd_candidate_ignores_active_sidebar_and_falls_back_to_content_pane() {
        let mut candidate = SessionCwdCandidate::default();

        candidate.observe(true, true, true, "/repo/sidebar");
        candidate.observe(true, false, false, "/repo/content");

        assert_eq!(candidate.selected().as_deref(), Some("/repo/content"));
    }

    #[test]
    fn cwd_candidate_prefers_active_content_pane_over_other_active_window_content() {
        let mut candidate = SessionCwdCandidate::default();

        candidate.observe(true, false, false, "/repo/first");
        candidate.observe(true, true, false, "/repo/active");

        assert_eq!(candidate.selected().as_deref(), Some("/repo/active"));
    }
}
