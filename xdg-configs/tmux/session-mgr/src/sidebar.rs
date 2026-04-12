use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{self, Stdout, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crossterm::cursor;
use crossterm::event::{
    self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
    KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::color::{compute_color, is_static};
use crate::group::{GroupMeta, session_group, session_suffix};
use crate::order::compute_order;
use crate::tmux::tmux;
use crate::usage_graph;

// Catppuccin Mocha
const MANTLE: Color = Color::Rgb(0x11, 0x11, 0x1b);
const BASE: Color = Color::Rgb(0x15, 0x15, 0x20);
const SURFACE0: Color = Color::Rgb(0x1c, 0x1c, 0x29);
const SURFACE1: Color = Color::Rgb(0x45, 0x47, 0x5a);
const OVERLAY0: Color = Color::Rgb(0x6c, 0x70, 0x86);
const SUBTEXT0: Color = Color::Rgb(0xa6, 0xad, 0xc8);
const TEXT: Color = Color::Rgb(0xcd, 0xd6, 0xf4);
const PEACH: Color = Color::Rgb(0xfa, 0xb3, 0x87);
const BLUE: Color = Color::Rgb(0x89, 0xb4, 0xfa);
const MAUVE: Color = Color::Rgb(0xcb, 0xa6, 0xf7);
const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);

const NUM_GLYPHS: &[char] = &[
    '\u{F03A6}',
    '\u{F03A9}',
    '\u{F03AC}',
    '\u{F03AE}',
    '\u{F03B0}',
    '\u{F03B5}',
    '\u{F03B8}',
    '\u{F03BB}',
    '\u{F03BE}',
    '\u{F03C1}',
];
const NUM_SELECTED: &[char] = &[
    '\u{F03A4}',
    '\u{F03A7}',
    '\u{F03AA}',
    '\u{F03AD}',
    '\u{F03B1}',
    '\u{F03B3}',
    '\u{F03B6}',
    '\u{F03B9}',
    '\u{F03BC}',
    '\u{F03BF}',
];
const GROUP_GLYPHS: &[char] = &[
    '\u{F03A5}',
    '\u{F03A8}',
    '\u{F03AB}',
    '\u{F03B2}',
    '\u{F03AF}',
    '\u{F03B4}',
    '\u{F03B7}',
    '\u{F03BA}',
    '\u{F03BD}',
    '\u{F03C0}',
];

fn num_glyph(idx: usize, selected: bool) -> char {
    let table = if selected { NUM_SELECTED } else { NUM_GLYPHS };
    *table.get(idx).unwrap_or(&table[table.len() - 1])
}

fn group_glyph(count: usize) -> char {
    let idx = count.clamp(1, GROUP_GLYPHS.len()) - 1;
    GROUP_GLYPHS[idx]
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color::Rgb(r, g, b)
    } else {
        Color::Rgb(0x89, 0xb4, 0xfa)
    }
}

fn truncate(s: &str, max: usize) -> String {
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

fn agent_color(name: &str) -> Color {
    AGENTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| *c)
        .unwrap_or(SUBTEXT0)
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

// ── Claude context % via pane statusline scraping ─────────────
// Claude Code's statusline is rendered right in the pane. We capture
// the last lines via `tmux capture-pane` and parse the segmented digit
// + ٪ (U+066A ARABIC PERCENT SIGN) pattern emitted by statusline.py.
// The FIRST ٪ in the statusline is the context percentage.

#[derive(Clone, Default)]
struct ClaudeCtx {
    pct: u8,
    tokens: String, // e.g. "348k/1.0m"
}

#[derive(Clone, Default)]
struct ClaudeScrape {
    ctx: Option<ClaudeCtx>,
    activity: Option<String>,
}

fn query_claude_scrapes(claude_sessions: &[String]) -> HashMap<String, ClaudeScrape> {
    let mut result = HashMap::new();
    for session in claude_sessions {
        let raw = tmux(&["capture-pane", "-t", session, "-p", "-S", "-30"]);
        let scrape = ClaudeScrape {
            ctx: parse_context(&raw),
            activity: parse_activity(&raw),
        };
        if scrape.ctx.is_some() || scrape.activity.is_some() {
            result.insert(session.clone(), scrape);
        }
    }
    result
}

/// Match Claude Code's activity lines like:
///   · Ebbing… (6m 50s · ↑ 4.1k tokens · thought for 1s)
///   ✻ Baked for 1m 16s
fn parse_activity(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        let mut chars = trimmed.chars();
        let Some(first) = chars.next() else {
            continue;
        };
        // Known Claude Code activity glyphs
        if !matches!(
            first,
            '\u{00B7}'  // ·
            | '\u{2022}'  // •
            | '\u{273B}'  // ✻
            | '\u{22C6}'  // ⋆
            | '\u{2726}'  // ✦
            | '\u{2727}'  // ✧
            | '\u{2736}'  // ✶
        ) {
            continue;
        }
        if chars.next() != Some(' ') {
            continue;
        }
        if trimmed.chars().count() < 5 {
            continue;
        }
        return Some(trimmed.to_string());
    }
    None
}

fn segdigit_value(c: char) -> Option<u32> {
    let n = c as u32;
    if (0x1FBF0..=0x1FBF9).contains(&n) {
        Some(n - 0x1FBF0)
    } else {
        None
    }
}

fn parse_context(text: &str) -> Option<ClaudeCtx> {
    for line in text.lines() {
        let Some(pct_pos) = line.find('\u{066A}') else {
            continue;
        };

        // Walk back from ٪ to get segmented digits → pct
        let before = &line[..pct_pos];
        let mut digits: Vec<u32> = Vec::new();
        for c in before.chars().rev() {
            match segdigit_value(c) {
                Some(d) => digits.push(d),
                None => {
                    if !digits.is_empty() {
                        break;
                    }
                }
            }
        }
        if digits.is_empty() {
            continue;
        }
        digits.reverse();
        let pct = (digits.iter().fold(0u32, |a, d| a * 10 + d)).min(100) as u8;

        // After ٪, the next whitespace-delimited token like "348k/1.0m"
        let after = &line[pct_pos + '\u{066A}'.len_utf8()..];
        let tokens = after
            .split_whitespace()
            .find(|t| t.contains('/') && t.bytes().any(|b| b.is_ascii_digit()))
            .unwrap_or("")
            .to_string();

        return Some(ClaudeCtx { pct, tokens });
    }
    None
}

// Position-based gradient colors matching statusline.py's context_bar.
// xterm-256: 65=#5f875f 114=#87d787 130=#af5f00 215=#ffaf5f 131=#af5f5f 203=#ff5f5f 242=#6c6c6c
const CTX_POS_COLORS: [Color; 12] = [
    Color::Rgb(0x5f, 0x87, 0x5f), // 0: dim green
    Color::Rgb(0x5f, 0x87, 0x5f), // 1
    Color::Rgb(0x87, 0xd7, 0x87), // 2: green
    Color::Rgb(0x87, 0xd7, 0x87), // 3
    Color::Rgb(0xaf, 0x5f, 0x00), // 4: dim orange
    Color::Rgb(0xaf, 0x5f, 0x00), // 5
    Color::Rgb(0xff, 0xaf, 0x5f), // 6: orange
    Color::Rgb(0xff, 0xaf, 0x5f), // 7
    Color::Rgb(0xff, 0xaf, 0x5f), // 8
    Color::Rgb(0xaf, 0x5f, 0x5f), // 9: dim red
    Color::Rgb(0xff, 0x5f, 0x5f), // 10: red
    Color::Rgb(0xff, 0x5f, 0x5f), // 11
];
const CTX_EMPTY_COLOR: Color = Color::Rgb(0x6c, 0x6c, 0x6c);

fn seg_digit(n: u32) -> char {
    char::from_u32(0x1FBF0 + n.min(9)).unwrap_or('0')
}

fn seg_number(n: u32) -> String {
    if n == 0 {
        return seg_digit(0).to_string();
    }
    let mut digits = Vec::new();
    let mut x = n;
    while x > 0 {
        digits.push(x % 10);
        x /= 10;
    }
    digits.iter().rev().map(|&d| seg_digit(d)).collect()
}

const TREE_COLOR: Color = Color::Rgb(0x2e, 0x2f, 0x40);

fn tree_prefix_spans(tree: Tree, indent: usize, row_bg: Color) -> Vec<Span<'static>> {
    let tree_style = Style::default().fg(TREE_COLOR).bg(row_bg);
    let space_style = Style::default().bg(row_bg);
    let (glyph, tail) = match tree {
        Tree::None | Tree::Blank => return vec![Span::styled(" ".repeat(indent), space_style)],
        Tree::Middle => ("\u{251C}", indent.saturating_sub(1)),
        Tree::Last => ("\u{2514}", indent.saturating_sub(1)),
        Tree::Pipe => ("\u{2502}", indent.saturating_sub(1)),
    };
    vec![
        Span::styled(glyph, tree_style),
        Span::styled(" ".repeat(tail), space_style),
    ]
}

/// Darken a color toward MANTLE (~40% brightness) for unfocused sessions.
fn dim_color(c: Color) -> Color {
    let Color::Rgb(r, g, b) = c else { return c };
    let scale = |v: u8| ((v as u32 * 40) / 100) as u8;
    Color::Rgb(scale(r), scale(g), scale(b))
}

fn ctx_label_color(pct: u8) -> Color {
    let full = (pct as usize * 12) / 100;
    if full >= 7 {
        Color::Rgb(0xff, 0x5f, 0x5f)
    } else if full >= 3 {
        Color::Rgb(0xff, 0xaf, 0x5f)
    } else {
        Color::Rgb(0x87, 0xd7, 0x87)
    }
}

// ── Claude activity age ──────────────────────────────────────
// Claude appends to `~/.claude/projects/{cwd-encoded}/{session_id}.jsonl`
// on every message. mtime of the most recent jsonl ≈ last activity.

fn query_claude_ages(
    claude_sessions: &[String],
    cwds: &HashMap<String, String>,
) -> HashMap<String, Duration> {
    let home = std::env::var("HOME").unwrap_or_default();
    let projects_root = std::path::PathBuf::from(&home).join(".claude/projects");
    let now = std::time::SystemTime::now();

    let mut result = HashMap::new();
    for session in claude_sessions {
        let Some(cwd) = cwds.get(session) else {
            continue;
        };
        let dir_name = cwd.replace('/', "-").replace('.', "-");
        let project_dir = projects_root.join(&dir_name);
        let Some(mtime) = most_recent_jsonl_mtime(&project_dir) else {
            continue;
        };
        if let Ok(elapsed) = now.duration_since(mtime) {
            result.insert(session.clone(), elapsed);
        }
    }
    result
}

fn most_recent_jsonl_mtime(dir: &std::path::Path) -> Option<std::time::SystemTime> {
    let mut best: Option<std::time::SystemTime> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let Ok(mtime) = entry.metadata().and_then(|m| m.modified()) else {
            continue;
        };
        best = Some(match best {
            None => mtime,
            Some(t) if mtime > t => mtime,
            Some(t) => t,
        });
    }
    best
}

fn format_age(d: Duration) -> String {
    let s = d.as_secs();
    if s < 60 {
        "<1m".to_string()
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else {
        ">1h".to_string()
    }
}

fn age_color(d: Duration) -> Color {
    let s = d.as_secs();
    if s < 300 {
        Color::Rgb(0xa6, 0xe3, 0xa1) // green
    } else if s < 3600 {
        Color::Rgb(0x89, 0xb4, 0xfa) // blue
    } else {
        Color::Rgb(0xf3, 0x8b, 0xa8) // red
    }
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
struct SessionMeta {
    branch: String,
    agent: String,
    claude_ctx: Option<ClaudeCtx>,
    claude_age: Option<Duration>,
    claude_activity: Option<String>,
    attention: bool,
    ports: Vec<u16>,
    status: String,
    progress: Option<u8>,
}

fn query_session_meta(sessions: &[String]) -> HashMap<String, SessionMeta> {
    // Pane info (cwd + pid for active pane of each session)
    let pane_raw = tmux(&[
        "list-panes",
        "-a",
        "-F",
        "#{session_name}\t#{window_active}\t#{pane_active}\t#{pane_current_path}\t#{pane_pid}",
    ]);
    let mut cwds: HashMap<String, String> = HashMap::new();
    let mut pane_pids: HashMap<String, u32> = HashMap::new();
    for line in pane_raw.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() == 5 && parts[1] == "1" && parts[2] == "1" {
            cwds.insert(parts[0].to_string(), parts[3].to_string());
            if let Ok(pid) = parts[4].parse::<u32>() {
                pane_pids.insert(parts[0].to_string(), pid);
            }
        }
    }

    // Session options
    let opts_raw = tmux(&[
        "list-sessions",
        "-F",
        "#{session_name}\t#{@attention}\t#{@sidebar_status}\t#{@sidebar_progress}",
    ]);
    let mut attn: HashMap<String, bool> = HashMap::new();
    let mut statuses: HashMap<String, String> = HashMap::new();
    let mut progresses: HashMap<String, u8> = HashMap::new();
    for line in opts_raw.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
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

    // Shared process info (used for agents + ports)
    let (parent_of, name_of) = build_process_info();

    // Agents
    let agents_map = query_agents(&pane_pids, &parent_of, &name_of);

    // Claude context %: scrape statusline from claude sessions' panes
    let claude_sessions: Vec<String> = agents_map
        .iter()
        .filter(|(_, name)| name.as_str() == "claude")
        .map(|(s, _)| s.clone())
        .collect();
    let claude_scrape_map = query_claude_scrapes(&claude_sessions);
    let claude_age_map = query_claude_ages(&claude_sessions, &cwds);

    // Git branches (one call per unique cwd)
    let mut branch_cache: HashMap<String, String> = HashMap::new();
    for cwd in cwds.values() {
        if !cwd.is_empty() && !branch_cache.contains_key(cwd) {
            branch_cache.insert(cwd.clone(), git_branch(cwd));
        }
    }

    // Ports
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
                claude_activity: claude_scrape_map
                    .get(name)
                    .and_then(|s| s.activity.clone()),
                attention: *attn.get(name).unwrap_or(&false),
                ports: ports_map.get(name).cloned().unwrap_or_default(),
                status: statuses.get(name).cloned().unwrap_or_default(),
                progress: progresses.get(name).copied(),
            },
        );
    }
    result
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

// ── Items ────────────────────────────────────────────────────

enum ItemKind {
    Session { attention: bool },
    Group,
    Branch,
    Agent { name: String, age: Option<Duration> },
    Activity(String),
    ContextBar { pct: u8, tokens: String },
    Ports(Vec<u16>),
    Status,
    Progress(u8),
}

#[derive(Clone, Copy)]
enum Tree {
    None,
    Middle, // ├ (session, not last in group)
    Last,   // └ (session, last in group)
    Pipe,   // │ (detail under non-last session)
    Blank,  // spaces (detail under last session)
}

struct Item {
    id: String,
    display: String,
    indent: u16,
    tree: Tree,
    color: Color,
    dim_color: Color,
    selectable: bool,
    session_id: Option<String>,
    kind: ItemKind,
}

// ── State ────────────────────────────────────────────────────

struct SidebarState {
    items: Vec<Item>,
    current: String,
    selected: usize,
    offset: usize,
    hover: Option<String>,
    meta: HashMap<String, SessionMeta>,
    /// Sticky cache for Claude's activity line so it doesn't flicker between
    /// refreshes. Cleared when the entry is older than ACTIVITY_GRACE.
    activity_cache: HashMap<String, (String, Instant)>,
    last_meta_refresh: Instant,
    focused: bool,
    notched: bool,
}

const ACTIVITY_GRACE: Duration = Duration::from_secs(15);

impl SidebarState {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            current: String::new(),
            selected: 0,
            offset: 0,
            hover: None,
            meta: HashMap::new(),
            activity_cache: HashMap::new(),
            last_meta_refresh: Instant::now() - Duration::from_secs(60),
            focused: true,
            notched: false,
        }
    }

    fn refresh(&mut self) {
        self.notched = tmux(&["show-option", "-gv", "@notched"]) == "1";
        let cur = tmux(&["display-message", "-p", "#S"]);
        let alive: HashSet<String> = tmux(&["list-sessions", "-F", "#S"])
            .lines()
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
        let sessions = compute_order(&alive, false);

        if self.last_meta_refresh.elapsed() >= Duration::from_secs(3) {
            let mut meta = query_session_meta(&sessions);
            let now = Instant::now();
            for (session, m) in meta.iter_mut() {
                match &m.claude_activity {
                    Some(act) => {
                        self.activity_cache
                            .insert(session.clone(), (act.clone(), now));
                    }
                    None => {
                        if let Some((cached, t)) = self.activity_cache.get(session)
                            && now.duration_since(*t) < ACTIVITY_GRACE
                        {
                            m.claude_activity = Some(cached.clone());
                        }
                    }
                }
            }
            self.activity_cache
                .retain(|_, (_, t)| now.duration_since(*t) < ACTIVITY_GRACE);
            self.meta = meta;
            self.last_meta_refresh = now;
        }

        let prev_id = self.items.get(self.selected).map(|i| i.id.clone());

        self.items = build_items(&sessions, &cur, &self.meta);
        self.current = cur;

        // When unfocused, always track current session
        if !self.focused {
            self.snap_to_current();
            return;
        }

        if let Some(ref id) = prev_id
            && let Some(pos) = self.items.iter().position(|i| i.id == *id)
        {
            self.selected = pos;
            return;
        }
        self.snap_to_current();
    }

    fn snap_to_current(&mut self) {
        self.selected = self
            .items
            .iter()
            .position(|i| i.selectable && i.id == self.current)
            .unwrap_or(0);
    }

    fn move_sel(&mut self, dir: i32) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let mut pos = self.selected;
        loop {
            if dir > 0 {
                if pos + 1 >= len {
                    return;
                }
                pos += 1;
            } else {
                if pos == 0 {
                    return;
                }
                pos -= 1;
            }
            if self.items[pos].selectable {
                self.selected = pos;
                return;
            }
        }
    }

    fn selected_session_id(&self) -> Option<String> {
        self.items
            .get(self.selected)
            .and_then(|i| i.session_id.clone())
    }

    fn switch_to_selected(&self) {
        if let Some(id) = self.selected_session_id() {
            tmux(&["switch-client", "-t", &id]);
        }
    }

    fn select_by_number(&mut self, c: char) {
        let n = (c as usize) - ('1' as usize);
        let selectable: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.selectable)
            .map(|(idx, _)| idx)
            .collect();
        if let Some(&idx) = selectable.get(n) {
            self.selected = idx;
            self.switch_to_selected();
        }
    }
}

fn build_items(sessions: &[String], cur: &str, meta: &HashMap<String, SessionMeta>) -> Vec<Item> {
    let group_meta = GroupMeta::new(sessions);

    let mut gpos_counter: HashMap<&str, usize> = HashMap::new();
    let mut orphan_idx = 0usize;
    let mut session_colors: Vec<(Color, Color)> = Vec::new();

    for name in sessions {
        let group = session_group(name);
        let gtotal = if group.is_empty() {
            0
        } else {
            *group_meta.counts.get(group).unwrap_or(&0)
        };

        let (color_hex, dim_hex) = if is_static(name) {
            compute_color(name, 0, 0, 0, 0)
        } else if !group.is_empty() {
            let gpos = *gpos_counter.get(group).unwrap_or(&0);
            let gidx = *group_meta.group_idx.get(group).unwrap_or(&0);
            let r = compute_color(name, gidx, group_meta.dynamic_total, gpos, gtotal);
            *gpos_counter.entry(group).or_default() += 1;
            r
        } else {
            let r = compute_color(
                name,
                group_meta.dynamic_groups + orphan_idx,
                group_meta.dynamic_total,
                0,
                0,
            );
            orphan_idx += 1;
            r
        };

        session_colors.push((hex_to_color(&color_hex), hex_to_color(&dim_hex)));
    }

    let empty_meta = SessionMeta::default();
    let mut items = Vec::new();
    let mut idx = 0usize;
    let mut last_group = String::new();

    for (i, name) in sessions.iter().enumerate() {
        let group = session_group(name);
        let gtotal = if group.is_empty() {
            0
        } else {
            *group_meta.counts.get(group).unwrap_or(&0)
        };
        let (color, dim_color) = session_colors[i];
        let sm = meta.get(name).unwrap_or(&empty_meta);

        let is_grouped = !group.is_empty() && gtotal > 1;
        let is_last_in_group = is_grouped
            && sessions.get(i + 1).map(|n| session_group(n)) != Some(group);
        let session_tree = if !is_grouped {
            Tree::None
        } else if is_last_in_group {
            Tree::Last
        } else {
            Tree::Middle
        };
        let detail_tree = if !is_grouped {
            Tree::None
        } else if is_last_in_group {
            Tree::Blank
        } else {
            Tree::Pipe
        };

        // Grouped session
        let (session_display, session_indent, detail_indent) = if is_grouped {
            if group != last_group {
                let gg = group_glyph(gtotal);
                items.push(Item {
                    id: format!("__group__{group}"),
                    display: format!("{gg} {group}"),
                    indent: 0,
                    tree: Tree::None,
                    color,
                    dim_color,
                    selectable: false,
                    session_id: None,
                    kind: ItemKind::Group,
                });
            }
            let suffix = {
                let s = session_suffix(name);
                if s.is_empty() {
                    group.to_string()
                } else {
                    s.to_string()
                }
            };
            let glyph = num_glyph(idx, name == cur);
            idx += 1;
            (format!("{glyph} {suffix}"), 2u16, 4u16)
        } else {
            let flat = if !group.is_empty() {
                group
            } else {
                name.as_str()
            };
            let glyph = num_glyph(idx, name == cur);
            idx += 1;
            (format!("{glyph} {flat}"), 0u16, 2u16)
        };

        items.push(Item {
            id: name.clone(),
            display: session_display,
            indent: session_indent,
            tree: session_tree,
            color,
            dim_color,
            selectable: true,
            session_id: Some(name.clone()),
            kind: ItemKind::Session {
                attention: sm.attention,
            },
        });

        // Detail rows (all indented to align after number glyph)
        if !sm.agent.is_empty() {
            items.push(Item {
                id: format!("__agent__{name}"),
                display: sm.agent.clone(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Agent {
                    name: sm.agent.clone(),
                    age: sm.claude_age,
                },
            });

            if let Some(act) = &sm.claude_activity {
                items.push(Item {
                    id: format!("__activity__{name}"),
                    display: act.clone(),
                    indent: detail_indent,
                    tree: detail_tree,
                    color,
                    dim_color,
                    selectable: false,
                    session_id: Some(name.clone()),
                    kind: ItemKind::Activity(act.clone()),
                });
            }

            if let Some(ctx) = &sm.claude_ctx {
                items.push(Item {
                    id: format!("__ctx__{name}"),
                    display: String::new(),
                    indent: detail_indent,
                    tree: detail_tree,
                    color,
                    dim_color,
                    selectable: false,
                    session_id: Some(name.clone()),
                    kind: ItemKind::ContextBar {
                        pct: ctx.pct,
                        tokens: ctx.tokens.clone(),
                    },
                });
            }
        }
        if !sm.branch.is_empty() {
            items.push(Item {
                id: format!("__branch__{name}"),
                display: sm.branch.clone(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Branch,
            });
        }
        if !sm.ports.is_empty() {
            items.push(Item {
                id: format!("__ports__{name}"),
                display: String::new(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Ports(sm.ports.clone()),
            });
        }
        if !sm.status.is_empty() {
            items.push(Item {
                id: format!("__status__{name}"),
                display: sm.status.clone(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Status,
            });
        }
        if let Some(pct) = sm.progress {
            items.push(Item {
                id: format!("__progress__{name}"),
                display: String::new(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Progress(pct),
            });
        }

        last_group = group.to_string();
    }

    items
}

// ── Main loop ────────────────────────────────────────────────

pub fn cmd_sidebar() {
    // Set WezTerm user var for toggle detection
    // "dHJ1ZQ==" is base64("true")
    print!("\x1b]1337;SetUserVar=is_sidebar=dHJ1ZQ==\x07");
    io::stdout().flush().ok();

    // Tell tmux status bar to hide the session list while sidebar is open
    tmux(&["set-option", "-g", "@sidebar_open", "1"]);
    refresh_status_bar();

    enter_tui();
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let mut state = SidebarState::new();
    state.refresh();

    // Cache layout for mouse-click mapping between draws
    let mut last_list_y: u16 = 0;
    let mut last_list_h: u16 = 0;

    loop {
        terminal
            .draw(|f| {
                let (ly, lh) = draw(f, &mut state);
                last_list_y = ly;
                last_list_h = lh;
            })
            .ok();

        if event::poll(Duration::from_millis(500)).unwrap_or(false) {
            match event::read() {
                Ok(Event::FocusGained) => {
                    state.focused = true;
                }
                Ok(Event::FocusLost) => {
                    state.focused = false;
                    state.hover = None;
                    state.snap_to_current();
                }
                Ok(Event::Mouse(me)) => match me.kind {
                    MouseEventKind::Down(MouseButton::Left)
                        if me.row >= last_list_y && me.row < last_list_y + last_list_h =>
                    {
                        let idx = state.offset + (me.row - last_list_y) as usize;
                        if let Some(sid) =
                            state.items.get(idx).and_then(|i| i.session_id.clone())
                            && let Some(row_idx) = state
                                .items
                                .iter()
                                .position(|i| i.selectable && i.session_id.as_ref() == Some(&sid))
                        {
                            state.selected = row_idx;
                            state.switch_to_selected();
                        }
                    }
                    MouseEventKind::Moved => {
                        if me.row >= last_list_y && me.row < last_list_y + last_list_h {
                            let idx = state.offset + (me.row - last_list_y) as usize;
                            state.hover = state
                                .items
                                .get(idx)
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
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _)
                        | (KeyCode::Esc, _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
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
                            spawn_subcmd(&mut terminal, &["new-session"]);
                            state.last_meta_refresh = Instant::now() - Duration::from_secs(60);
                        }
                        (KeyCode::Char('w'), _) => {
                            spawn_subcmd(&mut terminal, &["new-worktree"]);
                            state.last_meta_refresh = Instant::now() - Duration::from_secs(60);
                        }
                        (KeyCode::Char('r'), _) => {
                            spawn_subcmd(&mut terminal, &["rename"]);
                        }
                        (KeyCode::Char('x'), _) => {
                            spawn_subcmd(&mut terminal, &["ditch"]);
                        }
                        (KeyCode::Char('h'), _) => {
                            if let Some(id) = state.selected_session_id() {
                                spawn_subcmd(&mut terminal, &["hide-toggle", &id]);
                            }
                        }
                        (KeyCode::Char('/'), _) => {
                            spawn_subcmd(&mut terminal, &["chooser"]);
                            focus_main_pane();
                        }
                        (KeyCode::Char(c @ '1'..='9'), m)
                            if !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                        {
                            state.select_by_number(c);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        state.refresh();
    }

    leave_tui();

    // Restore status bar
    tmux(&["set-option", "-gu", "@sidebar_open"]);
    refresh_status_bar();
}

fn refresh_status_bar() {
    let exe = std::env::current_exe().unwrap_or_else(|_| "tmux-session".into());
    let _ = Command::new(&exe)
        .args(["update"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
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

fn focus_main_pane() {
    let _ = Command::new("wezterm")
        .args(["cli", "activate-pane-direction", "Right"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn spawn_subcmd(terminal: &mut Terminal<CrosstermBackend<Stdout>>, args: &[&str]) {
    leave_tui();
    let exe = std::env::current_exe().unwrap_or_else(|_| "tmux-session".into());
    let _ = Command::new(&exe).args(args).status();
    enter_tui();
    terminal.clear().ok();
}

// ── Drawing ──────────────────────────────────────────────────

/// Returns (list_y_start, list_height) for mouse mapping.
fn draw(f: &mut Frame, state: &mut SidebarState) -> (u16, u16) {
    let area = f.area();
    f.render_widget(Clear, area);

    let bg = MANTLE;

    if area.height < 3 || area.width < 6 {
        return (0, 0);
    }

    // On notched displays, reserve an extra solid-black row at the very top to
    // match the tmux filler row that hides behind the display cutout.
    let notch_h: u16 = if state.notched { 1 } else { 0 };

    // Fill background
    for y in area.y + notch_h..area.y + area.height {
        let row = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        f.render_widget(Paragraph::new("").style(Style::default().bg(bg)), row);
    }
    if notch_h > 0 {
        let notch_row = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: notch_h,
        };
        f.render_widget(
            Paragraph::new("").style(Style::default().bg(Color::Rgb(0, 0, 0))),
            notch_row,
        );
    }

    let content_w = area.width;

    // Layout:
    //  (optional) notch row (black, hidden behind notch)
    //  row 0: blank (top padding)
    //  list rows
    //  usage graph (if there's room)
    //  2 footer rows
    let list_y = area.y + notch_h + 1;
    let footer_h = 2u16;
    let graph_h = if area.height >= 4 + 1 + notch_h + usage_graph::HEIGHT + footer_h {
        usage_graph::HEIGHT
    } else {
        0
    };
    let list_h = area.height.saturating_sub(1 + notch_h + footer_h + graph_h);

    // Footer hints
    let hint1_y = area.y + area.height - 2;
    let hint2_y = area.y + area.height - 1;
    let (hint1, hint2) = footer_hints(content_w as usize);
    render_at(f, area.x, hint1_y, content_w, hint1, bg);
    render_at(f, area.x, hint2_y, content_w, hint2, bg);

    if graph_h > 0 {
        let graph_rect = Rect {
            x: area.x,
            y: list_y + list_h,
            width: content_w,
            height: graph_h,
        };
        usage_graph::draw(f, graph_rect, bg);
    }

    if list_h == 0 {
        return (list_y, 0);
    }

    let list_w_with_bar = content_w.saturating_sub(1); // right pad
    let total = state.items.len();
    let list_height = list_h as usize;

    // Scroll
    if state.selected < state.offset {
        state.offset = state.selected;
    }
    if state.selected >= state.offset + list_height {
        state.offset = state.selected - list_height + 1;
    }

    let selected_session: Option<String> = state
        .items
        .get(state.selected)
        .and_then(|i| i.session_id.clone());

    for vi in 0..list_height.min(total.saturating_sub(state.offset)) {
        let item_idx = state.offset + vi;
        let item = &state.items[item_idx];
        let is_sel = item.session_id.is_some() && item.session_id == selected_session;
        let is_hover = item.session_id.is_some() && item.session_id == state.hover;
        let belongs_to_current = item.session_id.as_deref() == Some(state.current.as_str());
        let row = Rect {
            x: area.x,
            y: list_y + vi as u16,
            width: list_w_with_bar,
            height: 1,
        };

        render_item(f, row, item, is_sel, is_hover, belongs_to_current, bg);
    }

    // Scrollbar
    if total > list_height {
        let sb_area = Rect {
            x: area.x + list_w_with_bar,
            y: list_y,
            width: 1,
            height: list_h,
        };
        let mut sb = ScrollbarState::new(total.saturating_sub(list_height)).position(state.offset);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some(" "))
                .track_style(Style::default().bg(bg))
                .thumb_style(Style::default().fg(SURFACE1)),
            sb_area,
            &mut sb,
        );
    }

    (list_y, list_h)
}

fn bar_span<'a>(item: &'a Item, is_sel: bool, row_bg: Color) -> Span<'a> {
    if is_sel {
        Span::styled("▌", Style::default().fg(item.color).bg(row_bg))
    } else {
        Span::styled(" ", Style::default().bg(row_bg))
    }
}

fn render_item(
    f: &mut Frame,
    row: Rect,
    item: &Item,
    is_sel: bool,
    is_hover: bool,
    is_cur: bool,
    bg: Color,
) {
    let w = row.width as usize;
    if w == 0 {
        return;
    }

    // Selection bar takes col 0; content starts at col 1
    let bar_w = 1usize;
    let indent = item.indent as usize;
    let content_w = w.saturating_sub(bar_w + indent);

    // Background priority: selected > hover > current-session > default
    const HOVER_BG: Color = Color::Rgb(0x28, 0x29, 0x3a);
    let row_bg = if is_sel {
        SURFACE0
    } else if is_hover {
        HOVER_BG
    } else if is_cur {
        BASE
    } else {
        bg
    };

    match &item.kind {
        ItemKind::Group => {
            let disp = truncate(&item.display, content_w);
            let mut line: Vec<Span<'_>> =
                vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            line.push(Span::styled(
                disp,
                Style::default().fg(OVERLAY0).bold().bg(row_bg),
            ));
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Session { attention } => {
            let fg = if is_sel || is_cur {
                item.color
            } else {
                item.dim_color
            };

            let mut spans: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
            spans.extend(tree_prefix_spans(item.tree, indent, row_bg));

            let mut reserved = 0usize;
            if *attention {
                reserved += 2;
            }
            if is_cur {
                reserved += 2;
            }
            let name_w = content_w.saturating_sub(reserved);
            let name = truncate(&item.display, name_w);

            let name_style = if is_cur {
                Style::default().fg(fg).bold().bg(row_bg)
            } else {
                Style::default().fg(fg).bg(row_bg)
            };
            spans.push(Span::styled(name, name_style));

            let used: usize = spans.iter().skip(1).map(|s| s.width()).sum();
            let pad = (w - bar_w).saturating_sub(used + reserved);
            if pad > 0 {
                spans.push(Span::styled(" ".repeat(pad), Style::default().bg(row_bg)));
            }

            if *attention {
                spans.push(Span::styled(
                    "●",
                    Style::default().fg(item.color).bg(row_bg),
                ));
                spans.push(Span::styled(" ", Style::default().bg(row_bg)));
            }
            if is_cur {
                spans.push(Span::styled("←", Style::default().fg(SUBTEXT0).bg(row_bg)));
                spans.push(Span::styled(" ", Style::default().bg(row_bg)));
            }

            f.render_widget(
                Paragraph::new(Line::from(spans)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Branch => {
            let disp = truncate(&item.display, content_w);
            let mut line: Vec<Span<'_>> =
                vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            line.push(Span::styled(
                disp,
                Style::default().fg(SURFACE1).italic().bg(row_bg),
            ));
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Agent { name, age } => {
            let color = if is_cur { agent_color(name) } else { SURFACE1 };
            let age_str = age.map(format_age).unwrap_or_default();
            let age_width = if age_str.is_empty() {
                0
            } else {
                age_str.chars().count() + 1 // " " separator
            };
            let name_w = content_w.saturating_sub(age_width);
            let disp = truncate(name, name_w);
            let style = if is_cur {
                Style::default().fg(color).bg(row_bg)
            } else {
                Style::default().fg(color).italic().bg(row_bg)
            };
            let mut line: Vec<Span<'_>> =
                vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            line.push(Span::styled(disp, style));
            if !age_str.is_empty() {
                let a_color = if is_cur {
                    age.map(age_color).unwrap_or(SURFACE1)
                } else {
                    age.map(|d| dim_color(age_color(d))).unwrap_or(SURFACE1)
                };
                line.push(Span::styled(" ", Style::default().bg(row_bg)));
                line.push(Span::styled(
                    age_str,
                    Style::default().fg(a_color).bg(row_bg),
                ));
            }
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Ports(ports) => {
            let text = ports
                .iter()
                .map(|p| format!(":{p}"))
                .collect::<Vec<_>>()
                .join(" ");
            let disp = truncate(&text, content_w);
            let color = if is_cur { GREEN } else { SURFACE1 };
            let mut line: Vec<Span<'_>> =
                vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            line.push(Span::styled(disp, Style::default().fg(color).bg(row_bg)));
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Status => {
            let disp = truncate(&item.display, content_w);
            let color = if is_cur { SUBTEXT0 } else { SURFACE1 };
            let mut line: Vec<Span<'_>> =
                vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            line.push(Span::styled(
                disp,
                Style::default().fg(color).italic().bg(row_bg),
            ));
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Activity(text) => {
            let disp = truncate(text, content_w);
            let color = if is_cur { SUBTEXT0 } else { SURFACE1 };
            let mut line: Vec<Span<'_>> =
                vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            line.push(Span::styled(
                disp,
                Style::default().fg(color).italic().bg(row_bg),
            ));
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::ContextBar { pct, tokens } => {
            // Build bar: 12 cells, per-position gradient colors matching statusline.py
            const BAR_WIDTH: usize = 12;
            let fill = (*pct as f32) * BAR_WIDTH as f32 / 100.0;
            let full = fill.floor() as usize;
            let frac = fill - full as f32;

            let mut spans: Vec<Span<'_>> = Vec::with_capacity(BAR_WIDTH + 6);
            spans.push(bar_span(item, is_sel, row_bg));
            spans.extend(tree_prefix_spans(item.tree, indent, row_bg));

            for i in 0..BAR_WIDTH {
                let (level, ch) = if i < full {
                    (1.0f32, '\u{25A0}') // ■
                } else if i == full && frac > 0.0 {
                    if frac < 0.5 {
                        (frac, '\u{25E7}') // ◧
                    } else {
                        (frac, '\u{25A0}') // ■
                    }
                } else {
                    (0.0, '\u{25A1}') // □
                };
                let color = if level > 0.0 {
                    if is_cur {
                        CTX_POS_COLORS[i]
                    } else {
                        dim_color(CTX_POS_COLORS[i])
                    }
                } else if is_cur {
                    CTX_EMPTY_COLOR
                } else {
                    SURFACE1
                };
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(color).bg(row_bg),
                ));
            }

            // Label: seg digits + ٪ colored by threshold
            let label_color = if is_cur {
                ctx_label_color(*pct)
            } else {
                dim_color(ctx_label_color(*pct))
            };
            spans.push(Span::styled(" ", Style::default().bg(row_bg)));
            spans.push(Span::styled(
                format!("{}\u{066A}", seg_number(*pct as u32)),
                Style::default().fg(label_color).bg(row_bg),
            ));

            // Tokens
            if !tokens.is_empty() {
                spans.push(Span::styled(" ", Style::default().bg(row_bg)));
                let tok_color = if is_cur { OVERLAY0 } else { SURFACE1 };
                spans.push(Span::styled(
                    tokens.clone(),
                    Style::default().fg(tok_color).bg(row_bg),
                ));
            }

            f.render_widget(
                Paragraph::new(Line::from(spans)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Progress(pct) => {
            let bar_cells = content_w.saturating_sub(5).min(12);
            let filled = (*pct as usize * bar_cells) / 100;
            let empty = bar_cells.saturating_sub(filled);
            let pct_text = format!(" {pct}%");
            let filled_color = if is_cur { item.color } else { SURFACE1 };
            let mut line: Vec<Span<'_>> =
                vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            line.push(Span::styled(
                "█".repeat(filled),
                Style::default().fg(filled_color).bg(row_bg),
            ));
            line.push(Span::styled(
                "░".repeat(empty),
                Style::default().fg(SURFACE1).bg(row_bg),
            ));
            line.push(Span::styled(
                pct_text,
                Style::default().fg(OVERLAY0).bg(row_bg),
            ));
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
    }
}

fn footer_hints(width: usize) -> (Line<'static>, Line<'static>) {
    let key = |k: &'static str| Span::styled(k, Style::default().fg(TEXT).bold());
    let lbl = |s: &'static str| Span::styled(s, Style::default().fg(OVERLAY0));
    let sep = || Span::styled("  ", Style::default());

    if width >= 34 {
        let line1 = Line::from(vec![
            lbl(" "),
            key("n"),
            lbl(" new"),
            sep(),
            key("w"),
            lbl(" worktree"),
            sep(),
            key("r"),
            lbl(" rename"),
        ]);
        let line2 = Line::from(vec![
            lbl(" "),
            key("x"),
            lbl(" ditch"),
            sep(),
            key("h"),
            lbl(" hide"),
            sep(),
            key("q"),
            lbl(" close"),
        ]);
        (line1, line2)
    } else if width >= 22 {
        let line1 = Line::from(vec![
            lbl(" "),
            key("n"),
            lbl(" new"),
            sep(),
            key("w"),
            lbl(" wt"),
            sep(),
            key("r"),
            lbl(" ren"),
        ]);
        let line2 = Line::from(vec![
            lbl(" "),
            key("x"),
            lbl(" del"),
            sep(),
            key("h"),
            lbl(" hide"),
            sep(),
            key("q"),
            lbl(" close"),
        ]);
        (line1, line2)
    } else {
        let line1 = Line::from(vec![
            lbl(" "),
            key("n"),
            lbl(" "),
            key("w"),
            lbl(" "),
            key("r"),
            lbl(" "),
            key("x"),
            lbl(" "),
            key("h"),
        ]);
        let line2 = Line::from(vec![lbl(" "), key("q"), lbl(" close")]);
        (line1, line2)
    }
}

fn render_at(f: &mut Frame, x: u16, y: u16, w: u16, line: Line<'_>, bg: Color) {
    let area = Rect {
        x,
        y,
        width: w,
        height: 1,
    };
    f.render_widget(Paragraph::new(line).style(Style::default().bg(bg)), area);
}
