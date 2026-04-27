use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::order::compute_order;
use crate::tmux::{home, tmux};
use crate::usage_bars;

use super::ACTIVITY_GRACE;
use super::claude::AgentCtx;
use super::meta::{AgentInstance, SessionMeta, query_session_meta};

const SNAPSHOT_VERSION: u32 = 1;
const SNAPSHOT_STALE: Duration = Duration::from_secs(5);
const TICK: Duration = Duration::from_millis(500);
const META_INTERVAL: Duration = Duration::from_secs(3);
const IDLE_EXIT_AFTER: Duration = Duration::from_secs(30);
const SIDEBAR_TOKEN: &str = super::SIDEBAR_TOKEN;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct SidebarSnapshot {
    version: u32,
    generated_at_ms: u64,
    pub(super) notched: bool,
    pub(super) alive_sessions: Vec<String>,
    pub(super) pane_sessions: HashMap<String, String>,
    meta: HashMap<String, SessionMetaSnapshot>,
    usage_bars: Vec<UsageBarSnapshot>,
    overage: Option<ClaudeOverageSnapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SessionMetaSnapshot {
    branch: String,
    agents: Vec<AgentSnapshot>,
    attention: bool,
    ports: Vec<u16>,
    status: String,
    progress: Option<u8>,
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
struct AgentCtxSnapshot {
    pct: u8,
    tokens: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UsageBarSnapshot {
    label: String,
    display: String,
    pct: f64,
    window_secs: i64,
    reset_ts: i64,
    last_ts: i64,
    provider_rgb: (u8, u8, u8),
    overage: Option<f64>,
    hit_rate: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ClaudeOverageSnapshot {
    five_h: f64,
    seven_d: f64,
    month: f64,
    total: f64,
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

    pub(super) fn usage_bars_runtime(&self) -> Vec<usage_bars::Bar> {
        self.usage_bars
            .iter()
            .map(UsageBarSnapshot::runtime)
            .collect()
    }

    pub(super) fn overage_runtime(&self) -> Option<usage_bars::ClaudeOverage> {
        self.overage.as_ref().map(ClaudeOverageSnapshot::runtime)
    }
}

impl SessionMetaSnapshot {
    fn from_runtime(meta: &SessionMeta) -> Self {
        Self {
            branch: meta.branch.clone(),
            agents: meta
                .agents
                .iter()
                .map(AgentSnapshot::from_runtime)
                .collect(),
            attention: meta.attention,
            ports: meta.ports.clone(),
            status: meta.status.clone(),
            progress: meta.progress,
        }
    }

    fn runtime(&self) -> SessionMeta {
        SessionMeta {
            branch: self.branch.clone(),
            agents: self.agents.iter().map(AgentSnapshot::runtime).collect(),
            attention: self.attention,
            ports: self.ports.clone(),
            status: self.status.clone(),
            progress: self.progress,
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

impl UsageBarSnapshot {
    fn from_runtime(bar: &usage_bars::Bar) -> Self {
        Self {
            label: bar.label.clone(),
            display: bar.display.clone(),
            pct: bar.pct,
            window_secs: bar.window_secs,
            reset_ts: bar.reset_ts,
            last_ts: bar.last_ts,
            provider_rgb: color_rgb(bar.provider),
            overage: bar.overage,
            hit_rate: bar.hit_rate,
        }
    }

    fn runtime(&self) -> usage_bars::Bar {
        usage_bars::Bar {
            label: self.label.clone(),
            display: self.display.clone(),
            pct: self.pct,
            window_secs: self.window_secs,
            reset_ts: self.reset_ts,
            last_ts: self.last_ts,
            provider: Color::Rgb(
                self.provider_rgb.0,
                self.provider_rgb.1,
                self.provider_rgb.2,
            ),
            overage: self.overage,
            hit_rate: self.hit_rate,
        }
    }
}

impl ClaudeOverageSnapshot {
    fn from_runtime(overage: &usage_bars::ClaudeOverage) -> Self {
        Self {
            five_h: overage.five_h,
            seven_d: overage.seven_d,
            month: overage.month,
            total: overage.total,
        }
    }

    fn runtime(&self) -> usage_bars::ClaudeOverage {
        usage_bars::ClaudeOverage {
            five_h: self.five_h,
            seven_d: self.seven_d,
            month: self.month,
            total: self.total,
        }
    }
}

struct DaemonCache {
    meta: HashMap<String, SessionMeta>,
    gerund_cache: HashMap<String, (String, Instant)>,
    last_active: HashMap<String, Instant>,
    last_meta_refresh: Instant,
    usage_bars: Vec<usage_bars::Bar>,
    overage: Option<usage_bars::ClaudeOverage>,
}

struct BaseSnapshotInput {
    notched: bool,
    alive_sessions: Vec<String>,
    pane_sessions: HashMap<String, String>,
    sidebar_panes: usize,
}

impl DaemonCache {
    fn new() -> Self {
        Self {
            meta: HashMap::new(),
            gerund_cache: HashMap::new(),
            last_active: HashMap::new(),
            last_meta_refresh: Instant::now() - Duration::from_secs(60),
            usage_bars: Vec::new(),
            overage: None,
        }
    }

    fn snapshot(&mut self) -> Option<(SidebarSnapshot, usize)> {
        let base = query_base_snapshot()?;
        let alive_set = base.alive_sessions.iter().cloned().collect();
        let sessions = compute_order(&alive_set, true);

        if self.last_meta_refresh.elapsed() >= META_INTERVAL {
            self.refresh_meta(&sessions);
        }

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
            usage_bars: self
                .usage_bars
                .iter()
                .map(UsageBarSnapshot::from_runtime)
                .collect(),
            overage: self
                .overage
                .as_ref()
                .map(ClaudeOverageSnapshot::from_runtime),
        };
        Some((snapshot, base.sidebar_panes))
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

        let snap = usage_bars::collect();
        self.usage_bars = snap.bars;
        self.overage = snap.overage;

        debug!(
            tmux_calls,
            session_count = sessions.len(),
            "sidebar daemon meta refresh"
        );
    }
}

pub(super) fn ensure_started() {
    if pid_alive() {
        return;
    }

    let exe = std::env::current_exe().unwrap_or_else(|_| "mux".into());
    let _ = Command::new(exe)
        .arg("sidebar-daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
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

pub(super) fn load_snapshot() -> Option<SidebarSnapshot> {
    let contents = fs::read_to_string(snapshot_path()).ok()?;
    let snapshot: SidebarSnapshot = serde_json::from_str(&contents).ok()?;
    if snapshot.version != SNAPSHOT_VERSION || snapshot.age_ms() > SNAPSHOT_STALE.as_millis() as u64
    {
        return None;
    }
    Some(snapshot)
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
        "#S",
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
    let alive_sessions = sections
        .next()
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

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
        pane_sessions,
        sidebar_panes,
    })
}

fn write_snapshot(snapshot: &SidebarSnapshot) -> std::io::Result<()> {
    fs::create_dir_all(state_dir())?;
    let path = snapshot_path();
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_vec(snapshot)?;
    fs::write(&tmp, data)?;
    fs::rename(tmp, path)
}

fn pid_alive() -> bool {
    let Ok(contents) = fs::read_to_string(pid_path()) else {
        return false;
    };
    let Ok(pid) = contents.trim().parse::<u32>() else {
        return false;
    };
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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

fn color_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Yellow => (255, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Magenta => (255, 0, 255),
        Color::Cyan => (0, 255, 255),
        Color::Gray | Color::DarkGray => (128, 128, 128),
        Color::LightRed => (255, 128, 128),
        Color::LightGreen => (128, 255, 128),
        Color::LightYellow => (255, 255, 128),
        Color::LightBlue => (128, 128, 255),
        Color::LightMagenta => (255, 128, 255),
        Color::LightCyan => (128, 255, 255),
        Color::Indexed(_) | Color::Reset => (255, 255, 255),
    }
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
    state_dir().join("snapshot.json")
}

fn pid_path() -> PathBuf {
    state_dir().join("daemon.pid")
}
