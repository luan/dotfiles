use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crossterm::cursor;
use crossterm::event::{
    self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
    KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;

use crate::order::compute_order;
use crate::tmux::tmux;
use tracing::debug;

use crate::usage_bars;

mod claude;
pub(crate) mod meta;
mod overlay;
mod render;
mod tree;

use meta::{SessionMeta, query_session_meta};
use overlay::{SidebarOverlay, handle_readline_key};
use render::draw;
use tree::{Item, build_items};

// Nerd Font keyboard modifier glyphs (md-apple-keyboard-* + md-keyboard-tab).
// These render at proper size/weight where the bare Unicode symbols (⌘⌃⌥⇧⇥)
// fall back to a non-keyboard font and come out tiny or wrong.
pub(super) const KEY_CMD: &str = "\u{F0633}";
pub(super) const KEY_CTRL: &str = "\u{F0634}";
pub(super) const KEY_OPT: &str = "\u{F0635}";
pub(super) const KEY_SHIFT: &str = "\u{F0636}";
pub(super) const KEY_TAB: &str = "\u{F0312}";

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
        // External session switches (e.g. MRU-cycle via Ctrl+Tab) should drag
        // the cursor along, not just the "current session" highlight — staying
        // put would leave the selection orphaned on a stale session.
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
}

// ── Main loop ────────────────────────────────────────────────

/// Exposed for `mux bench` — runs the full meta query pipeline and discards the result.
pub(crate) fn bench_query_session_meta(sessions: &[String]) {
    let _ = query_session_meta(sessions);
}

pub(crate) fn cmd_sidebar() {
    crate::usage::start_all();

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
        let pulse_active = state
            .pulse_starts
            .values()
            .any(|s| s.elapsed() < usage_bars::PULSE_DURATION);
        let gerund_active = state
            .gerund_cache
            .values()
            .any(|(_, t)| t.elapsed() < ACTIVITY_GRACE);
        let any_asking = state
            .meta
            .values()
            .any(|m| m.agents.iter().any(|a| a.asking));
        let poll_timeout = if pulse_active || gerund_active || any_asking {
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
                                KeyModifiers::CONTROL
                                    | KeyModifiers::ALT
                                    | KeyModifiers::SUPER,
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

    // Restore status bar
    tmux(&["set-option", "-gu", "@sidebar_open"]);
    refresh_status_bar();
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
    let _ = Command::new("wezterm")
        .args(["cli", "activate-pane-direction", "Right"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// Bounce an accidental keystroke to the neighbouring pane so typing `ls` in
/// the sidebar by mistake still lands where the user expected.
fn forward_char_to_main(c: char) {
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
        .args([
            "cli",
            "send-text",
            "--no-paste",
            "--pane-id",
            &id,
            text,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
