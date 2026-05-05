use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::order::compute_order;
use crate::process::spawn_reaped;
use crate::tmux::{home, tmux};
use crate::usage_bars;

use super::ACTIVITY_GRACE;
use super::claude::AgentCtx;
use super::instrument::{LatencySummary, SidebarCounters};
use super::meta::{
    AgentInstance, DiffStat, ProcessTreeInfo, PullRequestCheck, PullRequestCheckStatus,
    PullRequestCiState, PullRequestMeta, PullRequestReviewState, SessionMeta, query_session_meta,
};

const SNAPSHOT_VERSION: u32 = 15;
const SNAPSHOT_STALE: Duration = Duration::from_secs(5);
const TICK: Duration = Duration::from_millis(500);
const META_INTERVAL: Duration = Duration::from_secs(5);
const IDLE_EXIT_AFTER: Duration = Duration::from_secs(30);
const SIDEBAR_TOKEN: &str = super::SIDEBAR_TOKEN;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct SidebarSnapshot {
    version: u32,
    generated_at_ms: u64,
    pub(super) notched: bool,
    pub(super) alive_sessions: Vec<String>,
    pub(super) pane_sessions: HashMap<String, String>,
    meta: Vec<(String, SessionMetaSnapshot)>,
    usage_lines: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SessionMetaSnapshot {
    branch: String,
    pr: Option<PullRequestSnapshot>,
    diff: Option<DiffStatSnapshot>,
    cpu_pct: f32,
    mem_bytes: u64,
    processes: Vec<ProcessSnapshot>,
    agents: Vec<AgentSnapshot>,
    attention: bool,
    status: String,
    progress: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PullRequestSnapshot {
    number: u32,
    url: String,
    review_state: PullRequestReviewStateSnapshot,
    ci_state: PullRequestCiStateSnapshot,
    checks: Vec<PullRequestCheckSnapshot>,
    unresolved_comments: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum PullRequestReviewStateSnapshot {
    Draft,
    InReview,
    ChangesRequested,
    Approved,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum PullRequestCiStateSnapshot {
    Passing,
    Failing,
    RunningClean,
    RunningFailed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PullRequestCheckSnapshot {
    name: String,
    status: PullRequestCheckStatusSnapshot,
    elapsed_ms: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum PullRequestCheckStatusSnapshot {
    Running,
    Failing,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct DiffStatSnapshot {
    added: u32,
    removed: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AgentSnapshot {
    name: String,
    pane_id: String,
    gerund: Option<String>,
    ctx: Option<AgentCtxSnapshot>,
    age_ms: Option<u64>,
    asking: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProcessSnapshot {
    name: String,
    cpu_pct: f32,
    mem_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AgentCtxSnapshot {
    pct: u8,
    tokens: String,
}

#[derive(Clone, Debug, Serialize)]
struct DuplicateStringStat {
    value: String,
    count: usize,
    bytes_each: usize,
    duplicated_bytes: usize,
}

#[derive(Clone, Debug, Serialize)]
struct SnapshotStringStats {
    total_strings: usize,
    total_string_bytes: usize,
    unique_strings: usize,
    duplicated_occurrences: usize,
    duplicated_bytes: usize,
    top_duplicates: Vec<DuplicateStringStat>,
}

impl SidebarSnapshot {
    pub(super) fn age_ms(&self) -> u64 {
        now_ms().saturating_sub(self.generated_at_ms)
    }

    pub(super) fn meta_runtime(&self) -> HashMap<String, SessionMeta> {
        self.meta
            .iter()
            .map(|(session, meta)| (session.clone(), meta.runtime()))
            .collect()
    }

    pub(super) fn usage_lines(&self) -> Vec<String> {
        self.usage_lines.clone()
    }

    fn string_stats(&self) -> SnapshotStringStats {
        let mut counts = HashMap::<String, usize>::new();
        let mut total_strings = 0usize;
        let mut total_string_bytes = 0usize;

        let mut record = |value: &str| {
            total_strings += 1;
            total_string_bytes += value.len();
            *counts.entry(value.to_string()).or_default() += 1;
        };

        for session in &self.alive_sessions {
            record(session);
        }
        for (pane, session) in &self.pane_sessions {
            record(pane);
            record(session);
        }
        for (session, meta) in &self.meta {
            record(session);
            meta.record_strings(&mut record);
        }
        for line in &self.usage_lines {
            record(line);
        }

        let mut top_duplicates: Vec<_> = counts
            .iter()
            .filter_map(|(value, count)| {
                (*count > 1).then(|| DuplicateStringStat {
                    value: value.clone(),
                    count: *count,
                    bytes_each: value.len(),
                    duplicated_bytes: value.len() * (count - 1),
                })
            })
            .collect();
        top_duplicates.sort_by(|a, b| {
            b.duplicated_bytes
                .cmp(&a.duplicated_bytes)
                .then(b.count.cmp(&a.count))
                .then(a.value.cmp(&b.value))
        });
        let duplicated_occurrences = top_duplicates.iter().map(|stat| stat.count - 1).sum();
        let duplicated_bytes = top_duplicates
            .iter()
            .map(|stat| stat.duplicated_bytes)
            .sum();
        top_duplicates.truncate(20);

        SnapshotStringStats {
            total_strings,
            total_string_bytes,
            unique_strings: counts.len(),
            duplicated_occurrences,
            duplicated_bytes,
            top_duplicates,
        }
    }
}

impl SessionMetaSnapshot {
    fn record_strings(&self, record: &mut impl FnMut(&str)) {
        record(&self.branch);
        if let Some(pr) = &self.pr {
            pr.record_strings(record);
        }
        for process in &self.processes {
            record(&process.name);
        }
        for agent in &self.agents {
            record(&agent.name);
            record(&agent.pane_id);
            if let Some(gerund) = &agent.gerund {
                record(gerund);
            }
            if let Some(ctx) = &agent.ctx {
                record(&ctx.tokens);
            }
        }
        record(&self.status);
    }

    fn from_runtime(meta: &SessionMeta) -> Self {
        Self {
            branch: meta.branch.clone(),
            pr: meta.pr.as_ref().map(PullRequestSnapshot::from_runtime),
            diff: meta.diff.map(DiffStatSnapshot::from_runtime),
            cpu_pct: meta.cpu_pct,
            mem_bytes: meta.mem_bytes,
            processes: meta
                .processes
                .iter()
                .map(ProcessSnapshot::from_runtime)
                .collect(),
            agents: meta
                .agents
                .iter()
                .map(AgentSnapshot::from_runtime)
                .collect(),
            attention: meta.attention,
            status: meta.status.clone(),
            progress: meta.progress,
        }
    }

    fn runtime(&self) -> SessionMeta {
        SessionMeta {
            branch: self.branch.clone(),
            pr: self.pr.as_ref().map(PullRequestSnapshot::runtime),
            diff: self.diff.map(DiffStatSnapshot::runtime),
            cpu_pct: self.cpu_pct,
            mem_bytes: self.mem_bytes,
            processes: self
                .processes
                .iter()
                .map(ProcessSnapshot::runtime)
                .collect(),
            agents: self.agents.iter().map(AgentSnapshot::runtime).collect(),
            attention: self.attention,
            status: self.status.clone(),
            progress: self.progress,
        }
    }
}

impl DiffStatSnapshot {
    fn from_runtime(diff: DiffStat) -> Self {
        Self {
            added: diff.added,
            removed: diff.removed,
        }
    }

    fn runtime(self) -> DiffStat {
        DiffStat {
            added: self.added,
            removed: self.removed,
        }
    }
}

impl PullRequestSnapshot {
    fn record_strings(&self, record: &mut impl FnMut(&str)) {
        record(&self.url);
        for check in &self.checks {
            record(&check.name);
        }
    }

    fn from_runtime(pr: &PullRequestMeta) -> Self {
        Self {
            number: pr.number,
            url: pr.url.clone(),
            review_state: PullRequestReviewStateSnapshot::from_runtime(pr.review_state),
            ci_state: PullRequestCiStateSnapshot::from_runtime(pr.ci_state),
            checks: pr
                .checks
                .iter()
                .map(PullRequestCheckSnapshot::from_runtime)
                .collect(),
            unresolved_comments: pr.unresolved_comments,
        }
    }

    fn runtime(&self) -> PullRequestMeta {
        PullRequestMeta {
            number: self.number,
            url: self.url.clone(),
            review_state: self.review_state.runtime(),
            ci_state: self.ci_state.runtime(),
            checks: self
                .checks
                .iter()
                .map(PullRequestCheckSnapshot::runtime)
                .collect(),
            unresolved_comments: self.unresolved_comments,
        }
    }
}

impl PullRequestReviewStateSnapshot {
    fn from_runtime(state: PullRequestReviewState) -> Self {
        match state {
            PullRequestReviewState::Draft => Self::Draft,
            PullRequestReviewState::InReview => Self::InReview,
            PullRequestReviewState::ChangesRequested => Self::ChangesRequested,
            PullRequestReviewState::Approved => Self::Approved,
        }
    }

    fn runtime(self) -> PullRequestReviewState {
        match self {
            Self::Draft => PullRequestReviewState::Draft,
            Self::InReview => PullRequestReviewState::InReview,
            Self::ChangesRequested => PullRequestReviewState::ChangesRequested,
            Self::Approved => PullRequestReviewState::Approved,
        }
    }
}

impl PullRequestCiStateSnapshot {
    fn from_runtime(state: PullRequestCiState) -> Self {
        match state {
            PullRequestCiState::Passing => Self::Passing,
            PullRequestCiState::Failing => Self::Failing,
            PullRequestCiState::RunningClean => Self::RunningClean,
            PullRequestCiState::RunningFailed => Self::RunningFailed,
        }
    }

    fn runtime(self) -> PullRequestCiState {
        match self {
            Self::Passing => PullRequestCiState::Passing,
            Self::Failing => PullRequestCiState::Failing,
            Self::RunningClean => PullRequestCiState::RunningClean,
            Self::RunningFailed => PullRequestCiState::RunningFailed,
        }
    }
}

impl PullRequestCheckSnapshot {
    fn from_runtime(check: &PullRequestCheck) -> Self {
        Self {
            name: check.name.clone(),
            status: PullRequestCheckStatusSnapshot::from_runtime(check.status),
            elapsed_ms: check.elapsed.as_millis() as u64,
        }
    }

    fn runtime(&self) -> PullRequestCheck {
        PullRequestCheck {
            name: self.name.clone(),
            status: self.status.runtime(),
            elapsed: Duration::from_millis(self.elapsed_ms),
        }
    }
}

impl PullRequestCheckStatusSnapshot {
    fn from_runtime(status: PullRequestCheckStatus) -> Self {
        match status {
            PullRequestCheckStatus::Running => Self::Running,
            PullRequestCheckStatus::Failing => Self::Failing,
        }
    }

    fn runtime(self) -> PullRequestCheckStatus {
        match self {
            Self::Running => PullRequestCheckStatus::Running,
            Self::Failing => PullRequestCheckStatus::Failing,
        }
    }
}

impl ProcessSnapshot {
    fn from_runtime(process: &ProcessTreeInfo) -> Self {
        Self {
            name: process.name.clone(),
            cpu_pct: process.cpu_pct,
            mem_bytes: process.mem_bytes,
        }
    }

    fn runtime(&self) -> ProcessTreeInfo {
        ProcessTreeInfo {
            name: self.name.clone(),
            cpu_pct: self.cpu_pct,
            mem_bytes: self.mem_bytes,
        }
    }
}

impl AgentSnapshot {
    fn from_runtime(agent: &AgentInstance) -> Self {
        Self {
            name: agent.name.clone(),
            pane_id: agent.pane_id.clone(),
            gerund: agent.gerund.clone(),
            ctx: agent.ctx.as_ref().map(AgentCtxSnapshot::from_runtime),
            age_ms: agent.age.map(|age| age.as_millis() as u64),
            asking: agent.asking,
        }
    }

    fn runtime(&self) -> AgentInstance {
        AgentInstance {
            name: self.name.clone(),
            pane_id: self.pane_id.clone(),
            gerund: self.gerund.clone(),
            ctx: self.ctx.as_ref().map(AgentCtxSnapshot::runtime),
            age: self.age_ms.map(Duration::from_millis),
            asking: self.asking,
        }
    }
}

impl AgentCtxSnapshot {
    fn from_runtime(ctx: &AgentCtx) -> Self {
        Self {
            pct: ctx.pct,
            tokens: ctx.tokens.clone(),
        }
    }

    fn runtime(&self) -> AgentCtx {
        AgentCtx {
            pct: self.pct,
            tokens: self.tokens.clone(),
        }
    }
}

struct DaemonCache {
    meta: HashMap<String, SessionMeta>,
    gerund_cache: HashMap<String, (String, Instant)>,
    last_active: HashMap<String, Instant>,
    last_meta_refresh: Instant,
    usage_lines: Vec<String>,
    counters: SidebarCounters,
}

struct BaseSnapshotInput {
    notched: bool,
    alive_sessions: Vec<String>,
    quick_meta: HashMap<String, SessionQuickMeta>,
    pane_sessions: HashMap<String, String>,
    sidebar_panes: usize,
}

struct SessionQuickMeta {
    attention: bool,
    status: String,
    progress: Option<u8>,
}

impl DaemonCache {
    fn new() -> Self {
        Self {
            meta: HashMap::new(),
            gerund_cache: HashMap::new(),
            last_active: HashMap::new(),
            last_meta_refresh: Instant::now() - Duration::from_secs(60),
            usage_lines: Vec::new(),
            counters: SidebarCounters::default(),
        }
    }

    fn snapshot(&mut self) -> Option<(SidebarSnapshot, usize)> {
        let base = query_base_snapshot()?;
        let alive_set: HashSet<String> = base.alive_sessions.iter().cloned().collect();
        self.prune_dead_sessions(&alive_set);
        let sessions = compute_order(&alive_set, true);

        let meta_due = self.last_meta_refresh.elapsed() >= META_INTERVAL;
        self.counters.record_daemon_tick(meta_due);
        if meta_due {
            self.refresh_meta(&sessions);
        }
        self.apply_quick_meta(&base.quick_meta);

        let snapshot = SidebarSnapshot {
            version: SNAPSHOT_VERSION,
            generated_at_ms: now_ms(),
            notched: base.notched,
            alive_sessions: base.alive_sessions,
            pane_sessions: base.pane_sessions,
            meta: self
                .meta
                .iter()
                .map(|(session, meta)| (session.clone(), SessionMetaSnapshot::from_runtime(meta)))
                .collect(),
            usage_lines: self.usage_lines.clone(),
        };
        Some((snapshot, base.sidebar_panes))
    }

    fn apply_quick_meta(&mut self, quick: &HashMap<String, SessionQuickMeta>) {
        for (session, quick_meta) in quick {
            let meta = self.meta.entry(session.clone()).or_default();
            meta.attention = quick_meta.attention;
            meta.status = quick_meta.status.clone();
            meta.progress = quick_meta.progress;
        }
    }

    fn prune_dead_sessions(&mut self, alive: &HashSet<String>) {
        self.meta.retain(|session, _| alive.contains(session));
        self.gerund_cache
            .retain(|cache_key, _| cache_key_session_alive(cache_key, alive));
        self.last_active
            .retain(|cache_key, _| cache_key_session_alive(cache_key, alive));
    }

    fn refresh_meta(&mut self, sessions: &[String]) {
        let (mut meta, tmux_calls) = query_session_meta(sessions);
        let now = Instant::now();
        for (session, m) in meta.iter_mut() {
            for agent in m.agents.iter_mut() {
                let cache_key = format!("{}:{}", session, agent.pane_id);
                let raw_active = agent.gerund.is_some();
                if raw_active {
                    self.last_active.insert(cache_key.clone(), now);
                }
                match &agent.gerund {
                    Some(g) => {
                        self.gerund_cache
                            .insert(cache_key.clone(), (g.clone(), now));
                    }
                    None => {
                        if let Some((cached, t)) = self.gerund_cache.get(&cache_key)
                            && now.duration_since(*t) < ACTIVITY_GRACE
                        {
                            agent.gerund = Some(cached.clone());
                        }
                    }
                }
                if let Some(&t) = self.last_active.get(&cache_key) {
                    let from_cache = now.duration_since(t);
                    agent.age = Some(match agent.age {
                        Some(existing) if existing < from_cache => existing,
                        _ => from_cache,
                    });
                }
            }
        }
        self.gerund_cache
            .retain(|_, (_, t)| now.duration_since(*t) < ACTIVITY_GRACE);
        self.meta = meta;
        self.last_meta_refresh = now;

        self.usage_lines = usage_bars::collect(usage_width()).lines;

        debug!(
            tmux_calls,
            session_count = sessions.len(),
            "sidebar daemon meta refresh"
        );
    }
}

fn cache_key_session_alive(cache_key: &str, alive: &HashSet<String>) -> bool {
    cache_key
        .rsplit_once(':')
        .is_some_and(|(session, _)| alive.contains(session))
}

pub(super) fn ensure_started() {
    if let Some(pid) = daemon_pid()
        && process_alive(pid)
    {
        if snapshot_version_current() {
            return;
        }
        let _ = Command::new("kill")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = fs::remove_file(pid_path());
    }

    if let Some(pid) = daemon_pid()
        && process_alive(pid)
    {
        return;
    }

    let exe = std::env::current_exe().unwrap_or_else(|_| "mux".into());
    let mut command = Command::new(exe);
    command
        .arg("sidebar-daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _ = spawn_reaped(command);
}

pub(crate) fn cmd_sidebar_daemon() {
    if !claim_daemon_pid() {
        return;
    }

    let mut cache = DaemonCache::new();
    let mut idle_for = Duration::ZERO;

    loop {
        let started = Instant::now();
        match cache.snapshot() {
            Some((snapshot, sidebar_panes)) => {
                if let Err(e) = write_snapshot(&snapshot) {
                    error!(error = %e, "failed to write sidebar daemon snapshot");
                }
                if sidebar_panes == 0 {
                    idle_for += TICK;
                    if idle_for >= IDLE_EXIT_AFTER {
                        break;
                    }
                } else {
                    idle_for = Duration::ZERO;
                }
            }
            None => idle_for += TICK,
        }

        let elapsed = started.elapsed();
        if elapsed < TICK {
            std::thread::sleep(TICK - elapsed);
        }
    }

    let _ = fs::remove_file(pid_path());
}

pub(super) fn cmd_status_latency_profile(args: &[String]) {
    let iterations = args
        .first()
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(8);
    let target_p95_ms = args
        .get(1)
        .and_then(|arg| arg.parse::<u64>().ok())
        .unwrap_or(750);
    let max_wait = Duration::from_millis(
        args.get(2)
            .and_then(|arg| arg.parse::<u64>().ok())
            .unwrap_or(2_000),
    );
    let poll_interval = Duration::from_millis(10);

    ensure_started();

    let Some(session) = first_session_name() else {
        eprintln!("status latency profile requires at least one tmux session");
        std::process::exit(2);
    };

    let mut samples = Vec::with_capacity(iterations);
    let mut timeouts = 0usize;

    for iteration in 0..iterations {
        let status = format!("latency-{iteration}-{}", now_ms());
        let progress = ((iteration * 17) % 100) as u8;
        let attention = iteration % 2 == 0;
        let progress_text = progress.to_string();
        let attention_text = if attention { "1" } else { "0" };

        tmux(&[
            "set-option",
            "-t",
            &session,
            "-q",
            "@sidebar_status",
            &status,
            ";",
            "set-option",
            "-t",
            &session,
            "-q",
            "@sidebar_progress",
            &progress_text,
            ";",
            "set-option",
            "-t",
            &session,
            "-q",
            "@attention",
            attention_text,
        ]);

        let started = Instant::now();
        loop {
            if snapshot_has_status(&session, &status, progress, attention) {
                samples.push(started.elapsed().as_millis() as u64);
                break;
            }
            if started.elapsed() >= max_wait {
                timeouts += 1;
                samples.push(max_wait.as_millis() as u64);
                break;
            }
            std::thread::sleep(poll_interval);
        }
    }

    let summary = LatencySummary::from_samples(samples, target_p95_ms);
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "session": session,
            "iterations": iterations,
            "timeouts": timeouts,
            "poll_interval_ms": poll_interval.as_millis() as u64,
            "max_wait_ms": max_wait.as_millis() as u64,
            "summary": summary,
        }))
        .expect("serialize sidebar status latency profile")
    );

    if timeouts > 0 || !summary.passed {
        std::process::exit(1);
    }
}

pub(super) fn cmd_snapshot_string_stats(args: &[String]) {
    let path = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(snapshot_path);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("failed to read snapshot {}: {e}", path.display());
            std::process::exit(2);
        }
    };
    let Some(snapshot) = decode_snapshot_bytes(&bytes) else {
        eprintln!("failed to decode snapshot {}", path.display());
        std::process::exit(2);
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "path": path,
            "snapshot_bytes": bytes.len(),
            "stats": snapshot.string_stats(),
        }))
        .expect("serialize snapshot string stats")
    );
}

pub(super) fn load_snapshot() -> Option<SidebarSnapshot> {
    let contents = fs::read(snapshot_path()).ok()?;
    let snapshot: SidebarSnapshot = decode_snapshot_bytes(&contents)?;
    if snapshot.version != SNAPSHOT_VERSION || snapshot.age_ms() > SNAPSHOT_STALE.as_millis() as u64
    {
        return None;
    }
    Some(snapshot)
}

fn first_session_name() -> Option<String> {
    tmux(&["list-sessions", "-F", "#{session_name}"])
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

fn snapshot_has_status(session: &str, status: &str, progress: u8, attention: bool) -> bool {
    load_snapshot()
        .and_then(|snapshot| {
            snapshot
                .meta
                .iter()
                .find(|(candidate, _)| candidate == session)
                .map(|(_, meta)| meta.clone())
        })
        .is_some_and(|meta| {
            meta.status == status && meta.progress == Some(progress) && meta.attention == attention
        })
}

fn query_base_snapshot() -> Option<BaseSnapshotInput> {
    const DELIM: &str = "\x1e<<MUX_SIDEBAR_DAEMON_DELIM>>\x1e";
    let raw = tmux(&[
        "show-option",
        "-gv",
        "@notched",
        ";",
        "display-message",
        "-p",
        DELIM,
        ";",
        "list-sessions",
        "-F",
        "#{session_name}\t#{@attention}\t#{@sidebar_status}\t#{@sidebar_progress}",
        ";",
        "display-message",
        "-p",
        DELIM,
        ";",
        "list-panes",
        "-a",
        "-F",
        "#{pane_id}\t#{session_name}\t#{@mux_sidebar}\t#{@mux_sidebar_token}\t#{pane_current_command}",
    ]);
    if raw.is_empty() {
        return None;
    }

    let mut sections = raw.split(DELIM);
    let notched = sections
        .next()
        .and_then(|s| s.lines().next())
        .unwrap_or("")
        .trim()
        == "1";
    let mut alive_sessions = Vec::new();
    let mut quick_meta = HashMap::new();
    for line in sections.next().unwrap_or_default().lines() {
        let mut parts = line.split('\t');
        let session = parts.next().unwrap_or_default().trim();
        if session.is_empty() {
            continue;
        }
        let attention = parts.next().unwrap_or_default() == "1";
        let status = parts.next().unwrap_or_default().to_string();
        let progress = parts.next().and_then(|s| s.parse::<u8>().ok());
        alive_sessions.push(session.to_string());
        quick_meta.insert(
            session.to_string(),
            SessionQuickMeta {
                attention,
                status,
                progress,
            },
        );
    }

    let mut pane_sessions = HashMap::new();
    let mut sidebar_panes = 0usize;
    for line in sections.next().unwrap_or_default().lines() {
        let mut parts = line.split('\t');
        let pane = parts.next().unwrap_or_default();
        let session = parts.next().unwrap_or_default();
        let marker = parts.next().unwrap_or_default();
        let token = parts.next().unwrap_or_default();
        let command = parts.next().unwrap_or_default();
        if !pane.is_empty() && !session.is_empty() {
            pane_sessions.insert(pane.to_string(), session.to_string());
        }
        if marker == "1" && token == SIDEBAR_TOKEN && command == "mux" {
            sidebar_panes += 1;
        }
    }

    Some(BaseSnapshotInput {
        notched,
        alive_sessions,
        quick_meta,
        pane_sessions,
        sidebar_panes,
    })
}

fn write_snapshot(snapshot: &SidebarSnapshot) -> std::io::Result<()> {
    fs::create_dir_all(state_dir())?;
    let path = snapshot_path();
    let tmp = path.with_extension("msgpack.tmp");
    let data = encode_snapshot_bytes(snapshot)?;
    fs::write(&tmp, data)?;
    fs::rename(tmp, path)
}

pub(super) fn snapshot_from_parts_for_bench(
    generated_at_ms: u64,
    notched: bool,
    alive_sessions: Vec<String>,
    pane_sessions: HashMap<String, String>,
    meta: HashMap<String, SessionMeta>,
    usage_lines: Vec<String>,
) -> SidebarSnapshot {
    SidebarSnapshot {
        version: SNAPSHOT_VERSION,
        generated_at_ms,
        notched,
        alive_sessions,
        pane_sessions,
        meta: meta
            .iter()
            .map(|(session, meta)| (session.clone(), SessionMetaSnapshot::from_runtime(meta)))
            .collect(),
        usage_lines,
    }
}

pub(super) fn snapshot_json_for_bench(snapshot: &SidebarSnapshot) -> Vec<u8> {
    serde_json::to_vec(snapshot).expect("serialize sidebar benchmark snapshot")
}

pub(super) fn decode_snapshot_for_bench(bytes: &[u8]) -> Option<SidebarSnapshot> {
    decode_snapshot_bytes(bytes)
}

pub(super) fn decode_snapshot_via_utf8_string_for_bench(bytes: &[u8]) -> Option<SidebarSnapshot> {
    let contents = String::from_utf8(bytes.to_vec()).ok()?;
    serde_json::from_str(&contents).ok()
}

pub(super) fn snapshot_bytes_for_bench(snapshot: &SidebarSnapshot) -> Vec<u8> {
    encode_snapshot_bytes(snapshot).expect("serialize sidebar benchmark snapshot")
}

fn encode_snapshot_bytes(snapshot: &SidebarSnapshot) -> io::Result<Vec<u8>> {
    rmp_serde::to_vec(snapshot).map_err(io::Error::other)
}

fn decode_snapshot_bytes(bytes: &[u8]) -> Option<SidebarSnapshot> {
    rmp_serde::from_slice(bytes).ok()
}

fn pid_alive() -> bool {
    daemon_pid().is_some_and(process_alive)
}

fn daemon_pid() -> Option<u32> {
    let Ok(contents) = fs::read_to_string(pid_path()) else {
        return None;
    };
    contents.trim().parse::<u32>().ok()
}

fn process_alive(pid: u32) -> bool {
    // Avoid spawning `/bin/kill` on every sidebar cold start. `kill(pid, 0)` is
    // the POSIX liveness probe used by the shell command but stays in-process.
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

fn snapshot_version_current() -> bool {
    fs::read(snapshot_path())
        .ok()
        .and_then(|contents| decode_snapshot_bytes(&contents))
        .is_some_and(|snapshot| snapshot.version == SNAPSHOT_VERSION)
}

fn claim_daemon_pid() -> bool {
    if let Err(e) = fs::create_dir_all(state_dir()) {
        error!(error = %e, "failed to create sidebar daemon state dir");
        return false;
    }

    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(pid_path())
        {
            Ok(mut file) => {
                if let Err(e) = write!(file, "{}", std::process::id()) {
                    error!(error = %e, "failed to write sidebar daemon pid");
                    let _ = fs::remove_file(pid_path());
                    return false;
                }
                return true;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                if pid_alive() {
                    return false;
                }
                let _ = fs::remove_file(pid_path());
            }
            Err(e) => {
                error!(error = %e, "failed to claim sidebar daemon pid");
                return false;
            }
        }
    }
}

fn usage_width() -> u16 {
    super::sidebar_width().parse::<u16>().unwrap_or(36)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn state_dir() -> PathBuf {
    home().join(".local/state/mux/sidebar")
}

fn snapshot_path() -> PathBuf {
    state_dir().join("snapshot.msgpack")
}

fn pid_path() -> PathBuf {
    state_dir().join("daemon.pid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_string_stats_counts_duplicate_session_names() {
        let snapshot = SidebarSnapshot {
            version: SNAPSHOT_VERSION,
            generated_at_ms: 0,
            notched: false,
            alive_sessions: vec!["work".to_string()],
            pane_sessions: [
                ("%1".to_string(), "work".to_string()),
                ("%2".to_string(), "work".to_string()),
            ]
            .into(),
            meta: vec![(
                "work".to_string(),
                SessionMetaSnapshot::from_runtime(&SessionMeta {
                    branch: "main".to_string(),
                    status: "working".to_string(),
                    ..SessionMeta::default()
                }),
            )],
            usage_lines: vec![],
        };

        let stats = snapshot.string_stats();

        assert!(stats.duplicated_bytes >= "work".len() * 3);
        assert!(
            stats
                .top_duplicates
                .iter()
                .any(|duplicate| duplicate.value == "work" && duplicate.count == 4)
        );
    }

    #[test]
    fn daemon_cache_prunes_dead_session_state() {
        let mut cache = DaemonCache::new();
        cache
            .meta
            .insert("alive".to_string(), SessionMeta::default());
        cache
            .meta
            .insert("dead".to_string(), SessionMeta::default());
        cache.gerund_cache.insert(
            "alive:%1".to_string(),
            ("Running…".to_string(), Instant::now()),
        );
        cache.gerund_cache.insert(
            "dead:%2".to_string(),
            ("Running…".to_string(), Instant::now()),
        );
        cache
            .last_active
            .insert("alive:%1".to_string(), Instant::now());
        cache
            .last_active
            .insert("dead:%2".to_string(), Instant::now());

        cache.prune_dead_sessions(&["alive".to_string()].into());

        assert!(cache.meta.contains_key("alive"));
        assert!(!cache.meta.contains_key("dead"));
        assert!(cache.gerund_cache.contains_key("alive:%1"));
        assert!(!cache.gerund_cache.contains_key("dead:%2"));
        assert!(cache.last_active.contains_key("alive:%1"));
        assert!(!cache.last_active.contains_key("dead:%2"));
    }
}
