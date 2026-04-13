use std::cmp::min;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{self, Stdout, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crossterm::cursor;
use crossterm::event::{
    self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use nucleo_matcher::pattern::{Atom, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::picker::PickerItem;
use crate::color::{compute_color, is_static};
use crate::group::{GroupMeta, session_group, session_suffix};
use crate::order::compute_order;
use crate::project::{
    DitchPlan, WtEntry, build_ditch_plan, build_project_items, build_worktree_items,
    create_new_worktree, create_session_at_dir, default_session_name, execute_ditch_action,
    list_worktrees, rename_parts, rename_session, resolve_selected_dir_from_session,
    toggle_favorite, touch_lru, worktree_name_parts,
};
use crate::tmux::tmux;
use crate::{codex, copilot, usage_bars};

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
            | '\u{2736}' // ✶
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
                claude_activity: claude_scrape_map.get(name).and_then(|s| s.activity.clone()),
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum SidebarMode {
    Browse,
    Chooser,
}

enum SidebarOverlay {
    Rename(RenameOverlay),
    Ditch(ListOverlay),
    Project(ProjectOverlay),
    Worktree(WorktreeOverlay),
    SessionName(SessionNameOverlay),
}

enum OverlayKeyResult {
    Unhandled,
    Keep,
    Close,
}

struct RenameOverlay {
    old_name: String,
    prefix: String,
    input: String,
    cursor: usize,
    error: Option<String>,
}

struct ListOverlay {
    title: String,
    items: Vec<PickerItem>,
    selected: usize,
    offset: usize,
    error: Option<String>,
    plan: Option<DitchPlan>,
}

struct ProjectOverlay {
    filter: String,
    cursor: usize,
    all_items: Vec<PickerItem>,
    items: Vec<PickerItem>,
    selected: usize,
    offset: usize,
}

enum WorktreeFlow {
    NewSession,
    NewWorktree,
}

struct WorktreeOverlay {
    flow: WorktreeFlow,
    selected_dir: PathBuf,
    entries: Vec<WtEntry>,
    items: Vec<PickerItem>,
    selected: usize,
    offset: usize,
    error: Option<String>,
}

struct SessionNameOverlay {
    title: String,
    prefix: String,
    input: String,
    cursor: usize,
    default_on_empty: Option<String>,
    final_dir: PathBuf,
    error: Option<String>,
}

fn first_selectable_picker(items: &[PickerItem]) -> usize {
    items.iter().position(|item| item.selectable).unwrap_or(0)
}

fn move_picker_selection(items: &[PickerItem], selected: &mut usize, dir: i32) {
    if items.is_empty() {
        return;
    }
    let mut pos = (*selected).min(items.len().saturating_sub(1));
    loop {
        if dir > 0 {
            if pos + 1 >= items.len() {
                return;
            }
            pos += 1;
        } else {
            if pos == 0 {
                return;
            }
            pos -= 1;
        }
        if items[pos].selectable {
            *selected = pos;
            return;
        }
    }
}

fn prev_char_boundary(s: &str, cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }
    s[..cursor]
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

fn next_char_boundary(s: &str, cursor: usize) -> usize {
    if cursor >= s.len() {
        return s.len();
    }
    let mut iter = s[cursor..].char_indices();
    let _ = iter.next();
    iter.next().map(|(idx, _)| cursor + idx).unwrap_or(s.len())
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '/' | '.')
}

fn prev_word_boundary(s: &str, cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }

    let chars: Vec<(usize, char)> = s[..cursor].char_indices().collect();
    let mut i = chars.len();
    while i > 0 && !is_word_char(chars[i - 1].1) {
        i -= 1;
    }
    while i > 0 && is_word_char(chars[i - 1].1) {
        i -= 1;
    }
    chars.get(i).map(|(idx, _)| *idx).unwrap_or(0)
}

fn next_word_boundary(s: &str, cursor: usize) -> usize {
    if cursor >= s.len() {
        return s.len();
    }

    let chars: Vec<(usize, char)> = s[cursor..].char_indices().collect();
    let mut i = 0usize;
    while i < chars.len() && !is_word_char(chars[i].1) {
        i += 1;
    }
    while i < chars.len() && is_word_char(chars[i].1) {
        i += 1;
    }
    chars.get(i).map(|(idx, _)| cursor + idx).unwrap_or(s.len())
}

fn handle_readline_key(text: &mut String, cursor: &mut usize, key: KeyEvent) -> bool {
    match (key.code, key.modifiers) {
        (KeyCode::Char(c), m) if !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            text.insert(*cursor, c);
            *cursor += c.len_utf8();
            true
        }
        (KeyCode::Backspace, KeyModifiers::ALT) | (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
            let prev = prev_word_boundary(text, *cursor);
            text.drain(prev..*cursor);
            *cursor = prev;
            true
        }
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            if *cursor > 0 {
                let prev = prev_char_boundary(text, *cursor);
                text.drain(prev..*cursor);
                *cursor = prev;
            }
            true
        }
        (KeyCode::Delete, _) => {
            if *cursor < text.len() {
                let next = next_char_boundary(text, *cursor);
                text.drain(*cursor..next);
            }
            true
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            if *cursor < text.len() {
                let next = next_char_boundary(text, *cursor);
                text.drain(*cursor..next);
            }
            true
        }
        (KeyCode::Left, _) | (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
            *cursor = prev_char_boundary(text, *cursor);
            true
        }
        (KeyCode::Right, _) | (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
            *cursor = next_char_boundary(text, *cursor);
            true
        }
        (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
            *cursor = 0;
            true
        }
        (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
            *cursor = text.len();
            true
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            text.drain(..*cursor);
            *cursor = 0;
            true
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            text.truncate(*cursor);
            true
        }
        (KeyCode::Char('b'), KeyModifiers::ALT) => {
            *cursor = prev_word_boundary(text, *cursor);
            true
        }
        (KeyCode::Char('f'), KeyModifiers::ALT) => {
            *cursor = next_word_boundary(text, *cursor);
            true
        }
        (KeyCode::Char('d'), KeyModifiers::ALT) => {
            let next = next_word_boundary(text, *cursor);
            text.drain(*cursor..next);
            true
        }
        _ => false,
    }
}

fn filter_picker_items(items: &[PickerItem], query: &str) -> Vec<PickerItem> {
    if query.is_empty() {
        return items.to_vec();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let atom = Atom::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        nucleo_matcher::pattern::AtomKind::Fuzzy,
        false,
    );
    let needle = atom.needle_text();
    let mut buf = Vec::new();
    let mut matches = Vec::new();

    for item in items {
        let hay = format!("{} {}", item.display, item.id);
        let haystack = Utf32Str::new(&hay, &mut buf);
        let mut indices = Vec::new();
        if let Some(score) = matcher.fuzzy_indices(haystack, needle, &mut indices) {
            matches.push((item.clone(), score));
        }
        buf.clear();
    }

    matches.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.display.cmp(&b.0.display)));
    matches.into_iter().map(|(item, _)| item).collect()
}

fn session_exists(session_name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", &format!("={session_name}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn all_pane_dirs() -> Vec<PathBuf> {
    tmux(&["list-panes", "-a", "-F", "#{pane_current_path}"])
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn used_worktree_paths(entries: &[WtEntry]) -> HashSet<String> {
    let pane_dirs = all_pane_dirs();
    entries
        .iter()
        .filter(|entry| {
            let entry_path = Path::new(&entry.path);
            pane_dirs.iter().any(|dir| dir.starts_with(entry_path))
        })
        .map(|entry| entry.path.clone())
        .collect()
}

fn build_sidebar_worktree_items(entries: &[WtEntry]) -> (Vec<PickerItem>, usize) {
    let used = used_worktree_paths(entries);
    let mut items = build_worktree_items(entries);

    for item in items.iter_mut().skip(1) {
        if used.contains(&item.id) {
            item.right_label = "live".to_string();
        }
    }

    let selected = items
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, item)| item.selectable && !used.contains(&item.id))
        .map(|(idx, _)| idx)
        .unwrap_or_else(|| first_selectable_picker(&items));

    (items, selected)
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
    items: Vec<Item>,
    visible: Vec<usize>,
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
    mode: SidebarMode,
    overlay: Option<SidebarOverlay>,
    filter: String,
    filter_cursor: usize,
}

const ACTIVITY_GRACE: Duration = Duration::from_secs(15);

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
            activity_cache: HashMap::new(),
            last_meta_refresh: Instant::now() - Duration::from_secs(60),
            focused: true,
            notched: false,
            mode: SidebarMode::Browse,
            overlay: None,
            filter: String::new(),
            filter_cursor: 0,
        }
    }

    fn chooser_active(&self) -> bool {
        self.mode == SidebarMode::Chooser
    }

    fn overlay_active(&self) -> bool {
        self.overlay.is_some()
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

    fn open_rename_overlay(&mut self) {
        let Some(old_name) = self.selected_session_id() else {
            return;
        };
        let (prefix, suffix) = rename_parts(&old_name);
        self.overlay = Some(SidebarOverlay::Rename(RenameOverlay {
            old_name,
            prefix,
            input: suffix.clone(),
            cursor: suffix.len(),
            error: None,
        }));
    }

    fn open_ditch_overlay(&mut self) {
        let Some(session) = self.selected_session_id() else {
            return;
        };
        let Some(plan) = build_ditch_plan(&session) else {
            return;
        };
        self.overlay = Some(SidebarOverlay::Ditch(ListOverlay {
            title: format!("Ditch {session}"),
            selected: first_selectable_picker(&plan.actions),
            offset: 0,
            items: plan.actions.clone(),
            error: None,
            plan: Some(plan),
        }));
    }

    fn open_project_overlay(&mut self) {
        let items = build_project_items("all");
        self.overlay = Some(SidebarOverlay::Project(ProjectOverlay {
            filter: String::new(),
            cursor: 0,
            selected: first_selectable_picker(&items),
            offset: 0,
            all_items: items.clone(),
            items,
        }));
    }

    fn open_worktree_overlay(&mut self) {
        let Some(target) = self.selected_session_id() else {
            return;
        };
        let Some(selected_dir) = resolve_selected_dir_from_session(Some(&target)) else {
            return;
        };
        let entries = list_worktrees(&selected_dir);
        if entries.is_empty() {
            return;
        }
        let (items, selected) = build_sidebar_worktree_items(&entries);
        self.overlay = Some(SidebarOverlay::Worktree(WorktreeOverlay {
            flow: WorktreeFlow::NewWorktree,
            selected_dir,
            entries,
            selected,
            offset: 0,
            items,
            error: None,
        }));
    }

    fn open_worktree_overlay_for_dir(&mut self, selected_dir: PathBuf, flow: WorktreeFlow) {
        let entries = list_worktrees(&selected_dir);
        if entries.is_empty() {
            let final_dir = selected_dir.clone();
            let default_name = default_session_name(&selected_dir, &final_dir);
            self.open_session_name_overlay(
                "Session".to_string(),
                String::new(),
                default_name.clone(),
                Some(default_name),
                final_dir,
            );
            return;
        }
        let (items, selected) = build_sidebar_worktree_items(&entries);
        self.overlay = Some(SidebarOverlay::Worktree(WorktreeOverlay {
            flow,
            selected_dir,
            entries,
            selected,
            offset: 0,
            items,
            error: None,
        }));
    }

    fn open_session_name_overlay(
        &mut self,
        title: String,
        prefix: String,
        initial: String,
        default_on_empty: Option<String>,
        final_dir: PathBuf,
    ) {
        self.overlay = Some(SidebarOverlay::SessionName(SessionNameOverlay {
            title,
            prefix,
            cursor: initial.len(),
            input: initial,
            default_on_empty,
            final_dir,
            error: None,
        }));
    }

    fn rebuild_visible(&mut self) {
        self.visible.clear();
        self.visible.extend(0..self.items.len());
    }

    fn search_matches(&self) -> Vec<(usize, u16)> {
        let mut matcher = Matcher::new(Config::DEFAULT);
        let atom = Atom::new(
            &self.filter,
            CaseMatching::Ignore,
            Normalization::Smart,
            nucleo_matcher::pattern::AtomKind::Fuzzy,
            false,
        );
        let needle = atom.needle_text();
        let mut buf = Vec::new();
        let mut matches = Vec::new();

        for (idx, item) in self.items.iter().enumerate() {
            if !item.selectable {
                continue;
            }
            let hay = match item.session_id.as_ref() {
                Some(session_id) => format!("{} {}", item.display, session_id),
                None => item.display.clone(),
            };
            let haystack = Utf32Str::new(&hay, &mut buf);
            let mut indices = Vec::new();
            if let Some(score) = matcher.fuzzy_indices(haystack, needle, &mut indices) {
                matches.push((idx, score));
            }
            buf.clear();
        }

        matches.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        matches
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
        if let Some(idx) = self.selectable_visible_indices().into_iter().next() {
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
        self.rebuild_visible();

        // When unfocused, always track current session
        if !self.focused {
            self.snap_to_current();
            return;
        }

        if let Some(ref id) = prev_id
            && let Some(pos) = self.items.iter().position(|i| i.id == *id)
            && self.is_visible_index(pos)
        {
            self.selected = pos;
            return;
        }
        if self.chooser_active() && !self.filter.is_empty() {
            self.apply_filter_change();
        } else {
            self.snap_to_current();
        }
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
            let exe = std::env::current_exe().unwrap_or_else(|_| "tmux-session".into());
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

    fn handle_overlay_key(&mut self, key: KeyEvent) -> bool {
        let Some(mut overlay) = self.overlay.take() else {
            return false;
        };

        let result = match &mut overlay {
            SidebarOverlay::Rename(rename) => self.handle_rename_key(rename, key),
            SidebarOverlay::Ditch(list) => self.handle_ditch_key(list, key),
            SidebarOverlay::Project(project) => self.handle_project_key(project, key),
            SidebarOverlay::Worktree(worktree) => self.handle_worktree_key(worktree, key),
            SidebarOverlay::SessionName(session) => self.handle_session_name_key(session, key),
        };

        match result {
            OverlayKeyResult::Unhandled => {
                self.overlay = Some(overlay);
                false
            }
            OverlayKeyResult::Keep => {
                if self.overlay.is_none() {
                    self.overlay = Some(overlay);
                }
                true
            }
            OverlayKeyResult::Close => true,
        }
    }

    fn handle_rename_key(&mut self, rename: &mut RenameOverlay, key: KeyEvent) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                let new_suffix = rename.input.trim();
                if new_suffix.is_empty() {
                    rename.error = Some("session name required".to_string());
                    return OverlayKeyResult::Keep;
                }
                let new_name = format!("{}{}", rename.prefix, new_suffix);
                match rename_session(&rename.old_name, &new_name) {
                    Ok(()) => OverlayKeyResult::Close,
                    Err(err) => {
                        rename.error = Some(err);
                        OverlayKeyResult::Keep
                    }
                }
            }
            _ if handle_readline_key(&mut rename.input, &mut rename.cursor, key) => {
                rename.error = None;
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
        }
    }

    fn handle_ditch_key(&mut self, list: &mut ListOverlay, key: KeyEvent) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                if let Some(item) = list.items.get(list.selected)
                    && item.selectable
                    && let Some(plan) = list.plan.as_ref()
                {
                    match execute_ditch_action(plan, &item.id) {
                        Ok(()) => {
                            handoff_to_main(self);
                            return OverlayKeyResult::Close;
                        }
                        Err(err) => list.error = Some(err),
                    }
                }
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                move_picker_selection(&list.items, &mut list.selected, 1);
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                move_picker_selection(&list.items, &mut list.selected, -1);
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
        }
    }

    fn handle_project_key(&mut self, project: &mut ProjectOverlay, key: KeyEvent) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                let Some(item) = project.items.get(project.selected) else {
                    return OverlayKeyResult::Keep;
                };
                let selected_dir = PathBuf::from(&item.id);
                touch_lru(&item.id);
                self.open_worktree_overlay_for_dir(selected_dir, WorktreeFlow::NewSession);
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('s'), KeyModifiers::ALT) => {
                if let Some(item) = project.items.get(project.selected) {
                    toggle_favorite(&item.id);
                    project.all_items = build_project_items("all");
                    project.items = filter_picker_items(&project.all_items, &project.filter);
                    project.selected = first_selectable_picker(&project.items);
                    project.offset = 0;
                }
                OverlayKeyResult::Keep
            }
            _ if handle_readline_key(&mut project.filter, &mut project.cursor, key) => {
                project.items = filter_picker_items(&project.all_items, &project.filter);
                project.selected = first_selectable_picker(&project.items);
                project.offset = 0;
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                move_picker_selection(&project.items, &mut project.selected, 1);
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                move_picker_selection(&project.items, &mut project.selected, -1);
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
        }
    }

    fn handle_worktree_key(&mut self, worktree: &mut WorktreeOverlay, key: KeyEvent) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                let Some(item) = worktree.items.get(worktree.selected) else {
                    return OverlayKeyResult::Keep;
                };

                let (final_dir, branch) = if item.id == "__new__" {
                    match create_new_worktree(&worktree.selected_dir, &worktree.entries) {
                        Ok((path, name)) => (path, Some(name)),
                        Err(err) => {
                            worktree.error = Some(err);
                            return OverlayKeyResult::Keep;
                        }
                    }
                } else {
                    let branch = worktree
                        .entries
                        .iter()
                        .find(|entry| entry.path == item.id)
                        .and_then(|entry| entry.branch.clone());
                    (PathBuf::from(&item.id), branch)
                };

                match worktree.flow {
                    WorktreeFlow::NewSession => {
                        let default_name = default_session_name(&worktree.selected_dir, &final_dir);
                        self.open_session_name_overlay(
                            "Session".to_string(),
                            String::new(),
                            default_name.clone(),
                            Some(default_name),
                            final_dir,
                        );
                    }
                    WorktreeFlow::NewWorktree => {
                        let (repo_name, default_suffix) =
                            worktree_name_parts(&worktree.selected_dir, branch.as_deref());
                        let title = if repo_name.is_empty() {
                            "Session".to_string()
                        } else {
                            format!("{repo_name}/")
                        };
                        let prefix = if repo_name.is_empty() {
                            String::new()
                        } else {
                            format!("{repo_name}/")
                        };
                        self.open_session_name_overlay(
                            title,
                            prefix,
                            default_suffix,
                            None,
                            final_dir,
                        );
                    }
                }
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                move_picker_selection(&worktree.items, &mut worktree.selected, 1);
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                move_picker_selection(&worktree.items, &mut worktree.selected, -1);
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
        }
    }

    fn handle_session_name_key(&mut self, session: &mut SessionNameOverlay, key: KeyEvent) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                let raw = session.input.trim();
                let suffix = if raw.is_empty() {
                    session.default_on_empty.clone().unwrap_or_default()
                } else {
                    raw.to_string()
                };
                if suffix.is_empty() {
                    session.error = Some("session name required".to_string());
                    return OverlayKeyResult::Keep;
                }
                let session_name = format!("{}{}", session.prefix, suffix);
                if session_exists(&session_name) {
                    tmux(&["switch-client", "-t", &format!("={session_name}")]);
                } else {
                    create_session_at_dir(&session_name, &session.final_dir);
                }
                handoff_to_main(self);
                OverlayKeyResult::Close
            }
            _ if handle_readline_key(&mut session.input, &mut session.cursor, key) => {
                session.error = None;
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
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
        let is_last_in_group =
            is_grouped && sessions.get(i + 1).map(|n| session_group(n)) != Some(group);
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
    copilot::start_poller();
    codex::start_poller();

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
                    state.close_overlay();
                    state.close_chooser();
                    state.snap_to_current();
                }
                Ok(Event::Mouse(_)) if state.overlay_active() => {}
                Ok(Event::Mouse(me)) => match me.kind {
                    MouseEventKind::Down(MouseButton::Left)
                        if me.row >= last_list_y && me.row < last_list_y + last_list_h =>
                    {
                        let vis_idx = state.offset + (me.row - last_list_y) as usize;
                        if let Some(item_idx) = state.visible.get(vis_idx).copied()
                            && let Some(sid) = state
                                .items
                                .get(item_idx)
                                .and_then(|i| i.session_id.clone())
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
                            (KeyCode::Char('h'), KeyModifiers::ALT) => {
                                if let Some(id) = state.selected_session_id() {
                                    spawn_subcmd(&mut terminal, &["hide-toggle", &id]);
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
                            _ if handle_readline_key(&mut state.filter, &mut state.filter_cursor, key) => {
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
                        (KeyCode::Char('h'), _) => {
                            if let Some(id) = state.selected_session_id() {
                                spawn_subcmd(&mut terminal, &["hide-toggle", &id]);
                            }
                        }
                        (KeyCode::Char('/'), _) => {
                            state.open_chooser();
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
    let bg = if tmux(&["show-option", "-gv", "@notched"]) == "1" {
        "000000"
    } else {
        "11111b"
    };
    let _ = Command::new(&exe)
        .args(args)
        .env("TMUX_SESSION_BG", bg)
        .status();
    enter_tui();
    terminal.clear().ok();
}

// ── Drawing ──────────────────────────────────────────────────

/// Returns (list_y_start, list_height) for mouse mapping.
fn draw(f: &mut Frame, state: &mut SidebarState) -> (u16, u16) {
    let area = f.area();
    f.render_widget(Clear, area);

    let bg = if state.notched {
        Color::Rgb(0, 0, 0)
    } else {
        MANTLE
    };

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
    //  row 0: blank or chooser filter
    //  list rows
    //  usage graph (if there's room)
    //  2 footer rows
    let list_y = area.y + notch_h + 1;
    let footer_h = 2u16;
    let bars = usage_bars::collect();
    let wanted_bars_h = usage_bars::height(bars.len());
    // Only show bars when the list still has room for at least 4 rows after.
    // Add 2 extra rows for dim separator lines above and below the bars.
    let bars_h = if wanted_bars_h > 0 && area.height >= 4 + 1 + notch_h + wanted_bars_h + 2 + footer_h {
        wanted_bars_h
    } else {
        0
    };
    let sep_h = if bars_h > 0 { 2u16 } else { 0 };
    let list_h = area.height.saturating_sub(1 + notch_h + footer_h + bars_h + sep_h);

    // Footer hints
    let hint1_y = area.y + area.height - 2;
    let hint2_y = area.y + area.height - 1;
    let (hint1, hint2) = if state.overlay_active() {
        overlay_footer_hints(content_w as usize)
    } else if state.chooser_active() {
        chooser_footer_hints(content_w as usize)
    } else {
        footer_hints(content_w as usize)
    };
    render_at(f, area.x, hint1_y, content_w, hint1, bg);
    render_at(f, area.x, hint2_y, content_w, hint2, bg);

    if state.chooser_active() {
        let filter_area = Rect {
            x: area.x,
            y: area.y + notch_h,
            width: content_w,
            height: 1,
        };
        render_filter_row(f, filter_area, state, bg);
    }

    if bars_h > 0 {
        let sep_y_above = list_y + list_h;
        let bars_y = sep_y_above + 1;
        let sep_y_below = bars_y + bars_h;
        let sep_line = Line::from(Span::styled(
            "╌".repeat(content_w as usize),
            Style::default().fg(SURFACE1).bg(bg),
        ));
        render_at(f, area.x, sep_y_above, content_w, sep_line.clone(), bg);
        render_at(f, area.x, sep_y_below, content_w, sep_line, bg);
        let bars_rect = Rect {
            x: area.x,
            y: bars_y,
            width: content_w,
            height: bars_h,
        };
        usage_bars::draw(f, bars_rect, bg, &bars);
    }

    if list_h == 0 {
        return (list_y, 0);
    }

    let list_w_with_bar = content_w.saturating_sub(1); // right pad
    let total = state.visible.len();
    let list_height = list_h as usize;
    let selected_visible = state.visible.iter().position(|idx| *idx == state.selected);

    // Scroll
    if let Some(selected_visible) = selected_visible {
        if selected_visible < state.offset {
            state.offset = selected_visible;
        }
        if selected_visible >= state.offset + list_height {
            state.offset = selected_visible - list_height + 1;
        }
    } else {
        state.offset = 0;
    }

    let selected_session = state.selected_session_id();

    for vi in 0..list_height.min(total.saturating_sub(state.offset)) {
        let item_idx = state.visible[state.offset + vi];
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

        let rendered_inline_rename = if item_idx == state.selected {
            if let Some(SidebarOverlay::Rename(rename)) = state.overlay.as_mut() {
                render_inline_rename_item(
                    f,
                    row,
                    item,
                    rename,
                    is_hover,
                    belongs_to_current,
                    bg,
                    state.focused,
                );
                true
            } else {
                false
            }
        } else {
            false
        };

        if !rendered_inline_rename {
            render_item(f, row, item, is_sel, is_hover, belongs_to_current, bg);
        }
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

    if let Some(overlay) = state.overlay.as_mut() {
        if !matches!(overlay, SidebarOverlay::Rename(_)) {
            let anchor_rel = selected_visible
                .map(|idx| idx.saturating_sub(state.offset))
                .unwrap_or(0) as u16;
            let anchor_y = list_y + anchor_rel.min(list_h.saturating_sub(1));
            let overlay_h = overlay_height(overlay, list_h);
            let overlay_y = anchor_y.min(list_y + list_h.saturating_sub(overlay_h));
            let overlay_area = Rect {
                x: area.x,
                y: overlay_y,
                width: list_w_with_bar,
                height: overlay_h,
            };
            render_overlay(f, overlay_area, overlay, bg, state.focused);
        }
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
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
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
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
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
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
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
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
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
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
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
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
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
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
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

fn render_inline_rename_item(
    f: &mut Frame,
    row: Rect,
    item: &Item,
    rename: &mut RenameOverlay,
    is_hover: bool,
    is_cur: bool,
    _bg: Color,
    focused: bool,
) {
    let w = row.width as usize;
    if w == 0 {
        return;
    }

    const HOVER_BG: Color = Color::Rgb(0x28, 0x29, 0x3a);
    let row_bg = if is_hover {
        HOVER_BG
    } else if is_cur {
        BASE
    } else {
        SURFACE0
    };

    let indent = item.indent as usize;
    let mut spans: Vec<Span<'_>> = vec![bar_span(item, true, row_bg)];
    spans.extend(tree_prefix_spans(item.tree, indent, row_bg));

    let prefix_width = rename.prefix.chars().count();
    let error_text = rename
        .error
        .as_ref()
        .map(|err| format!("  ! {}", truncate(err, 24)))
        .unwrap_or_default();
    let reserved = error_text.chars().count();
    let used = spans.iter().skip(1).map(|s| s.width()).sum::<usize>();
    let available = w.saturating_sub(1 + used + reserved);
    let editable_width = available.saturating_sub(prefix_width).max(1);
    let shown_input = truncate(&rename.input, editable_width);

    spans.push(Span::styled(
        rename.prefix.clone(),
        Style::default().fg(SUBTEXT0).bg(row_bg),
    ));
    if shown_input.is_empty() {
        spans.push(Span::styled(" ", Style::default().bg(row_bg)));
    } else {
        spans.push(Span::styled(
            shown_input.clone(),
            Style::default().fg(TEXT).bold().bg(row_bg),
        ));
    }
    if !error_text.is_empty() {
        spans.push(Span::styled(
            error_text,
            Style::default().fg(PEACH).bg(row_bg),
        ));
    }

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(row_bg)),
        row,
    );

    if focused {
        let base_x = row.x + 1 + indent as u16 + prefix_width as u16;
        let max_x = row.x + row.width.saturating_sub(1);
        let cursor_x = (base_x + rename.cursor as u16).min(max_x);
        f.set_cursor_position((cursor_x, row.y));
    }
}

fn footer_hints(width: usize) -> (Line<'static>, Line<'static>) {
    let key = |k: &'static str| Span::styled(k, Style::default().fg(TEXT).bold().italic());
    let lbl = |s: &'static str| Span::styled(s, Style::default().fg(OVERLAY0).italic());
    let sep = || Span::styled("  ", Style::default());

    if width >= 34 {
        let line1 = Line::from(vec![
            lbl(" "),
            key("n"),
            lbl(" new"),
            sep(),
            key("w"),
            lbl(" work"),
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
            lbl(" hid"),
            sep(),
            key("M-jk"),
            lbl(" mv"),
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

fn chooser_footer_hints(width: usize) -> (Line<'static>, Line<'static>) {
    let key = |k: &'static str| Span::styled(k, Style::default().fg(TEXT).bold().italic());
    let lbl = |s: &'static str| Span::styled(s, Style::default().fg(OVERLAY0).italic());
    let sep = || Span::styled("  ", Style::default());

    if width >= 34 {
        let line1 = Line::from(vec![
            lbl(" "),
            key("↵"),
            lbl(" jump"),
            sep(),
            key("/text"),
            lbl(" search"),
            sep(),
            key("esc"),
            lbl(" done"),
        ]);
        let line2 = Line::from(vec![
            lbl(" "),
            key("M-h"),
            lbl(" hide"),
            sep(),
            key("M-jk"),
            lbl(" move"),
            sep(),
            key("q"),
            lbl(" close"),
        ]);
        (line1, line2)
    } else if width >= 22 {
        let line1 = Line::from(vec![
            lbl(" "),
            key("↵"),
            lbl(" jump"),
            sep(),
            key("esc"),
            lbl(" done"),
        ]);
        let line2 = Line::from(vec![
            lbl(" "),
            key("M-h"),
            lbl(" hide"),
            sep(),
            key("q"),
            lbl(" close"),
        ]);
        (line1, line2)
    } else {
        let line1 = Line::from(vec![lbl(" "), key("↵"), lbl(" "), key("esc")]);
        let line2 = Line::from(vec![lbl(" "), key("q"), lbl(" close")]);
        (line1, line2)
    }
}

fn overlay_footer_hints(width: usize) -> (Line<'static>, Line<'static>) {
    let key = |k: &'static str| Span::styled(k, Style::default().fg(TEXT).bold().italic());
    let lbl = |s: &'static str| Span::styled(s, Style::default().fg(OVERLAY0).italic());
    let sep = || Span::styled("  ", Style::default());

    if width >= 34 {
        (
            Line::from(vec![
                lbl(" "),
                key("↵"),
                lbl(" use"),
                sep(),
                key("esc"),
                lbl(" back"),
            ]),
            Line::from(vec![lbl(" ")]),
        )
    } else {
        (
            Line::from(vec![lbl(" "), key("↵"), lbl(" "), key("esc")]),
            Line::from(vec![lbl(" ")]),
        )
    }
}

fn overlay_height(overlay: &SidebarOverlay, max_list_h: u16) -> u16 {
    let desired = match overlay {
        SidebarOverlay::Rename(rename) => 1 + u16::from(rename.error.is_some()),
        SidebarOverlay::SessionName(session) => 1 + u16::from(session.error.is_some()),
        SidebarOverlay::Project(project) => 1 + min(project.items.len(), 4) as u16,
        SidebarOverlay::Worktree(worktree) => u16::from(worktree.error.is_some()) + worktree.items.len() as u16,
        SidebarOverlay::Ditch(list) => u16::from(list.error.is_some()) + min(list.items.len(), 4) as u16,
    };
    desired.clamp(1, max_list_h.max(1))
}

fn render_overlay(
    f: &mut Frame,
    area: Rect,
    overlay: &mut SidebarOverlay,
    bg: Color,
    focused: bool,
) {
    if area.width < 4 || area.height == 0 {
        return;
    }

    let inner = area;
    f.render_widget(Clear, inner);
    f.render_widget(Paragraph::new("").style(Style::default().bg(bg)), inner);

    match overlay {
        SidebarOverlay::Rename(_) => {}
        SidebarOverlay::SessionName(session) => render_text_overlay(
            f,
            inner,
            &session.title,
            &session.prefix,
            &session.input,
            session.cursor,
            session.error.as_deref(),
            focused,
            bg,
        ),
        SidebarOverlay::Project(project) => {
            render_picker_overlay(f, inner, "New Session", Some((&project.filter, project.cursor)), &mut project.selected, &mut project.offset, &project.items, None, focused, bg)
        }
        SidebarOverlay::Worktree(worktree) => render_picker_overlay(
            f,
            inner,
            match worktree.flow {
                WorktreeFlow::NewSession => "Worktree",
                WorktreeFlow::NewWorktree => "New Worktree",
            },
            None,
            &mut worktree.selected,
            &mut worktree.offset,
            &worktree.items,
            worktree.error.as_deref(),
            focused,
            bg,
        ),
        SidebarOverlay::Ditch(list) => render_picker_overlay(
            f,
            inner,
            &list.title,
            None,
            &mut list.selected,
            &mut list.offset,
            &list.items,
            list.error.as_deref(),
            focused,
            bg,
        ),
    }
}

fn render_text_overlay(
    f: &mut Frame,
    area: Rect,
    title: &str,
    prefix: &str,
    input: &str,
    cursor: usize,
    error: Option<&str>,
    focused: bool,
    bg: Color,
) {
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(if error.is_some() { 1 } else { 0 })]).split(area);

    let input_bg = bg;
    let placeholder = if prefix.is_empty() {
        format!("{}...", title.to_lowercase())
    } else {
        "session name...".to_string()
    };
    let content = if input.is_empty() {
        Line::from(vec![
            Span::styled(prefix.to_string(), Style::default().fg(SUBTEXT0).bg(input_bg)),
            Span::styled(
                placeholder,
                Style::default().fg(OVERLAY0).italic().bg(input_bg),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(prefix.to_string(), Style::default().fg(SUBTEXT0).bg(input_bg)),
            Span::styled(input.to_string(), Style::default().fg(TEXT).bg(input_bg)),
        ])
    };
    let line = Line::from(
        std::iter::once(Span::styled("▌", Style::default().fg(BLUE).bg(input_bg)))
            .chain(std::iter::once(Span::styled(" ", Style::default().bg(input_bg))))
            .chain(content.spans)
            .collect::<Vec<_>>(),
    );
    f.render_widget(Paragraph::new(line).style(Style::default().bg(input_bg)), chunks[0]);

    if let Some(error) = error {
        render_at(
            f,
            chunks[1].x,
            chunks[1].y,
            chunks[1].width,
            Line::from(vec![
                Span::styled("! ", Style::default().fg(PEACH).bg(bg)),
                Span::styled(error.to_string(), Style::default().fg(PEACH).bg(bg)),
            ]),
            bg,
        );
    }

    if focused {
        let max_x = chunks[0].x + chunks[0].width.saturating_sub(1);
        let cursor_x = (chunks[0].x + 2 + prefix.chars().count() as u16 + cursor as u16).min(max_x);
        f.set_cursor_position((cursor_x, chunks[0].y));
    }
}

#[allow(clippy::too_many_arguments)]
fn render_picker_overlay(
    f: &mut Frame,
    area: Rect,
    _title: &str,
    filter: Option<(&str, usize)>,
    selected: &mut usize,
    offset: &mut usize,
    items: &[PickerItem],
    error: Option<&str>,
    focused: bool,
    _bg: Color,
) {
    let overlay_bg = _bg;
    let chunks = Layout::vertical([
        Constraint::Length(if filter.is_some() { 1 } else { 0 }),
        Constraint::Length(if error.is_some() { 1 } else { 0 }),
        Constraint::Min(1),
    ])
    .split(area);
    if let Some((query, cursor)) = filter {
        let input_bg = overlay_bg;
        let line = if query.is_empty() {
            Line::from(vec![
                Span::styled("/ ", Style::default().fg(SUBTEXT0).bg(input_bg)),
                Span::styled(
                    "search...",
                    Style::default().fg(OVERLAY0).italic().bg(input_bg),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled("/ ", Style::default().fg(SUBTEXT0).bg(input_bg)),
                Span::styled(query.to_string(), Style::default().fg(TEXT).bg(input_bg)),
            ])
        };
        f.render_widget(Paragraph::new(line).style(Style::default().bg(input_bg)), chunks[0]);
        if focused {
            let max_x = chunks[0].x + chunks[0].width.saturating_sub(1);
            let cursor_x = (chunks[0].x + 2 + cursor as u16).min(max_x);
            f.set_cursor_position((cursor_x, chunks[0].y));
        }
    }

    if let Some(error) = error {
        let row = if filter.is_some() { chunks[1] } else { chunks[0] };
        render_at(
            f,
            row.x,
            row.y,
            row.width,
            Line::from(vec![
                Span::styled("! ", Style::default().fg(PEACH).bg(overlay_bg)),
                Span::styled(error.to_string(), Style::default().fg(PEACH).bg(overlay_bg)),
            ]),
            overlay_bg,
        );
    }

    let list_area = chunks[2];
    let visible_height = list_area.height as usize;
    if visible_height == 0 {
        return;
    }

    if *selected < *offset {
        *offset = *selected;
    }
    if *selected >= *offset + visible_height {
        *offset = *selected - visible_height + 1;
    }

    for vi in 0..visible_height.min(items.len().saturating_sub(*offset)) {
        let idx = *offset + vi;
        let item = &items[idx];
        let row = Rect {
            x: list_area.x,
            y: list_area.y + vi as u16,
            width: list_area.width,
            height: 1,
        };

        if !item.selectable && item.display.is_empty() {
            render_at(
                f,
                row.x,
                row.y,
                row.width,
                Line::from(Span::styled(
                    "╌".repeat(row.width as usize),
                    Style::default().fg(SURFACE1).bg(overlay_bg),
                )),
                overlay_bg,
            );
            continue;
        }

        let row_bg = if idx == *selected { SURFACE0 } else { overlay_bg };
        let marker_color = if idx == *selected {
            item.color.unwrap_or(BLUE)
        } else if item.selectable {
            SURFACE1
        } else {
            TREE_COLOR
        };
        let text_style = if idx == *selected {
            item.style.fg(item.color.unwrap_or(TEXT)).bold().bg(row_bg)
        } else if item.selectable {
            item.style.bg(row_bg)
        } else {
            item.style.fg(OVERLAY0).bg(row_bg)
        };
        let right_style = if idx == *selected {
            Style::default().fg(SUBTEXT0).italic().bg(row_bg)
        } else {
            Style::default().fg(OVERLAY0).italic().bg(row_bg)
        };

        let mut spans = vec![Span::styled(
            if idx == *selected {
                "▌"
            } else if item.selectable {
                "│"
            } else {
                " "
            },
            Style::default().fg(marker_color).bg(row_bg),
        )];
        spans.push(Span::styled(" ", Style::default().bg(row_bg)));

        let text_width = row.width.saturating_sub(2) as usize;
        let label_width = item.right_label.chars().count();
        let main_width =
            text_width.saturating_sub(label_width.saturating_add(if label_width > 0 { 1 } else { 0 }));
        spans.push(Span::styled(truncate(&item.display, main_width), text_style));
        if label_width > 0 {
            let used = spans.iter().map(|span| span.width()).sum::<usize>().saturating_sub(2);
            let pad = text_width.saturating_sub(used + label_width);
            if pad > 0 {
                spans.push(Span::styled(" ".repeat(pad), Style::default().bg(row_bg)));
            }
            spans.push(Span::styled(item.right_label.clone(), right_style));
        }
        f.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(row_bg)),
            row,
        );
    }
}

fn render_filter_row(f: &mut Frame, row: Rect, state: &SidebarState, bg: Color) {
    if row.width == 0 {
        return;
    }

    let prefix = "/ ";
    let line = if state.filter.is_empty() {
        Line::from(vec![
            Span::styled(prefix, Style::default().fg(SUBTEXT0).bg(bg)),
            Span::styled("search...", Style::default().fg(OVERLAY0).italic().bg(bg)),
        ])
    } else {
        Line::from(vec![
            Span::styled(prefix, Style::default().fg(SUBTEXT0).bg(bg)),
            Span::styled(&state.filter, Style::default().fg(TEXT).bg(bg)),
        ])
    };

    f.render_widget(Paragraph::new(line).style(Style::default().bg(bg)), row);

    if state.focused {
        let max_x = row.x + row.width.saturating_sub(1);
        let cursor_x = (row.x + prefix.chars().count() as u16 + state.filter_cursor as u16).min(max_x);
        f.set_cursor_position((cursor_x, row.y));
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
