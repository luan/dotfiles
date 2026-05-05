use std::time::Duration;

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoopState {
    VisibleIdle,
    HiddenIdle,
    ActiveAnimation,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SidebarCounters {
    pub redraws: u64,
    pub refreshes: u64,
    pub tmux_spawns: u64,
    pub snapshot_load_hits: u64,
    pub snapshot_load_misses: u64,
    pub visible_polls: u64,
    pub hidden_polls: u64,
    pub animation_frames: u64,
    pub daemon_ticks: u64,
    pub daemon_meta_refreshes: u64,
    pub meta_refresh_interval_ms: u64,
}

impl SidebarCounters {
    pub fn record_redraw(&mut self, animation_active: bool) {
        self.redraws = self.redraws.saturating_add(1);
        if animation_active {
            self.animation_frames = self.animation_frames.saturating_add(1);
        }
    }

    pub fn record_refresh(&mut self, tmux_spawns: u64, snapshot_hit: bool) {
        self.refreshes = self.refreshes.saturating_add(1);
        self.record_tmux_spawns(tmux_spawns);
        if snapshot_hit {
            self.snapshot_load_hits = self.snapshot_load_hits.saturating_add(1);
        } else {
            self.snapshot_load_misses = self.snapshot_load_misses.saturating_add(1);
        }
    }

    pub fn record_tmux_spawns(&mut self, count: u64) {
        self.tmux_spawns = self.tmux_spawns.saturating_add(count);
    }

    pub fn record_poll(&mut self, state: LoopState) {
        match state {
            LoopState::VisibleIdle | LoopState::ActiveAnimation => {
                self.visible_polls = self.visible_polls.saturating_add(1);
            }
            LoopState::HiddenIdle => {
                self.hidden_polls = self.hidden_polls.saturating_add(1);
            }
        }
    }

    pub fn record_daemon_tick(&mut self, meta_refreshed: bool) {
        self.daemon_ticks = self.daemon_ticks.saturating_add(1);
        if meta_refreshed {
            self.daemon_meta_refreshes = self.daemon_meta_refreshes.saturating_add(1);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HiddenLifecycleAction {
    Run,
    Park,
    Exit,
}

#[derive(Clone, Debug)]
pub struct HiddenLifecycle {
    first_hidden_at: Option<Duration>,
    exit_after: Duration,
}

impl Default for HiddenLifecycle {
    fn default() -> Self {
        Self {
            first_hidden_at: None,
            exit_after: Duration::from_secs(2),
        }
    }
}

impl HiddenLifecycle {
    pub fn observe(&mut self, on_screen: bool, now: Duration) -> HiddenLifecycleAction {
        if on_screen {
            self.first_hidden_at = None;
            return HiddenLifecycleAction::Run;
        }

        let first_hidden_at = *self.first_hidden_at.get_or_insert(now);
        if now.saturating_sub(first_hidden_at) >= self.exit_after {
            HiddenLifecycleAction::Exit
        } else {
            HiddenLifecycleAction::Park
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SyntheticProfile {
    pub state: LoopState,
    pub duration_ms: u64,
    pub counters: SidebarCounters,
}

#[derive(Clone, Debug, Serialize)]
pub struct LatencySummary {
    pub samples: usize,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub max_ms: u64,
    pub target_p95_ms: u64,
    pub passed: bool,
}

impl LatencySummary {
    pub fn from_samples(mut samples: Vec<u64>, target_p95_ms: u64) -> Self {
        samples.sort_unstable();
        let sample_count = samples.len();
        let p50_ms = percentile(&samples, 50);
        let p95_ms = percentile(&samples, 95);
        let max_ms = samples.last().copied().unwrap_or_default();

        Self {
            samples: sample_count,
            p50_ms,
            p95_ms,
            max_ms,
            target_p95_ms,
            passed: p95_ms <= target_p95_ms,
        }
    }
}

fn percentile(samples: &[u64], percentile: usize) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let rank = ((samples.len() * percentile).div_ceil(100)).saturating_sub(1);
    samples[rank.min(samples.len() - 1)]
}

pub fn synthetic_profiles(duration: Duration) -> Vec<SyntheticProfile> {
    vec![
        synthetic_profile(LoopState::VisibleIdle, duration),
        synthetic_profile(LoopState::HiddenIdle, duration),
        synthetic_profile(LoopState::ActiveAnimation, duration),
    ]
}

pub fn daemon_profile(duration: Duration) -> SidebarCounters {
    let duration_ms = duration.as_millis() as u64;
    SidebarCounters {
        daemon_ticks: duration_ms / 500,
        daemon_meta_refreshes: duration_ms / 5_000,
        meta_refresh_interval_ms: 5_000,
        ..SidebarCounters::default()
    }
}

pub(crate) fn cmd_profile(args: &[String]) {
    let duration_ms = args
        .first()
        .and_then(|arg| arg.parse::<u64>().ok())
        .unwrap_or(2000);
    let duration = Duration::from_millis(duration_ms);
    let profiles = args
        .get(1)
        .and_then(|state| parse_loop_state(state))
        .map(|state| vec![synthetic_profile(state, duration)])
        .unwrap_or_else(|| synthetic_profiles(duration));
    let daemon = daemon_profile(duration);
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "sidebar": profiles,
            "daemon": daemon,
        }))
        .expect("serialize sidebar profile")
    );
}

fn parse_loop_state(state: &str) -> Option<LoopState> {
    match state {
        "visible-idle" => Some(LoopState::VisibleIdle),
        "hidden-idle" | "hidden-offscreen" | "hidden-off-screen" => Some(LoopState::HiddenIdle),
        "active-animation" => Some(LoopState::ActiveAnimation),
        _ => None,
    }
}

fn synthetic_profile(state: LoopState, duration: Duration) -> SyntheticProfile {
    let duration_ms = duration.as_millis() as u64;
    let mut counters = SidebarCounters::default();

    match state {
        LoopState::VisibleIdle => {
            counters.redraws = u64::from(duration_ms > 0);
            counters.refreshes = duration_ms / 500;
            counters.snapshot_load_hits = counters.refreshes;
            counters.visible_polls = duration_ms / 500;
        }
        LoopState::HiddenIdle => {
            if duration_ms > 0 {
                counters.hidden_polls = 1;
                counters.tmux_spawns = 1;
            }
        }
        LoopState::ActiveAnimation => {
            counters.redraws = duration_ms / 33;
            counters.animation_frames = counters.redraws;
            counters.refreshes = duration_ms / 500;
            counters.snapshot_load_hits = counters.refreshes;
            counters.visible_polls = counters.redraws;
        }
    }

    SyntheticProfile {
        state,
        duration_ms,
        counters,
    }
}
