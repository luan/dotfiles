use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::cursor;
use crossterm::event::{
    self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
    KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;

use crate::order::compute_order;
use crate::process::spawn_reaped;
use crate::tmux::tmux;
use tracing::debug;

use crate::usage_bars;

mod claude;
mod daemon;
pub(crate) mod meta;
mod overlay;
mod pi;
mod render;
mod tree;

use meta::{SessionMeta, query_session_meta};
use overlay::{SidebarOverlay, handle_readline_key};
use render::draw;
use tree::{Item, ItemKind, build_items};

// Nerd Font keyboard modifier glyphs (md-apple-keyboard-* + md-keyboard-tab).
// These render at proper size/weight where the bare Unicode symbols (⌘⌃⌥⇧⇥)
// fall back to a non-keyboard font and come out tiny or wrong.
pub(super) const KEY_CMD: &str = "\u{F0633}";
pub(super) const KEY_CTRL: &str = "\u{F0634}";
pub(super) const KEY_OPT: &str = "\u{F0635}";
pub(super) const KEY_SHIFT: &str = "\u{F0636}";
pub(super) const KEY_TAB: &str = "\u{F0312}";
const SIDEBAR_WIDTH_DEFAULT: &str = "45";
const SIDEBAR_TOKEN: &str = "mux-sidebar-v1";
const SIDEBAR_BORDER_COLOR: &str = "#1A1B26";
const SIDEBAR_OPEN_LOCK: &str = "mux-sidebar-open.lock";
const SIDEBAR_OPEN_LOCK_STALE: Duration = Duration::from_secs(30);
const SIDEBAR_OPEN_LOCK_TIMEOUT: Duration = Duration::from_secs(3);

struct SidebarOpenLock {
    path: std::path::PathBuf,
}

impl Drop for SidebarOpenLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn acquire_sidebar_open_lock() -> Option<SidebarOpenLock> {
    let path = std::env::temp_dir().join(SIDEBAR_OPEN_LOCK);
    let start = Instant::now();

    loop {
        match fs::create_dir(&path) {
            Ok(()) => return Some(SidebarOpenLock { path }),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                let stale = fs::metadata(&path)
                    .and_then(|meta| meta.modified())
                    .ok()
                    .and_then(|modified| modified.elapsed().ok())
                    .is_some_and(|age| age > SIDEBAR_OPEN_LOCK_STALE);

                if stale {
                    let _ = fs::remove_dir(&path);
                    continue;
                }

                if start.elapsed() > SIDEBAR_OPEN_LOCK_TIMEOUT {
                    return None;
                }

                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => return None,
        }
    }
}

pub(super) fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else if max == 0 {
        String::new()
    } else {
        let cut: String = chars.iter().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SidebarMode {
    Browse,
    Chooser,
}

fn handoff_to_main(state: &mut SidebarState) {
    state.overlay = None;
    state.last_meta_refresh = Instant::now() - Duration::from_secs(60);
    state.focused = false;
    state.hover = None;
    state.refresh();
    focus_main_pane();
}

// ── State ────────────────────────────────────────────────────

struct SidebarState {
    pub(super) items: Vec<Item>,
    pub(super) visible: Vec<usize>,
    pub(super) current: String,
    pub(super) selected: usize,
    pub(super) offset: usize,
    pub(super) hover: Option<String>,
    pub(super) meta: HashMap<String, SessionMeta>,
    /// Sticky cache for gerund text so it doesn't flicker between refreshes.
    /// Cleared when the entry is older than ACTIVITY_GRACE.
    pub(super) gerund_cache: HashMap<String, (String, Instant)>,
    /// Persistent "last seen active" timestamps for non-claude agents.
    /// Survives gerund_cache pruning so the age timer keeps ticking.
    pub(super) last_active: HashMap<String, Instant>,
    pub(super) last_meta_refresh: Instant,
    pub(super) focused: bool,
    pub(super) notched: bool,
    pub(super) mode: SidebarMode,
    pub(super) overlay: Option<SidebarOverlay>,
    pub(super) filter: String,
    pub(super) filter_cursor: usize,
    /// Cached usage bars — refreshed on the 3s meta cycle, not every 500ms draw.
    pub(super) usage_bars_cache: Vec<usage_bars::Bar>,
    /// Claude overage snapshot cached alongside usage_bars_cache.
    pub(super) overage: Option<usage_bars::ClaudeOverage>,
    /// Active pulse animations keyed by Bar.label. Populated when a bar's used
    /// pct strictly increases (= remaining decreases) between refreshes.
    /// Entries are pruned once the pulse animation completes.
    pub(super) pulse_starts: HashMap<String, Instant>,
    /// y-origin and height of the usage bars rect from the last draw — used
    /// to map mouse clicks to bar labels for manual pulse triggers.
    pub(super) last_bars_y: u16,
    pub(super) last_bars_h: u16,
    /// Number of tmux process spawns during the most recent refresh().
    pub(super) tmux_call_count: u32,
    /// When true, hidden sessions are included in the list.
    pub(super) show_hidden: bool,
}

pub(super) const ACTIVITY_GRACE: Duration = Duration::from_secs(15);

impl SidebarState {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            visible: Vec::new(),
            current: String::new(),
            selected: 0,
            offset: 0,
            hover: None,
            meta: HashMap::new(),
            gerund_cache: HashMap::new(),
            last_active: HashMap::new(),
            last_meta_refresh: Instant::now() - Duration::from_secs(60),
            focused: true,
            notched: false,
            mode: SidebarMode::Browse,
            overlay: None,
            filter: String::new(),
            filter_cursor: 0,
            usage_bars_cache: Vec::new(),
            overage: None,
            pulse_starts: HashMap::new(),
            last_bars_y: 0,
            last_bars_h: 0,
            tmux_call_count: 0,
            show_hidden: false,
        }
    }

    fn chooser_active(&self) -> bool {
        self.mode == SidebarMode::Chooser
    }

    fn overlay_active(&self) -> bool {
        self.overlay.is_some()
    }

    fn force_refresh(&mut self) {
        self.last_meta_refresh = Instant::now() - Duration::from_secs(60);
    }

    fn open_chooser(&mut self) {
        self.mode = SidebarMode::Chooser;
        self.overlay = None;
        self.filter.clear();
        self.filter_cursor = 0;
        self.offset = 0;
        self.rebuild_visible();
        self.snap_to_current();
    }

    fn close_chooser(&mut self) {
        if !self.chooser_active() {
            return;
        }
        self.mode = SidebarMode::Browse;
        self.filter.clear();
        self.filter_cursor = 0;
        self.offset = 0;
        self.rebuild_visible();
    }

    fn close_overlay(&mut self) {
        self.overlay = None;
    }

    fn rebuild_visible(&mut self) {
        self.visible.clear();
        self.visible.extend(0..self.items.len());
    }

    fn search_matches(&self) -> Vec<(usize, u16)> {
        let selectable: Vec<(usize, &Item)> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.selectable)
            .collect();
        let raw = crate::filter::fuzzy_match(&selectable, &self.filter, |(_, item)| {
            match item.session_id.as_ref() {
                Some(session_id) => format!("{} {}", item.display, session_id),
                None => item.display.clone(),
            }
        });
        raw.into_iter()
            .map(|(match_idx, score)| (selectable[match_idx].0, score))
            .collect()
    }

    fn selectable_visible_indices(&self) -> Vec<usize> {
        self.visible
            .iter()
            .copied()
            .filter(|idx| self.items.get(*idx).is_some_and(|item| item.selectable))
            .collect()
    }

    fn is_visible_index(&self, idx: usize) -> bool {
        self.visible.contains(&idx)
    }

    fn snap_to_first_visible(&mut self) {
        if let Some(&idx) = self
            .visible
            .iter()
            .find(|idx| self.items.get(**idx).is_some_and(|item| item.selectable))
        {
            self.selected = idx;
        }
    }

    fn apply_filter_change(&mut self) {
        self.offset = 0;
        if self.filter.is_empty() {
            self.snap_to_current();
            return;
        }
        if let Some((idx, _)) = self.search_matches().into_iter().next() {
            self.selected = idx;
        }
    }

    fn refresh(&mut self) {
        if let Some(snapshot) = daemon::load_snapshot() {
            self.refresh_from_snapshot(snapshot);
            return;
        }

        self.refresh_direct();
    }

    fn refresh_from_snapshot(&mut self, snapshot: daemon::SidebarSnapshot) {
        let t0 = std::time::Instant::now();
        self.tmux_call_count = 0;
        self.notched = snapshot.notched;

        let cur = std::env::var("TMUX_PANE")
            .ok()
            .and_then(|pane| snapshot.pane_sessions.get(&pane).cloned())
            .or_else(|| (!self.current.is_empty()).then(|| self.current.clone()))
            .or_else(|| snapshot.alive_sessions.first().cloned())
            .unwrap_or_default();

        let alive: HashSet<String> = snapshot.alive_sessions.iter().cloned().collect();
        let sessions = compute_order(&alive, self.show_hidden);
        let meta = snapshot.meta_runtime();

        let pulse_now = Instant::now();
        let bars = snapshot.usage_bars_runtime();
        for nb in &bars {
            if let Some(prev) = self.usage_bars_cache.iter().find(|b| b.label == nb.label)
                && nb.pct > prev.pct
            {
                self.pulse_starts.insert(nb.label.clone(), pulse_now);
            }
        }
        let overage = snapshot.overage_runtime();
        if let (Some(old), Some(new)) = (&self.overage, &overage) {
            if old.five_h != new.five_h {
                self.pulse_starts.insert("over:claude 5h".into(), pulse_now);
            }
            if old.seven_d != new.seven_d {
                self.pulse_starts.insert("over:claude 7d".into(), pulse_now);
            }
            if old.month != new.month {
                self.pulse_starts.insert("over:mo".into(), pulse_now);
            }
            if old.total != new.total {
                self.pulse_starts.insert("over:total".into(), pulse_now);
            }
        }
        self.pulse_starts
            .retain(|_, started| pulse_now.duration_since(*started) < usage_bars::PULSE_DURATION);
        self.usage_bars_cache = bars;
        self.overage = overage;

        let prev_id = self.items.get(self.selected).map(|i| i.id.clone());
        let current_changed = cur != self.current;

        self.meta = meta;
        self.items = build_items(&sessions, &cur, &self.meta);
        self.current = cur;
        self.rebuild_visible();

        let session_count = self.items.len() as u64;

        if !self.focused || current_changed {
            self.snap_to_current();
            debug!(
                duration_ms = t0.elapsed().as_millis() as u64,
                session_count,
                tmux_call_count = self.tmux_call_count,
                snapshot_age_ms = snapshot.age_ms(),
                "sidebar refresh from daemon"
            );
            return;
        }

        if let Some(ref id) = prev_id
            && let Some(pos) = self.items.iter().position(|i| i.id == *id)
            && self.is_visible_index(pos)
        {
            self.selected = pos;
            debug!(
                duration_ms = t0.elapsed().as_millis() as u64,
                session_count,
                tmux_call_count = self.tmux_call_count,
                snapshot_age_ms = snapshot.age_ms(),
                "sidebar refresh from daemon"
            );
            return;
        }
        if self.chooser_active() && !self.filter.is_empty() {
            self.apply_filter_change();
        } else {
            self.snap_to_current();
        }
        debug!(
            duration_ms = t0.elapsed().as_millis() as u64,
            session_count,
            tmux_call_count = self.tmux_call_count,
            snapshot_age_ms = snapshot.age_ms(),
            "sidebar refresh from daemon"
        );
    }

    fn refresh_direct(&mut self) {
        let t0 = std::time::Instant::now();
        self.tmux_call_count = 0;

        // Batch: notched + current session + session list in one tmux invocation
        let batch = tmux(&[
            "show-option",
            "-gv",
            "@notched",
            ";",
            "display-message",
            "-p",
            "#S",
            ";",
            "list-sessions",
            "-F",
            "#S",
        ]);
        self.tmux_call_count += 1;
        let mut lines = batch.lines();
        self.notched = lines.next().unwrap_or("") == "1";
        let cur = lines.next().unwrap_or("").to_string();
        let alive: HashSet<String> = lines.filter(|l| !l.is_empty()).map(String::from).collect();
        let sessions = compute_order(&alive, self.show_hidden);

        let meta_refreshed = self.last_meta_refresh.elapsed() >= Duration::from_secs(3);
        if meta_refreshed {
            let (mut meta, meta_calls) = query_session_meta(&sessions);
            self.tmux_call_count += meta_calls;
            let now = Instant::now();
            for (session, m) in meta.iter_mut() {
                for agent in m.agents.iter_mut() {
                    let cache_key = format!("{}:{}", session, agent.pane_id);
                    // Record last_active from the RAW gerund (before cache),
                    // so the timestamp freezes when the agent truly stops.
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
                    // Derive age from last_active for all agents (claude gets
                    // JSONL mtime in query_session_meta, overwritten here only
                    // if last_active is newer).
                    if let Some(&t) = self.last_active.get(&cache_key) {
                        let from_cache = now.duration_since(t);
                        // For claude, keep the shorter of JSONL age and cache age.
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
            // Detect usage increases (= remaining decrease in display) per bar
            // label and start a pulse animation.
            let pulse_now = Instant::now();
            for nb in &snap.bars {
                if let Some(prev) = self.usage_bars_cache.iter().find(|b| b.label == nb.label)
                    && nb.pct > prev.pct
                {
                    self.pulse_starts.insert(nb.label.clone(), pulse_now);
                }
            }
            // Detect overage dollar-value changes (any direction) and pulse
            // the specific field with a red color flash. Skip first load
            // (old == None) — that's initialization, not a change.
            if let (Some(old), Some(new)) = (&self.overage, &snap.overage) {
                if old.five_h != new.five_h {
                    self.pulse_starts.insert("over:claude 5h".into(), pulse_now);
                }
                if old.seven_d != new.seven_d {
                    self.pulse_starts.insert("over:claude 7d".into(), pulse_now);
                }
                if old.month != new.month {
                    self.pulse_starts.insert("over:mo".into(), pulse_now);
                }
                if old.total != new.total {
                    self.pulse_starts.insert("over:total".into(), pulse_now);
                }
            }
            self.pulse_starts.retain(|_, started| {
                pulse_now.duration_since(*started) < usage_bars::PULSE_DURATION
            });
            self.usage_bars_cache = snap.bars;
            self.overage = snap.overage;
        }

        let prev_id = self.items.get(self.selected).map(|i| i.id.clone());
        // External session switches (e.g. Ctrl+Tab toggling the last session)
        // should drag the cursor along, not just the "current session"
        // highlight — staying put would leave the selection orphaned on a
        // stale session.
        let current_changed = cur != self.current;

        self.items = build_items(&sessions, &cur, &self.meta);
        self.current = cur;
        self.rebuild_visible();

        let session_count = self.items.len() as u64;

        // When unfocused, or when the active session changed from under us,
        // track the current session.
        if !self.focused || current_changed {
            self.snap_to_current();
            debug!(
                duration_ms = t0.elapsed().as_millis() as u64,
                session_count,
                tmux_call_count = self.tmux_call_count,
                usage_cache_hit = !meta_refreshed,
                "sidebar refresh"
            );
            return;
        }

        if let Some(ref id) = prev_id
            && let Some(pos) = self.items.iter().position(|i| i.id == *id)
            && self.is_visible_index(pos)
        {
            self.selected = pos;
            debug!(
                duration_ms = t0.elapsed().as_millis() as u64,
                session_count,
                tmux_call_count = self.tmux_call_count,
                usage_cache_hit = !meta_refreshed,
                "sidebar refresh"
            );
            return;
        }
        if self.chooser_active() && !self.filter.is_empty() {
            self.apply_filter_change();
        } else {
            self.snap_to_current();
        }
        debug!(
            duration_ms = t0.elapsed().as_millis() as u64,
            session_count,
            tmux_call_count = self.tmux_call_count,
            usage_cache_hit = !meta_refreshed,
            "sidebar refresh"
        );
    }

    fn snap_to_current(&mut self) {
        if let Some(pos) = self
            .items
            .iter()
            .position(|i| i.selectable && i.id == self.current)
            .filter(|pos| self.is_visible_index(*pos))
        {
            self.selected = pos;
            return;
        }
        self.snap_to_first_visible();
    }

    fn move_sel(&mut self, dir: i32) {
        let selectable = self.selectable_visible_indices();
        if selectable.is_empty() {
            return;
        }
        let Some(mut pos) = selectable.iter().position(|idx| *idx == self.selected) else {
            self.selected = selectable[0];
            return;
        };
        if dir > 0 {
            if pos + 1 >= selectable.len() {
                return;
            }
            pos += 1;
        } else {
            if pos == 0 {
                return;
            }
            pos -= 1;
        }
        self.selected = selectable[pos];
    }

    fn selected_session_id(&self) -> Option<String> {
        if !self.is_visible_index(self.selected) {
            return None;
        }
        self.items
            .get(self.selected)
            .and_then(|i| i.session_id.clone())
    }

    fn switch_to_selected(&self) {
        if let Some(id) = self.selected_session_id() {
            // A tmux-native sidebar lives inside a tmux window, unlike the old
            // WezTerm-native split. When jumping from the sidebar, pre-create
            // the same sidebar in the target session's current window so the
            // UX stays persistent without global auto-spawn hooks.
            if running_as_tmux_sidebar() {
                open_tmux_sidebar_in_target(Some(&id), false);
            }
            tmux(&["switch-client", "-t", &id]);
        }
    }

    fn move_selected_session(&mut self, direction: &str) {
        if let Some(id) = self.selected_session_id() {
            let exe = std::env::current_exe().unwrap_or_else(|_| "mux".into());
            let _ = Command::new(exe)
                .args(["move", direction, &id])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            self.last_meta_refresh = Instant::now() - Duration::from_secs(60);
            self.refresh();
        }
    }

    fn select_by_number(&mut self, c: char) {
        let n = (c as usize) - ('1' as usize);
        let selectable = self.selectable_visible_indices();
        if let Some(&idx) = selectable.get(n) {
            self.selected = idx;
            self.switch_to_selected();
        }
    }

    fn visible_agent_animation_active(&self, list_h: u16) -> bool {
        let visible_rows = (list_h as usize).min(self.visible.len().saturating_sub(self.offset));
        self.visible
            .iter()
            .skip(self.offset)
            .take(visible_rows)
            .filter_map(|idx| self.items.get(*idx))
            .any(|item| {
                matches!(
                    &item.kind,
                    ItemKind::Agent { gerund, asking, .. } if gerund.is_some() || *asking
                )
            })
    }
}

// ── Main loop ────────────────────────────────────────────────

/// Exposed for `mux bench` — runs the full meta query pipeline and discards the result.
pub(crate) fn bench_query_session_meta(sessions: &[String]) {
    let _ = query_session_meta(sessions);
}

fn pane_is_sidebar_line(line: &str) -> Option<String> {
    let mut parts = line.split('\t');
    let pane = parts.next()?;
    let marker = parts.next()?;
    let token = parts.next()?;
    let command = parts.next().unwrap_or_default();
    (marker == "1" && token == SIDEBAR_TOKEN && command == "mux" && !pane.is_empty())
        .then(|| pane.to_string())
}

fn running_as_tmux_sidebar() -> bool {
    std::env::var("MUX_SIDEBAR_TMUX").ok().as_deref() == Some(SIDEBAR_TOKEN)
}

fn current_pane_id() -> String {
    if let Ok(pane) = std::env::var("TMUX_PANE")
        && !pane.trim().is_empty()
    {
        return pane;
    }
    tmux(&["display-message", "-p", "#{pane_id}"])
}

fn current_window_id() -> Option<String> {
    let window = tmux(&["display-message", "-p", "#{window_id}"]);
    (!window.trim().is_empty()).then_some(window)
}

fn mark_current_sidebar_pane() {
    if !running_as_tmux_sidebar() {
        return;
    }
    if std::env::var("MUX_SIDEBAR_MARKED").ok().as_deref() == Some("1") {
        return;
    }
    let pane = current_pane_id();
    if pane.is_empty() {
        return;
    }
    tmux(&["set-option", "-p", "-t", &pane, "@mux_sidebar", "1"]);
    tmux(&[
        "set-option",
        "-p",
        "-t",
        &pane,
        "@mux_sidebar_token",
        SIDEBAR_TOKEN,
    ]);
}

fn unmark_current_sidebar_pane() {
    if !running_as_tmux_sidebar() {
        return;
    }
    let pane = current_pane_id();
    if pane.is_empty() {
        return;
    }
    tmux(&["set-option", "-pu", "-t", &pane, "@mux_sidebar"]);
    tmux(&["set-option", "-pu", "-t", &pane, "@mux_sidebar_token"]);
}

fn sidebar_pane_in_target(target: Option<&str>) -> Option<String> {
    let format = "#{pane_id}\t#{@mux_sidebar}\t#{@mux_sidebar_token}\t#{pane_current_command}";
    let output = if let Some(target) = target {
        tmux(&["list-panes", "-t", target, "-F", format])
    } else {
        tmux(&["list-panes", "-F", format])
    };

    output.lines().find_map(pane_is_sidebar_line)
}

fn current_sidebar_pane() -> Option<String> {
    sidebar_pane_in_target(None)
}

fn all_tmux_windows_and_sidebars() -> (Vec<String>, HashMap<String, String>) {
    let output = tmux(&[
        "list-panes",
        "-a",
        "-F",
        "#{window_id}\t#{pane_id}\t#{@mux_sidebar}\t#{@mux_sidebar_token}\t#{pane_current_command}",
    ]);
    let mut windows = Vec::new();
    let mut sidebar_by_window = HashMap::new();
    for line in output.lines() {
        let mut parts = line.split('\t');
        let Some(window) = parts.next().filter(|s| !s.is_empty()) else {
            continue;
        };
        let Some(pane) = parts.next().filter(|s| !s.is_empty()) else {
            continue;
        };
        if !windows.iter().any(|seen| seen == window) {
            windows.push(window.to_string());
        }
        let marker = parts.next().unwrap_or_default();
        let token = parts.next().unwrap_or_default();
        let command = parts.next().unwrap_or_default();
        if marker == "1" && token == SIDEBAR_TOKEN && command == "mux" {
            sidebar_by_window
                .entry(window.to_string())
                .or_insert_with(|| pane.to_string());
        }
    }
    (windows, sidebar_by_window)
}

fn all_sidebar_panes() -> Vec<String> {
    tmux(&[
        "list-panes",
        "-a",
        "-F",
        "#{pane_id}\t#{@mux_sidebar}\t#{@mux_sidebar_token}\t#{pane_current_command}",
    ])
    .lines()
    .filter_map(pane_is_sidebar_line)
    .collect()
}

fn any_sidebar_pane() -> bool {
    !all_sidebar_panes().is_empty()
}

fn active_pane_is_sidebar() -> bool {
    tmux(&[
        "display-message",
        "-p",
        "#{@mux_sidebar}\t#{@mux_sidebar_token}\t#{pane_current_command}",
    ]) == format!("1\t{SIDEBAR_TOKEN}\tmux")
}

fn sidebar_enabled() -> bool {
    tmux(&["show-option", "-gv", "@sidebar_enabled"]) != "0"
}

fn sidebar_width() -> String {
    let width = tmux(&["show-option", "-gqv", "@sidebar_width"]);
    if width.is_empty() {
        SIDEBAR_WIDTH_DEFAULT.to_string()
    } else {
        width
    }
}

fn resize_sidebar_pane(pane: &str) {
    let width = sidebar_width();
    tmux(&["resize-pane", "-t", pane, "-x", &width]);
}

fn resize_all_tmux_sidebars() {
    batch_resize_sidebar_panes(&all_sidebar_panes(), &sidebar_width());
}

fn batch_resize_sidebar_panes(panes: &[String], width: &str) {
    if panes.is_empty() {
        return;
    }
    let mut args: Vec<String> = Vec::new();
    for (idx, pane) in panes.iter().enumerate() {
        if idx > 0 {
            args.push(";".into());
        }
        args.extend([
            "resize-pane".into(),
            "-t".into(),
            pane.clone(),
            "-x".into(),
            width.to_string(),
        ]);
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    tmux(&refs);
}

fn batch_mark_and_resize_sidebar_panes(panes: &[String], width: &str) {
    if panes.is_empty() {
        return;
    }
    let mut args: Vec<String> = Vec::new();
    for (idx, pane) in panes.iter().enumerate() {
        if idx > 0 {
            args.push(";".into());
        }
        args.extend([
            "set-option".into(),
            "-p".into(),
            "-t".into(),
            pane.clone(),
            "@mux_sidebar".into(),
            "1".into(),
            ";".into(),
            "set-option".into(),
            "-p".into(),
            "-t".into(),
            pane.clone(),
            "@mux_sidebar_token".into(),
            SIDEBAR_TOKEN.into(),
            ";".into(),
            "resize-pane".into(),
            "-t".into(),
            pane.clone(),
            "-x".into(),
            width.to_string(),
        ]);
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    tmux(&refs);
}

fn split_sidebar_panes_in_windows(windows: &[String], width: &str) -> Vec<String> {
    if windows.is_empty() {
        return Vec::new();
    }

    let exe = std::env::current_exe().unwrap_or_else(|_| "mux".into());
    let command = format!(
        "exec env MUX_SIDEBAR_TMUX={} MUX_SIDEBAR_MARKED=1 {} sidebar",
        SIDEBAR_TOKEN,
        exe.display()
    );
    let mut args: Vec<String> = Vec::new();
    for (idx, window) in windows.iter().enumerate() {
        if idx > 0 {
            args.push(";".into());
        }
        args.extend([
            "split-window".into(),
            "-t".into(),
            window.clone(),
            "-h".into(),
            "-b".into(),
            "-f".into(),
            "-l".into(),
            width.to_string(),
            "-d".into(),
            "-P".into(),
            "-F".into(),
            "#{pane_id}".into(),
            command.clone(),
        ]);
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    tmux(&refs)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn sidebar_borderless() -> bool {
    tmux(&["show-option", "-gqv", "@sidebar_borderless"]) != "0"
}

fn sidebar_border_color() -> String {
    let color = tmux(&["show-option", "-gqv", "@sidebar_border_color"]);
    if color.is_empty() {
        SIDEBAR_BORDER_COLOR.to_string()
    } else {
        color
    }
}

fn local_pane_option(pane: &str, option: &str) -> String {
    let raw = tmux(&["show-option", "-pq", "-t", pane, option]);
    raw.strip_prefix(option)
        .and_then(|value| value.strip_prefix(' '))
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

fn set_pane_option(pane: &str, option: &str, value: &str) {
    tmux(&["set-option", "-p", "-t", pane, option, value]);
}

fn unset_pane_option(pane: &str, option: &str) {
    tmux(&["set-option", "-pu", "-t", pane, option]);
}

fn apply_borderless_to_pane(pane: &str) {
    if !sidebar_borderless() {
        return;
    }

    let style = format!("fg={}", sidebar_border_color());
    if local_pane_option(pane, "@mux_sidebar_border_touched") != "1" {
        let prev_border = local_pane_option(pane, "pane-border-style");
        let prev_active = local_pane_option(pane, "pane-active-border-style");
        set_pane_option(pane, "@mux_sidebar_prev_pane_border_style", &prev_border);
        set_pane_option(
            pane,
            "@mux_sidebar_prev_pane_active_border_style",
            &prev_active,
        );
        set_pane_option(pane, "@mux_sidebar_border_touched", "1");
    }

    set_pane_option(pane, "pane-border-style", &style);
    set_pane_option(pane, "pane-active-border-style", &style);
}

fn restore_sidebar_borders() {
    let output = tmux(&[
        "list-panes",
        "-a",
        "-F",
        "#{pane_id}\t#{@mux_sidebar_border_touched}",
    ]);

    for pane in output.lines().filter_map(|line| {
        let (pane, touched) = line.split_once('\t')?;
        (touched == "1" && !pane.is_empty()).then_some(pane.to_string())
    }) {
        let prev_border = local_pane_option(&pane, "@mux_sidebar_prev_pane_border_style");
        let prev_active = local_pane_option(&pane, "@mux_sidebar_prev_pane_active_border_style");

        if prev_border.is_empty() {
            unset_pane_option(&pane, "pane-border-style");
        } else {
            set_pane_option(&pane, "pane-border-style", &prev_border);
        }

        if prev_active.is_empty() {
            unset_pane_option(&pane, "pane-active-border-style");
        } else {
            set_pane_option(&pane, "pane-active-border-style", &prev_active);
        }

        unset_pane_option(&pane, "@mux_sidebar_prev_pane_border_style");
        unset_pane_option(&pane, "@mux_sidebar_prev_pane_active_border_style");
        unset_pane_option(&pane, "@mux_sidebar_border_touched");
    }
}

fn set_sidebar_open(open: bool) {
    if open {
        tmux(&["set-option", "-g", "@sidebar_open", "1"]);
    } else if !any_sidebar_pane() {
        tmux(&["set-option", "-gu", "@sidebar_open"]);
    }
    refresh_status_bar();
}

fn open_tmux_sidebar_in_target(target: Option<&str>, select: bool) -> Option<String> {
    if !sidebar_enabled() {
        return None;
    }

    // session-created/after-new-window hooks can fire in a burst while the
    // three-window session template is being assembled. Without serializing the
    // check+split sequence, those background jobs can all observe "no sidebar"
    // and briefly create duplicate sidebar panes that prune-orphans cleans up a
    // moment later. The final layout was right, but the visible pop-in/pop-out
    // was jarring.
    let _lock = acquire_sidebar_open_lock()?;

    open_tmux_sidebar_in_target_locked(target, select)
}

fn open_tmux_sidebar_in_target_locked(target: Option<&str>, select: bool) -> Option<String> {
    if let Some(pane) = sidebar_pane_in_target(target) {
        resize_sidebar_pane(&pane);
        apply_borderless_to_pane(&pane);
        if select {
            tmux(&["select-pane", "-t", &pane]);
        }
        set_sidebar_open(true);
        return Some(pane);
    }

    let exe = std::env::current_exe().unwrap_or_else(|_| "mux".into());
    let command = format!(
        "exec env MUX_SIDEBAR_TMUX={} MUX_SIDEBAR_MARKED=1 {} sidebar",
        SIDEBAR_TOKEN,
        exe.display()
    );
    let width = sidebar_width();
    let pane = if let Some(target) = target {
        tmux(&[
            "split-window",
            "-t",
            target,
            "-h",
            "-b",
            "-f",
            "-l",
            &width,
            "-d",
            "-P",
            "-F",
            "#{pane_id}",
            &command,
        ])
    } else {
        tmux(&[
            "split-window",
            "-h",
            "-b",
            "-f",
            "-l",
            &width,
            "-d",
            "-P",
            "-F",
            "#{pane_id}",
            &command,
        ])
    };

    if pane.is_empty() {
        return None;
    }

    tmux(&["set-option", "-p", "-t", &pane, "@mux_sidebar", "1"]);
    tmux(&[
        "set-option",
        "-p",
        "-t",
        &pane,
        "@mux_sidebar_token",
        SIDEBAR_TOKEN,
    ]);
    resize_sidebar_pane(&pane);
    apply_borderless_to_pane(&pane);
    set_sidebar_open(true);
    if select {
        tmux(&["select-pane", "-t", &pane]);
    }
    Some(pane)
}

fn open_tmux_sidebar(select: bool) -> Option<String> {
    open_tmux_sidebar_in_target(None, select)
}

fn open_tmux_sidebar_everywhere() {
    if !sidebar_enabled() {
        return;
    }

    daemon::ensure_started();

    let _lock = match acquire_sidebar_open_lock() {
        Some(lock) => lock,
        None => return,
    };

    let current_window = tmux(&["display-message", "-p", "#{window_id}"]);
    let (windows, existing_by_window) = all_tmux_windows_and_sidebars();
    let width = sidebar_width();
    let missing_windows: Vec<String> = windows
        .iter()
        .filter(|window| !existing_by_window.contains_key(*window))
        .cloned()
        .collect();
    let mut panes: Vec<String> = existing_by_window.values().cloned().collect();
    let new_panes = split_sidebar_panes_in_windows(&missing_windows, &width);
    panes.extend(new_panes);

    batch_mark_and_resize_sidebar_panes(&panes, &width);
    if sidebar_borderless() {
        for pane in &panes {
            apply_borderless_to_pane(pane);
        }
    }

    prune_orphan_sidebar_windows();

    set_sidebar_open(any_sidebar_pane());

    if active_pane_is_sidebar() {
        return;
    }

    // Keep focus in the user's current content pane after a global reveal.
    if windows.iter().any(|window| window == &current_window) {
        let active = tmux(&["list-panes", "-F", "#{pane_id}\t#{pane_active}"])
            .lines()
            .find_map(|line| {
                let (pane_id, active) = line.split_once('\t')?;
                let is_sidebar = panes.iter().any(|pane| pane == pane_id);
                (active == "1" && !is_sidebar).then(|| pane_id.to_string())
            });
        if let Some(active) = active {
            tmux(&["select-pane", "-t", &active]);
        }
    }
}

fn close_all_tmux_sidebars() {
    for pane in all_sidebar_panes() {
        tmux(&["kill-pane", "-t", &pane]);
    }
    restore_sidebar_borders();
    set_sidebar_open(false);
}

fn prune_orphan_sidebar_windows() {
    let output = tmux(&[
        "list-panes",
        "-a",
        "-F",
        "#{window_id}\t#{pane_id}\t#{pane_left}\t#{@mux_sidebar}\t#{@mux_sidebar_token}\t#{pane_current_command}",
    ]);

    let mut windows: std::collections::HashMap<String, (Vec<(u16, String)>, usize)> =
        std::collections::HashMap::new();

    for line in output.lines() {
        let mut parts = line.split('\t');
        let Some(window_id) = parts.next() else {
            continue;
        };
        let Some(pane_id) = parts.next() else {
            continue;
        };
        let pane_left: u16 = parts
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(u16::MAX);
        if window_id.is_empty() || pane_id.is_empty() {
            continue;
        }
        let marker = parts.next().unwrap_or_default();
        let token = parts.next().unwrap_or_default();
        let command = parts.next().unwrap_or_default();
        let is_sidebar = marker == "1" && token == SIDEBAR_TOKEN && command == "mux";

        let entry = windows.entry(window_id.to_string()).or_default();
        if is_sidebar {
            entry.0.push((pane_left, pane_id.to_string()));
        } else {
            entry.1 += 1;
        }
    }

    for (window_id, (mut sidebars, content)) in windows {
        if !sidebars.is_empty() && content == 0 {
            tmux(&["kill-window", "-t", &window_id]);
            continue;
        }

        // If multiple global-open jobs race (for example new-session hooks),
        // keep the leftmost marked sidebar and kill the duplicates. Only panes
        // passing the full marker+token+command check above are eligible.
        if sidebars.len() > 1 {
            sidebars.sort_by_key(|(left, _)| *left);
            for (_, pane) in sidebars.into_iter().skip(1) {
                tmux(&["kill-pane", "-t", &pane]);
            }
        }
    }
}

fn focus_tmux_sidebar() {
    if active_pane_is_sidebar() {
        tmux(&["switch-client", "-l"]);
        tmux(&["select-pane", "-R"]);
    } else if let Some(window) = current_window_id()
        && let Some(pane) = sidebar_pane_in_target(Some(&window))
    {
        tmux(&["select-pane", "-t", &pane]);
    } else if let Some(pane) = current_sidebar_pane() {
        tmux(&["select-pane", "-t", &pane]);
    } else {
        open_tmux_sidebar(true);
    }
}

pub(crate) fn send_key_to_current_sidebar(key: &str) -> bool {
    let Some(pane) = current_sidebar_pane() else {
        return false;
    };
    tmux(&["select-pane", "-t", &pane]);
    tmux(&["send-keys", "-t", &pane, "-l", key]);
    true
}

pub(crate) fn cmd_sidebar_control(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("toggle") => {
            if any_sidebar_pane() {
                close_all_tmux_sidebars();
            } else {
                open_tmux_sidebar_everywhere();
            }
        }
        Some("open") => {
            open_tmux_sidebar_everywhere();
        }
        Some("focus") => focus_tmux_sidebar(),
        Some("close") => close_all_tmux_sidebars(),
        Some("prune-orphans") => prune_orphan_sidebar_windows(),
        Some("resize") => resize_all_tmux_sidebars(),
        _ => {}
    }
}

pub(crate) fn cmd_sidebar() {
    daemon::ensure_started();

    crate::usage::start_all();

    mark_current_sidebar_pane();

    // Set WezTerm user var for toggle detection
    // "dHJ1ZQ==" is base64("true")
    print!("\x1b]1337;SetUserVar=is_sidebar=dHJ1ZQ==\x07");
    io::stdout().flush().ok();

    // Tell tmux status bar to hide the session list while sidebar is open
    tmux(&["set-option", "-g", "@sidebar_open", "1"]);
    refresh_status_bar();

    // On notched displays, paint this pane's terminal background black via
    // OSC 11 so the wezterm split reads as solid black (matching the status
    // filler) without affecting the main pane's bg.
    let notched_pane = tmux(&["show-option", "-gv", "@notched"]) == "1";
    if notched_pane {
        print!("\x1b]11;rgb:0000/0000/0000\x1b\\");
        io::stdout().flush().ok();
    }

    enter_tui();
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let mut state = SidebarState::new();
    state.refresh();

    // Cache layout for mouse-click mapping between draws
    let mut last_list_y: u16 = 0;
    let mut last_list_h: u16 = 0;
    let mut last_refresh = Instant::now();
    const IDLE_POLL: Duration = Duration::from_millis(500);
    const PULSE_POLL: Duration = Duration::from_millis(33); // ~30 fps during pulse
    const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

    loop {
        terminal
            .draw(|f| {
                let (ly, lh) = draw(f, &mut state);
                last_list_y = ly;
                last_list_h = lh;
            })
            .ok();

        // High-frequency redraw while any animation is mid-flight: bar
        // pulses OR gerund percolation.
        let pulse_active = state.last_bars_h > 0
            && state
                .pulse_starts
                .values()
                .any(|s| s.elapsed() < usage_bars::PULSE_DURATION);
        let agent_animation_active = state.visible_agent_animation_active(last_list_h);
        let poll_timeout = if pulse_active || agent_animation_active {
            PULSE_POLL
        } else {
            IDLE_POLL
        };

        if event::poll(poll_timeout).unwrap_or(false) {
            match event::read() {
                Ok(Event::FocusGained) => {
                    state.focused = true;
                }
                Ok(Event::FocusLost) => {
                    state.focused = false;
                    state.hover = None;
                    state.close_overlay();
                    state.close_chooser();
                    state.snap_to_current();
                }
                Ok(Event::Mouse(_)) if state.overlay_active() => {}
                Ok(Event::Mouse(me)) => match me.kind {
                    MouseEventKind::Down(MouseButton::Left)
                        if state.last_bars_h > 0
                            && me.row >= state.last_bars_y
                            && me.row < state.last_bars_y + state.last_bars_h =>
                    {
                        // Manual pulse trigger for testing.
                        let bar_idx =
                            ((me.row - state.last_bars_y) / usage_bars::ROWS_PER_BAR) as usize;
                        if let Some(label) =
                            state.usage_bars_cache.get(bar_idx).map(|b| b.label.clone())
                        {
                            state.pulse_starts.insert(label, Instant::now());
                        }
                    }
                    MouseEventKind::Down(MouseButton::Left)
                        if me.row >= last_list_y && me.row < last_list_y + last_list_h =>
                    {
                        let vis_idx = state.offset + (me.row - last_list_y) as usize;
                        if let Some(item_idx) = state.visible.get(vis_idx).copied()
                            && let Some(sid) =
                                state.items.get(item_idx).and_then(|i| i.session_id.clone())
                            && let Some(row_idx) = state
                                .items
                                .iter()
                                .position(|i| i.selectable && i.session_id.as_ref() == Some(&sid))
                        {
                            state.selected = row_idx;
                            state.switch_to_selected();
                            if state.chooser_active() {
                                state.close_chooser();
                                focus_main_pane();
                            }
                        }
                    }
                    MouseEventKind::Moved => {
                        if me.row >= last_list_y && me.row < last_list_y + last_list_h {
                            let vis_idx = state.offset + (me.row - last_list_y) as usize;
                            state.hover = state
                                .visible
                                .get(vis_idx)
                                .and_then(|idx| state.items.get(*idx))
                                .and_then(|it| it.session_id.clone());
                        } else {
                            state.hover = None;
                        }
                    }
                    MouseEventKind::ScrollUp => state.move_sel(-1),
                    MouseEventKind::ScrollDown => state.move_sel(1),
                    _ => {}
                },
                Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                    if state.overlay_active() {
                        let handled = state.handle_overlay_key(key);
                        if handled || state.overlay_active() {
                            continue;
                        }
                    }

                    if state.chooser_active() {
                        match (key.code, key.modifiers) {
                            (KeyCode::Esc, _) => {
                                state.close_chooser();
                                continue;
                            }
                            (KeyCode::Enter, _) => {
                                state.switch_to_selected();
                                state.close_chooser();
                                focus_main_pane();
                                continue;
                            }
                            (KeyCode::Char('h'), m) if m.contains(KeyModifiers::ALT) => {
                                if let Some(id) = state.selected_session_id() {
                                    toggle_hidden(&id);
                                    state.force_refresh();
                                }
                                continue;
                            }
                            (KeyCode::Char('j'), KeyModifiers::ALT) => {
                                state.move_selected_session("down");
                                continue;
                            }
                            (KeyCode::Char('k'), KeyModifiers::ALT) => {
                                state.move_selected_session("up");
                                continue;
                            }
                            _ if handle_readline_key(
                                &mut state.filter,
                                &mut state.filter_cursor,
                                key,
                            ) =>
                            {
                                state.apply_filter_change();
                                continue;
                            }
                            (KeyCode::Char('j'), _)
                            | (KeyCode::Down, _)
                            | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                                state.move_sel(1);
                                continue;
                            }
                            (KeyCode::Char('k'), _)
                            | (KeyCode::Up, _)
                            | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                                state.move_sel(-1);
                                continue;
                            }
                            _ => {}
                        }
                    }

                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _)
                        | (KeyCode::Esc, _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                        // Cmd+O from WezTerm arrives as Ctrl+O (see wezterm.lua
                        // focus_sidebar) — toggle to the last session.
                        (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                            tmux(&["switch-client", "-l"]);
                            focus_main_pane();
                        }
                        (KeyCode::Char('j'), KeyModifiers::ALT) => {
                            state.move_selected_session("down");
                        }
                        (KeyCode::Char('k'), KeyModifiers::ALT) => {
                            state.move_selected_session("up");
                        }
                        (KeyCode::Char('j'), _)
                        | (KeyCode::Down, _)
                        | (KeyCode::Char('n'), KeyModifiers::CONTROL) => state.move_sel(1),
                        (KeyCode::Char('k'), _)
                        | (KeyCode::Up, _)
                        | (KeyCode::Char('p'), KeyModifiers::CONTROL) => state.move_sel(-1),
                        (KeyCode::Enter, _) => {
                            state.switch_to_selected();
                            focus_main_pane();
                        }
                        (KeyCode::Char('n'), m)
                            if !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                        {
                            state.open_project_overlay();
                        }
                        (KeyCode::Char('w'), _) => {
                            state.open_worktree_overlay();
                        }
                        (KeyCode::Char('r'), _) => {
                            state.open_rename_overlay();
                        }
                        (KeyCode::Char('x'), _) => {
                            state.open_ditch_overlay();
                        }
                        (KeyCode::Char('h'), m) if m.contains(KeyModifiers::ALT) => {
                            if let Some(id) = state.selected_session_id() {
                                toggle_hidden(&id);
                                state.force_refresh();
                            }
                        }
                        (KeyCode::Char('h'), _) => {
                            state.show_hidden = !state.show_hidden;
                        }
                        (KeyCode::Char('/'), _) => {
                            state.open_chooser();
                        }
                        (KeyCode::Char(c @ '1'..='9'), m)
                            if !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                        {
                            state.select_by_number(c);
                        }
                        (KeyCode::Char(c), m)
                            if !m.intersects(
                                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                            ) =>
                        {
                            forward_char_to_main(c);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Throttle refresh to IDLE cadence — pulse-driven high-fps redraws
        // shouldn't multiply tmux process spawns.
        if last_refresh.elapsed() >= REFRESH_INTERVAL {
            state.refresh();
            last_refresh = Instant::now();
        }
    }

    leave_tui();

    // Reset pane background (OSC 111) to whatever the user's theme defines.
    if notched_pane {
        print!("\x1b]111\x1b\\");
        io::stdout().flush().ok();
    }

    // Restore status bar. For tmux-native sidebars, unmark this pane before
    // checking whether any other marked sidebars remain; otherwise our own
    // soon-to-exit pane would keep @sidebar_open stuck on.
    let tmux_native_sidebar = running_as_tmux_sidebar();
    unmark_current_sidebar_pane();
    if tmux_native_sidebar {
        close_all_tmux_sidebars();
        return;
    }
    set_sidebar_open(false);
}

pub(crate) fn cmd_sidebar_daemon() {
    daemon::cmd_sidebar_daemon();
}

fn refresh_status_bar() {
    crate::process::spawn_detached_update();
}

fn enter_tui() {
    terminal::enable_raw_mode().ok();
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableFocusChange,
        cursor::Hide
    );
}

fn leave_tui() {
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(
        stdout,
        DisableFocusChange,
        DisableMouseCapture,
        cursor::Show,
        LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
}

fn toggle_hidden(session: &str) {
    let path = crate::order::hidden_file();
    let mut lines = crate::order::load_lines(&path);
    if let Some(pos) = lines.iter().position(|l| l == session) {
        lines.remove(pos);
    } else {
        lines.push(session.to_string());
    }
    crate::order::save_lines(&path, &lines);
}

fn focus_main_pane() {
    if running_as_tmux_sidebar() {
        tmux(&["select-pane", "-R"]);
        return;
    }

    let mut command = Command::new("wezterm");
    command
        .args(["cli", "activate-pane-direction", "Right"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _ = spawn_reaped(command);
}

/// Bounce an accidental keystroke to the neighbouring pane so typing `ls` in
/// the sidebar by mistake still lands where the user expected.
fn forward_char_to_main(c: char) {
    if running_as_tmux_sidebar() {
        tmux(&["select-pane", "-R"]);
        let mut buf = [0u8; 4];
        let text = c.encode_utf8(&mut buf);
        tmux(&["send-keys", "-l", text]);
        return;
    }

    let Ok(out) = Command::new("wezterm")
        .args(["cli", "get-pane-direction", "Right"])
        .output()
    else {
        return;
    };
    if !out.status.success() {
        return;
    }
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if id.is_empty() {
        return;
    }
    let _ = Command::new("wezterm")
        .args(["cli", "activate-pane", "--pane-id", &id])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let mut buf = [0u8; 4];
    let text = c.encode_utf8(&mut buf);
    let _ = Command::new("wezterm")
        .args(["cli", "send-text", "--no-paste", "--pane-id", &id, text])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
