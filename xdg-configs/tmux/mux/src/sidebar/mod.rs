use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::cursor;
use crossterm::event::{
    self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
    KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::order::compute_order;
use crate::process::spawn_reaped;
use crate::tmux::tmux;
use tracing::debug;

#[allow(dead_code)]
pub mod bench_support;
mod claude;
mod hooks;
pub mod instrument;
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
const TERMINAL_RUNTIME_USER_VAR: &str = "dGVybWluYWw="; // base64("terminal")
const TERMINAL_SIDEBAR_TITLE: &str = "mux-sidebar-terminal";
const BOOT_CACHE_VERSION: u32 = 2;
const BOOT_CACHE_MAX_AGE: Duration = Duration::from_secs(60);

#[derive(Serialize, Deserialize)]
struct SidebarBootCache {
    version: u32,
    written_at_ms: u128,
    current: String,
    sessions: Vec<String>,
    notched: bool,
    meta: HashMap<String, SessionMeta>,
    usage_lines: Vec<String>,
}

fn boot_cache_path() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("mux-sidebar-boot-cache.json")
}

fn now_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn read_boot_cache() -> Option<SidebarBootCache> {
    let cache: SidebarBootCache =
        serde_json::from_slice(&std::fs::read(boot_cache_path()).ok()?).ok()?;
    if cache.version != BOOT_CACHE_VERSION {
        return None;
    }
    let age = now_epoch_ms().saturating_sub(cache.written_at_ms);
    (age <= BOOT_CACHE_MAX_AGE.as_millis()).then_some(cache)
}

fn write_boot_cache(
    current: &str,
    sessions: &[String],
    notched: bool,
    meta: &HashMap<String, SessionMeta>,
    usage_lines: &[String],
) {
    let cache = SidebarBootCache {
        version: BOOT_CACHE_VERSION,
        written_at_ms: now_epoch_ms(),
        current: current.to_string(),
        sessions: sessions.to_vec(),
        notched,
        meta: meta.clone(),
        usage_lines: usage_lines.to_vec(),
    };
    let Ok(json) = serde_json::to_vec(&cache) else {
        return;
    };
    let path = boot_cache_path();
    std::thread::spawn(move || {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, json).is_ok() {
            let _ = std::fs::rename(tmp, path);
        }
    });
}

pub(crate) fn attention_target(sessions: &[String]) -> Option<String> {
    let (meta, _) = query_session_meta(sessions);
    sessions
        .iter()
        .find(|session| meta.get(*session).is_some_and(|m| m.attention))
        .cloned()
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
    meta_refresh_rx: Option<Receiver<MetaRefreshResult>>,
    meta_refresh_inflight: bool,
    pub(super) focused: bool,
    pub(super) notched: bool,
    pub(super) mode: SidebarMode,
    pub(super) overlay: Option<SidebarOverlay>,
    pub(super) filter: String,
    pub(super) filter_cursor: usize,
    /// Cached usage section lines rendered by `ct tui usage-bars`, refreshed on
    /// the 3s meta cycle, not every 500ms draw.
    pub(super) usage_lines_cache: Vec<String>,
    last_boot_cache_write: Instant,
    /// y-origin and height of the usage bars rect from the last draw — used
    /// to map mouse clicks to bar labels for manual pulse triggers.
    pub(super) last_bars_y: u16,
    pub(super) last_bars_h: u16,
    /// Number of tmux process spawns during the most recent refresh().
    pub(super) tmux_call_count: u32,
    /// When true, hidden sessions are included in the list.
    pub(super) show_hidden: bool,
    /// True when this sidebar pane belongs to a window that is active in an
    /// attached tmux client. Hidden panes keep their process alive, so avoid
    /// redraws/refreshes there.
    pub(super) on_screen: bool,
    pub(super) counters: instrument::SidebarCounters,
    pub(super) hidden_lifecycle: instrument::HiddenLifecycle,
}

pub(super) const ACTIVITY_GRACE: Duration = Duration::from_secs(15);
const META_REFRESH_INTERVAL: Duration = Duration::from_secs(3);

struct MetaRefreshResult {
    meta: HashMap<String, SessionMeta>,
    usage_lines: Vec<String>,
    tmux_call_count: u32,
    duration: Duration,
}

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
            // Force an immediate rich metadata worker on first refresh. It runs
            // off the UI thread so startup still paints from the cheap session
            // snapshot immediately.
            last_meta_refresh: Instant::now() - Duration::from_secs(60),
            meta_refresh_rx: None,
            meta_refresh_inflight: false,
            focused: true,
            notched: false,
            mode: SidebarMode::Browse,
            overlay: None,
            filter: String::new(),
            filter_cursor: 0,
            usage_lines_cache: Vec::new(),
            last_boot_cache_write: Instant::now() - Duration::from_secs(60),
            last_bars_y: 0,
            last_bars_h: 0,
            tmux_call_count: 0,
            show_hidden: false,
            on_screen: true,
            counters: instrument::SidebarCounters::default(),
            hidden_lifecycle: instrument::HiddenLifecycle::default(),
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

    fn hydrate_from_boot_cache(&mut self, mut cache: SidebarBootCache) {
        let cache_age = Duration::from_millis(
            now_epoch_ms()
                .saturating_sub(cache.written_at_ms)
                .min(u64::MAX as u128) as u64,
        );
        for meta in cache.meta.values_mut() {
            for agent in &mut meta.agents {
                if let Some(age) = agent.age {
                    agent.age = Some(age.saturating_add(cache_age));
                }
            }
        }
        self.current = cache.current.clone();
        self.notched = cache.notched;
        self.meta = cache.meta;
        self.usage_lines_cache = cache.usage_lines;
        self.items = build_items(&cache.sessions, &cache.current, &self.meta);
        self.rebuild_visible();
        self.snap_to_current();
    }

    fn start_meta_refresh(&mut self, sessions: Vec<String>) {
        if self.meta_refresh_inflight {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.meta_refresh_rx = Some(rx);
        self.meta_refresh_inflight = true;
        self.last_meta_refresh = Instant::now();
        std::thread::spawn(move || {
            let t0 = Instant::now();
            let (meta, tmux_call_count) = query_session_meta(&sessions);
            let usage_lines = crate::usage_bars::collect(45).lines;
            let _ = tx.send(MetaRefreshResult {
                meta,
                usage_lines,
                tmux_call_count,
                duration: t0.elapsed(),
            });
        });
    }

    fn maybe_start_meta_refresh(&mut self, sessions: &[String]) {
        if self.meta_refresh_inflight {
            return;
        }
        if self.meta.is_empty() || self.last_meta_refresh.elapsed() >= META_REFRESH_INTERVAL {
            self.start_meta_refresh(sessions.to_vec());
        }
    }

    fn drain_meta_refresh(&mut self) -> bool {
        let Some(rx) = self.meta_refresh_rx.take() else {
            return false;
        };
        match rx.try_recv() {
            Ok(result) => {
                self.meta_refresh_inflight = false;
                self.apply_meta_refresh_result(result);
                true
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.meta_refresh_rx = Some(rx);
                false
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.meta_refresh_inflight = false;
                false
            }
        }
    }

    fn apply_meta_refresh_result(&mut self, mut result: MetaRefreshResult) {
        self.tmux_call_count = self.tmux_call_count.saturating_add(result.tmux_call_count);
        self.counters
            .record_tmux_spawns(result.tmux_call_count.into());
        let now = Instant::now();
        for (session, m) in result.meta.iter_mut() {
            for agent in m.agents.iter_mut() {
                let cache_key = format!("{}:{}", session, agent.pane_id);
                // Record last_active from the RAW gerund (before cache), so the
                // timestamp freezes when the agent truly stops.
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
                // Derive age from last_active for all agents (claude gets JSONL
                // mtime in query_session_meta, overwritten here only if
                // last_active is newer).
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
        self.meta = result.meta;
        self.usage_lines_cache = result.usage_lines;
        self.last_meta_refresh = now;
        let sessions: Vec<String> = self
            .items
            .iter()
            .filter(|item| item.selectable)
            .map(|item| item.id.clone())
            .collect();
        write_boot_cache(
            &self.current,
            &sessions,
            self.notched,
            &self.meta,
            &self.usage_lines_cache,
        );
        self.last_boot_cache_write = now;
        debug!(
            duration_ms = result.duration.as_millis() as u64,
            tmux_call_count = result.tmux_call_count,
            "sidebar async metadata refresh"
        );
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
        crate::filter::fuzzy_match_borrowed(&self.items, &self.filter, |item| {
            if item.selectable {
                item.search_text.as_str()
            } else {
                ""
            }
        })
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
        self.counters.record_refresh(0);
        self.refresh_direct();
    }

    fn refresh_direct(&mut self) {
        let t0 = std::time::Instant::now();
        self.tmux_call_count = 0;

        // Batch: notched + focused client sessions + session list in one tmux
        // invocation. The sidebar's own pane can remain visible while focus is
        // in the main pane, so "#S" from this process is not a reliable
        // "current session" signal; use the most recently active tmux client.
        const DELIM: &str = "\x1e<<MUX_SIDEBAR_DIRECT_DELIM>>\x1e";
        let batch = tmux(&[
            "show-option",
            "-gv",
            "@notched",
            ";",
            "display-message",
            "-p",
            DELIM,
            ";",
            "list-clients",
            "-F",
            "#{client_activity}\t#{client_session}",
            ";",
            "display-message",
            "-p",
            DELIM,
            ";",
            "list-sessions",
            "-F",
            "#S",
        ]);
        self.tmux_call_count += 1;
        self.counters.record_tmux_spawns(1);
        let mut sections = batch.split(DELIM);
        self.notched = sections
            .next()
            .and_then(|s| s.lines().next())
            .unwrap_or("")
            .trim()
            == "1";
        let cur = sections
            .next()
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('\t');
                let activity = parts.next()?.parse::<u64>().ok()?;
                let session = parts.next()?.trim();
                (!session.is_empty()).then(|| (activity, session.to_string()))
            })
            .max_by_key(|(activity, _)| *activity)
            .map(|(_, session)| session)
            .or_else(|| (!self.current.is_empty()).then(|| self.current.clone()))
            .unwrap_or_default();
        let alive: HashSet<String> = sections
            .next()
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
        let sessions = compute_order(&alive, self.show_hidden);

        let meta_changed = self.drain_meta_refresh();
        self.maybe_start_meta_refresh(&sessions);

        let prev_id = self.items.get(self.selected).map(|i| i.id.clone());
        // External session switches (e.g. Ctrl+Tab toggling the last session)
        // should drag the cursor along, not just the "current session"
        // highlight — staying put would leave the selection orphaned on a
        // stale session.
        let current_changed = cur != self.current;

        self.items = build_items(&sessions, &cur, &self.meta);
        self.current = cur;
        self.rebuild_visible();
        if current_changed
            || meta_changed
            || self.last_boot_cache_write.elapsed() >= Duration::from_secs(5)
        {
            write_boot_cache(
                &self.current,
                &sessions,
                self.notched,
                &self.meta,
                &self.usage_lines_cache,
            );
            self.last_boot_cache_write = Instant::now();
        }

        let session_count = self.items.len() as u64;

        // When unfocused, or when the active session changed from under us,
        // track the current session.
        if !self.focused || current_changed {
            self.snap_to_current();
            debug!(
                duration_ms = t0.elapsed().as_millis() as u64,
                session_count,
                tmux_call_count = self.tmux_call_count,
                meta_changed,
                meta_inflight = self.meta_refresh_inflight,
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
                meta_changed,
                meta_inflight = self.meta_refresh_inflight,
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
            meta_changed,
            meta_inflight = self.meta_refresh_inflight,
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
            .and_then(|i| i.session_id.as_ref().map(|session| session.to_string()))
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

    fn visible_agent_animation_active(&self, list_h: u16) -> bool {
        if !self.on_screen {
            return false;
        }
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

    fn view_fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.current.hash(&mut hasher);
        self.selected.hash(&mut hasher);
        self.offset.hash(&mut hasher);
        self.focused.hash(&mut hasher);
        self.show_hidden.hash(&mut hasher);
        self.filter.hash(&mut hasher);
        self.usage_lines_cache.hash(&mut hasher);
        self.items.len().hash(&mut hasher);
        for item in &self.items {
            item.id.hash(&mut hasher);
            item.display.hash(&mut hasher);
            item.search_text.hash(&mut hasher);
            item.indent.hash(&mut hasher);
            item.selectable.hash(&mut hasher);
            item.session_id.hash(&mut hasher);
            fingerprint_item_kind(&item.kind, &mut hasher);
        }
        hasher.finish()
    }
}

fn fingerprint_item_kind(kind: &ItemKind, hasher: &mut DefaultHasher) {
    std::mem::discriminant(kind).hash(hasher);
    match kind {
        ItemKind::Session {
            diff,
            cpu_pct,
            mem_bytes,
        } => {
            diff.map(|d| (d.added, d.removed)).hash(hasher);
            cpu_pct.to_bits().hash(hasher);
            mem_bytes.hash(hasher);
        }
        ItemKind::Process(process) => {
            process.name.hash(hasher);
            process.cpu_pct.to_bits().hash(hasher);
            process.mem_bytes.hash(hasher);
        }
        ItemKind::Agent {
            name,
            age,
            gerund,
            ctx,
            asking,
        } => {
            name.hash(hasher);
            age.map(|d| d.as_secs()).hash(hasher);
            gerund.hash(hasher);
            ctx.hash(hasher);
            asking.hash(hasher);
        }
        ItemKind::Progress(pct) => pct.hash(hasher),
        ItemKind::Group | ItemKind::Status | ItemKind::Branch => {}
    }
}

// ── Main loop ────────────────────────────────────────────────

/// Exposed for `mux bench` — runs the full meta query pipeline and discards the result.
pub(crate) fn bench_query_session_meta(sessions: &[String]) {
    let _ = query_session_meta(sessions);
}

pub(crate) fn prepare_session_switch(_target: &str) {}

pub(crate) fn finish_session_switch() {}

pub(crate) fn prepare_window_switch(_target: &str) {}

pub(crate) fn finish_window_switch() {}

pub(crate) fn cmd_sidebar_control(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("toggle")
        | Some("open")
        | Some("sync")
        | Some("focus")
        | Some("prune-orphans")
        | Some("resize") => {}
        Some("close") => {}
        Some("--terminal") | Some("terminal") => cmd_sidebar_terminal(),
        Some("profile") => instrument::cmd_profile(&args[1..]),
        _ => {}
    }
}

pub(crate) fn cmd_sidebar() {
    cmd_sidebar_terminal();
}

pub(crate) fn cmd_sidebar_terminal() {
    cmd_sidebar_terminal_tui();
}

fn cmd_sidebar_terminal_tui() {
    // Set WezTerm user var for toggle detection
    // "dHJ1ZQ==" is base64("true")
    print!(
        "\x1b]1337;SetUserVar=is_sidebar=dHJ1ZQ==\x07\x1b]1337;SetUserVar=mux_sidebar_runtime={}\x07",
        TERMINAL_RUNTIME_USER_VAR
    );
    let title = std::env::var("MUX_SIDEBAR_TITLE")
        .ok()
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| TERMINAL_SIDEBAR_TITLE.to_string());
    print!("\x1b]2;{title}\x07");
    io::stdout().flush().ok();

    enter_tui();
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let mut state = SidebarState::new();
    if let Some(cache) = read_boot_cache() {
        state.hydrate_from_boot_cache(cache);
    }

    // Paint sidebar before any tmux refresh work. A fresh boot cache gives us
    // the previous fully-hydrated sidebar frame immediately; the exact tmux
    // state and rich metadata still refresh asynchronously right after this.
    // switches can make a freshly-spawned pane visible while metadata is still
    // warming; an immediate frame avoids the perception that the sidebar UI is
    // missing until the first full refresh completes.
    terminal
        .draw(|f| {
            let _ = draw(f, &mut state);
        })
        .ok();

    // On notched displays, paint this pane's terminal background black via OSC
    // 11. Use the boot cache first so startup doesn't block on an extra tmux
    // spawn; the immediate refresh below corrects the value and reapplies it.
    let mut notched_bg_applied = false;
    if state.notched {
        print!("\x1b]11;rgb:0000/0000/0000\x1b\\");
        io::stdout().flush().ok();
        notched_bg_applied = true;
    }

    state.refresh();
    if state.notched {
        print!("\x1b]11;rgb:0000/0000/0000\x1b\\");
        io::stdout().flush().ok();
        notched_bg_applied = true;
    }
    let sidebar_started = Instant::now();

    // Cache layout for mouse mapping between draws.
    let mut last_list_area = Rect::default();
    let mut last_refresh = Instant::now();
    let mut dirty = true;
    // A sidebar pane can be spawned for the target session just before tmux
    // switches the client there. During that tiny handoff window tmux can still
    // report the pane as off-screen, which sends the loop into the hidden
    // parking path before it ever draws. Paint one frame immediately so the
    // pane buffer is never blank when the client lands on it.
    if terminal
        .draw(|f| {
            last_list_area = draw(f, &mut state);
        })
        .is_ok()
    {
        let agent_animation_active = state.visible_agent_animation_active(last_list_area.height);
        state.counters.record_redraw(agent_animation_active);
        dirty = false;
    }
    const IDLE_POLL: Duration = Duration::from_millis(500);
    const ANIMATION_POLL: Duration = Duration::from_millis(33); // ~30 fps during animation
    const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

    loop {
        state
            .hidden_lifecycle
            .observe(true, sidebar_started.elapsed());

        if dirty {
            terminal
                .draw(|f| {
                    last_list_area = draw(f, &mut state);
                })
                .ok();
        }

        // High-frequency redraw while any gerund percolation is mid-flight.
        let agent_animation_active = state.visible_agent_animation_active(last_list_area.height);
        if dirty {
            state.counters.record_redraw(agent_animation_active);
            dirty = false;
        }
        let poll_timeout = if agent_animation_active || state.meta_refresh_inflight {
            ANIMATION_POLL
        } else {
            IDLE_POLL
        };
        state.counters.record_poll(if agent_animation_active {
            instrument::LoopState::ActiveAnimation
        } else {
            instrument::LoopState::VisibleIdle
        });

        if event::poll(poll_timeout).unwrap_or(false) {
            match event::read() {
                Ok(Event::FocusGained) => {
                    state.focused = true;
                    dirty = true;
                }
                Ok(Event::FocusLost) => {
                    state.focused = false;
                    state.hover = None;
                    state.close_overlay();
                    state.close_chooser();
                    state.snap_to_current();
                    dirty = true;
                }
                Ok(Event::Mouse(_)) if state.overlay_active() => {}
                Ok(Event::Mouse(me)) => match me.kind {
                    MouseEventKind::Down(MouseButton::Left)
                        if last_list_area.contains(Position {
                            x: me.column,
                            y: me.row,
                        }) =>
                    {
                        let vis_idx = state.offset + (me.row - last_list_area.y) as usize;
                        if let Some(item_idx) = state.visible.get(vis_idx).copied()
                            && let Some(sid) = state.items.get(item_idx).and_then(|i| {
                                i.selectable
                                    .then(|| i.session_id.as_ref().map(|s| s.to_string()))
                                    .flatten()
                            })
                            && let Some(row_idx) = state.items.iter().position(|i| {
                                i.selectable && i.session_id.as_deref() == Some(sid.as_str())
                            })
                        {
                            state.selected = row_idx;
                            state.switch_to_selected();
                            if state.chooser_active() {
                                state.close_chooser();
                                focus_main_pane();
                            }
                            dirty = true;
                        }
                    }
                    MouseEventKind::Moved => {
                        if last_list_area.contains(Position {
                            x: me.column,
                            y: me.row,
                        }) {
                            let vis_idx = state.offset + (me.row - last_list_area.y) as usize;
                            state.hover = state
                                .visible
                                .get(vis_idx)
                                .and_then(|idx| state.items.get(*idx))
                                .and_then(|it| {
                                    it.selectable
                                        .then(|| it.session_id.as_ref().map(|s| s.to_string()))
                                        .flatten()
                                });
                        } else {
                            state.hover = None;
                        }
                        dirty = true;
                    }
                    MouseEventKind::ScrollUp => {
                        state.move_sel(-1);
                        dirty = true;
                    }
                    MouseEventKind::ScrollDown => {
                        state.move_sel(1);
                        dirty = true;
                    }
                    _ => {}
                },
                Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                    if state.overlay_active() {
                        let handled = state.handle_overlay_key(key);
                        if handled || state.overlay_active() {
                            dirty = true;
                            continue;
                        }
                    }

                    if state.chooser_active() {
                        match (key.code, key.modifiers) {
                            (KeyCode::Esc, _) => {
                                state.close_chooser();
                                dirty = true;
                                continue;
                            }
                            (KeyCode::Enter, _) => {
                                state.switch_to_selected();
                                state.close_chooser();
                                focus_main_pane();
                                dirty = true;
                                continue;
                            }
                            (KeyCode::Char('h'), m) if m.contains(KeyModifiers::ALT) => {
                                if let Some(id) = state.selected_session_id() {
                                    toggle_hidden(&id);
                                    state.force_refresh();
                                }
                                dirty = true;
                                continue;
                            }
                            (KeyCode::Char('j'), KeyModifiers::ALT) => {
                                state.move_selected_session("down");
                                dirty = true;
                                continue;
                            }
                            (KeyCode::Char('k'), KeyModifiers::ALT) => {
                                state.move_selected_session("up");
                                dirty = true;
                                continue;
                            }
                            _ if handle_readline_key(
                                &mut state.filter,
                                &mut state.filter_cursor,
                                key,
                            ) =>
                            {
                                state.apply_filter_change();
                                dirty = true;
                                continue;
                            }
                            (KeyCode::Char('j'), _)
                            | (KeyCode::Down, _)
                            | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                                state.move_sel(1);
                                dirty = true;
                                continue;
                            }
                            (KeyCode::Char('k'), _)
                            | (KeyCode::Up, _)
                            | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                                state.move_sel(-1);
                                dirty = true;
                                continue;
                            }
                            _ => {}
                        }
                    }

                    match (key.code, key.modifiers) {
                        (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                        // Cmd+O from WezTerm arrives as Ctrl+O (see wezterm.lua
                        // focus_sidebar) — toggle to the last session.
                        (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                            tmux(&["switch-client", "-l"]);
                            finish_session_switch();
                            focus_main_pane();
                            dirty = true;
                        }
                        (KeyCode::Char('j'), KeyModifiers::ALT) => {
                            state.move_selected_session("down");
                            dirty = true;
                        }
                        (KeyCode::Char('k'), KeyModifiers::ALT) => {
                            state.move_selected_session("up");
                            dirty = true;
                        }
                        (KeyCode::Char('j'), _)
                        | (KeyCode::Down, _)
                        | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                            state.move_sel(1);
                            dirty = true;
                        }
                        (KeyCode::Char('k'), _)
                        | (KeyCode::Up, _)
                        | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                            state.move_sel(-1);
                            dirty = true;
                        }
                        (KeyCode::Enter, _) => {
                            state.switch_to_selected();
                            focus_main_pane();
                            dirty = true;
                        }
                        (KeyCode::Char('n'), m)
                            if !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                        {
                            state.open_project_overlay();
                            dirty = true;
                        }
                        (KeyCode::Char('w'), _) => {
                            state.open_worktree_overlay();
                            dirty = true;
                        }
                        (KeyCode::Char('r'), _) => {
                            state.open_rename_overlay();
                            dirty = true;
                        }
                        (KeyCode::Char('x'), _) => {
                            state.open_ditch_overlay();
                            dirty = true;
                        }
                        (KeyCode::Char('h'), m) if m.contains(KeyModifiers::ALT) => {
                            if let Some(id) = state.selected_session_id() {
                                toggle_hidden(&id);
                                state.force_refresh();
                            }
                            dirty = true;
                        }
                        (KeyCode::Char('h'), _) => {
                            state.show_hidden = !state.show_hidden;
                            dirty = true;
                        }
                        (KeyCode::Char('/'), _) => {
                            state.open_chooser();
                            dirty = true;
                        }
                        (KeyCode::Char(c @ '1'..='9'), m)
                            if !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                        {
                            state.select_by_number(c);
                            dirty = true;
                        }
                        (KeyCode::Char(c), m)
                            if !m.intersects(
                                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                            ) =>
                        {
                            forward_char_to_main(c);
                            dirty = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        dirty |= state.drain_meta_refresh();

        // Throttle refresh to IDLE cadence — pulse-driven high-fps redraws
        // shouldn't multiply tmux process spawns.
        if last_refresh.elapsed() >= REFRESH_INTERVAL {
            let before = state.view_fingerprint();
            state.refresh();
            dirty |= state.view_fingerprint() != before;
            last_refresh = Instant::now();
        }
        dirty |= agent_animation_active;
    }

    leave_tui();

    // Reset pane background (OSC 111) to whatever the user's theme defines.
    if notched_bg_applied {
        print!("\x1b]111\x1b\\");
        io::stdout().flush().ok();
    }
}

pub(crate) fn cmd_hook() {
    hooks::ingest_stdin();
}

#[cfg(test)]
mod runtime_tests {
    use super::TERMINAL_RUNTIME_USER_VAR;

    #[test]
    fn runtime_user_var_identifies_terminal_hosting_model() {
        assert_eq!(TERMINAL_RUNTIME_USER_VAR, "dGVybWluYWw=");
    }

    #[test]
    fn applescript_string_literals_escape_input() {
        assert_eq!(
            super::applescript_string_literal(r#"a "quote" and \ slash"#),
            r#""a \"quote\" and \\ slash""#
        );
    }
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
    if terminal_sidebar_host_is("ghostty") {
        focus_ghostty_main_split();
        return;
    }

    let mut command = Command::new("wezterm");
    command
        .args(["cli", "activate-pane-direction", "Right"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _ = spawn_reaped(command);
}

fn terminal_sidebar_host_is(host: &str) -> bool {
    std::env::var("MUX_SIDEBAR_HOST").is_ok_and(|value| value.eq_ignore_ascii_case(host))
}

fn focus_ghostty_main_split() {
    run_ghostty_applescript(
        r#"tell application "Ghostty"
    set term to focused terminal of selected tab of front window
    perform action "goto_split:right" on term
end tell"#,
    );
}

fn applescript_string_literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn run_ghostty_applescript(script: &str) {
    let _ = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Bounce an accidental keystroke to the neighbouring pane so typing `ls` in
/// the sidebar by mistake still lands where the user expected.
fn forward_char_to_main(c: char) {
    if terminal_sidebar_host_is("ghostty") {
        let mut buf = [0u8; 4];
        let text = applescript_string_literal(c.encode_utf8(&mut buf));
        run_ghostty_applescript(&format!(
            r#"tell application "Ghostty"
    set term to focused terminal of selected tab of front window
    perform action "goto_split:right" on term
    set targetTerm to focused terminal of selected tab of front window
    input text {text} to targetTerm
end tell"#
        ));
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
