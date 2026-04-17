use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::group::GroupMeta;
use crate::order::compute_order;
use crate::palette::{
    BASE, MANTLE, OVERLAY0, PEACH, SUBTEXT0, SURFACE0, SURFACE1, TEXT, hex_to_color,
};
use crate::sidebar::meta::{AgentBadge, SessionBadges, query_session_badges};
use crate::status::compute_all_colors;
use crate::tmux::tmux;

const TIMEOUT_MS: u128 = 1500;
const OVERLAY_GRACE_MS: u128 = 400;
const HEARTBEAT_FRESH_MS: u128 = 300;
/// Wait this long after the latest press before trusting Ctrl-is-up as a
/// "release" signal — otherwise the brief gap between keydowns while the user
/// is still tapping Tab (with Ctrl held) could be misread as a release.
const CTRL_RELEASE_GRACE_MS: u128 = 80;

// Poll the physical Ctrl modifier via CoreGraphics so we can dismiss the
// overlay the instant the user lets go of the key. Terminals don't observe
// keyup, but the window server does.
#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventSourceFlagsState(state_id: i32) -> u64;
}

#[cfg(target_os = "macos")]
fn ctrl_down() -> bool {
    // kCGEventSourceStateHIDSystemState = 1 — hardware-level modifier state.
    const HID_STATE: i32 = 1;
    const MASK_CTRL: u64 = 0x00040000; // kCGEventFlagMaskControl
    // SAFETY: CGEventSourceFlagsState is thread-safe, reads a single word of
    // WindowServer state, and returns 0 on any error — no pointers involved.
    unsafe { (CGEventSourceFlagsState(HID_STATE) & MASK_CTRL) != 0 }
}

#[cfg(not(target_os = "macos"))]
fn ctrl_down() -> bool {
    true
}

fn state_path() -> PathBuf {
    env::temp_dir().join("mux-mru-cycle")
}

fn heartbeat_path() -> PathBuf {
    env::temp_dir().join("mux-mru-overlay.heartbeat")
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

struct State {
    ts_ms: u128,
    index: isize,
    order: Vec<String>,
}

fn load() -> Option<State> {
    let txt = fs::read_to_string(state_path()).ok()?;
    let mut lines = txt.lines();
    let ts_ms: u128 = lines.next()?.parse().ok()?;
    let index: isize = lines.next()?.parse().ok()?;
    let order: Vec<String> = lines.map(|s| s.to_string()).collect();
    if order.is_empty() {
        return None;
    }
    Some(State {
        ts_ms,
        index,
        order,
    })
}

fn save(s: &State) {
    let mut buf = String::new();
    buf.push_str(&s.ts_ms.to_string());
    buf.push('\n');
    buf.push_str(&s.index.to_string());
    buf.push('\n');
    for name in &s.order {
        buf.push_str(name);
        buf.push('\n');
    }
    // Atomic write — rapid Ctrl+Tab presses can land concurrent saves;
    // a torn read makes `load` return None and resets the index to 0.
    let path = state_path();
    let tmp = path.with_extension("tmp");
    if fs::write(&tmp, buf).is_ok() {
        let _ = fs::rename(&tmp, &path);
    }
}

/// Fetch current session + MRU-ordered session list in a single tmux call —
/// saves one fork+exec on cache-miss (typically first press of a cycle).
const SENTINEL: &str = "\x1e<<MUX_MRU>>\x1e";

fn fetch_current_and_order() -> (String, Vec<String>) {
    let batch = tmux(&[
        "display-message",
        "-p",
        "#S",
        ";",
        "display-message",
        "-p",
        SENTINEL,
        ";",
        "list-sessions",
        "-F",
        "#{session_name}\t#{session_last_attached}",
    ]);
    let Some((cur, rest)) = batch.split_once(SENTINEL) else {
        return (batch.trim().to_string(), Vec::new());
    };
    let current = cur.trim().to_string();
    let mut entries: Vec<(String, u64)> = rest
        .lines()
        .filter_map(|l| {
            let mut it = l.splitn(2, '\t');
            let name = it.next()?.to_string();
            let ts: u64 = it.next()?.parse().ok()?;
            Some((name, ts))
        })
        .collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.1));
    let mut order: Vec<String> = entries.into_iter().map(|(n, _)| n).collect();
    if let Some(pos) = order.iter().position(|n| n == &current) {
        let s = order.remove(pos);
        order.insert(0, s);
    } else if !current.is_empty() {
        order.insert(0, current.clone());
    }
    (current, order)
}

fn overlay_alive() -> bool {
    let Ok(meta) = fs::metadata(heartbeat_path()) else {
        return false;
    };
    let Ok(mtime) = meta.modified() else {
        return false;
    };
    let Ok(age) = mtime.elapsed() else {
        return false;
    };
    age.as_millis() <= HEARTBEAT_FRESH_MS
}

fn self_exe() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
}

fn spawn_overlay(target_session: &str) {
    let Some(exe) = self_exe() else {
        return;
    };
    let clients = tmux(&["list-clients", "-F", "#{client_name}"]);
    let client = clients.lines().next().unwrap_or("").to_string();
    let mut args: Vec<String> = vec!["display-popup".into()];
    if !client.is_empty() {
        args.push("-c".into());
        args.push(client);
    }
    args.extend(
        [
            "-t",
            target_session,
            "-w",
            "100%",
            "-h",
            "7",
            "-x",
            "C",
            "-y",
            "C",
            "-b",
            "none",
            "-s",
            "bg=#11111b",
            "-S",
            "bg=#11111b",
            "-E",
            &exe,
            "mru-overlay",
        ]
        .iter()
        .map(|s| s.to_string()),
    );
    let _ = Command::new("tmux")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

pub(crate) fn cmd_mru_cycle(args: &[String]) {
    let back = args.iter().any(|a| a == "--back");
    let now = now_ms();

    // Cache-hit path skips the tmux round-trip entirely — rapid Ctrl+Tab
    // presses only pay for switch-client. Cache-miss batches current+list.
    let (order, mut index) = match load() {
        Some(s) if now.saturating_sub(s.ts_ms) <= TIMEOUT_MS => (s.order, s.index),
        _ => {
            let (current, order) = fetch_current_and_order();
            if current.is_empty() {
                return;
            }
            (order, 0)
        }
    };
    if order.len() < 2 {
        return;
    }
    let len = order.len() as isize;
    let step = if back { -1 } else { 1 };
    index = ((index + step) % len + len) % len;
    let target = order[index as usize].clone();
    tmux(&["switch-client", "-t", &target]);
    save(&State {
        ts_ms: now,
        index,
        order,
    });
    if !overlay_alive() {
        // Touch the heartbeat now so a second Ctrl+Tab fired while tmux is
        // still starting the popup (~100-300ms) sees it alive and skips a
        // duplicate spawn. The overlay takes over the heartbeat once it runs.
        let _ = fs::write(heartbeat_path(), now.to_string());
        spawn_overlay(&target);
    }
}

// ── Overlay ────────────────────────────────────────────────────────────────

pub(crate) fn cmd_mru_overlay() {
    let stdout = io::stdout();
    if terminal::enable_raw_mode().is_err() {
        return;
    }
    let mut stdout = stdout;
    if execute!(stdout, EnterAlternateScreen).is_err() {
        let _ = terminal::disable_raw_mode();
        return;
    }
    let backend = CrosstermBackend::new(stdout);
    let Ok(mut term) = Terminal::new(backend) else {
        let _ = terminal::disable_raw_mode();
        return;
    };

    // Cache colours + agent badges; these only need to refresh if the order set
    // changes. Cycling through sessions doesn't add new ones.
    let mut last_order: Vec<String> = Vec::new();
    let mut colors: HashMap<String, Color> = HashMap::new();
    let mut badges: HashMap<String, SessionBadges> = HashMap::new();

    let tick = Duration::from_millis(50);
    loop {
        let _ = fs::write(heartbeat_path(), now_ms().to_string());

        let st = load();
        let now = now_ms();
        let stale = match &st {
            Some(s) => now.saturating_sub(s.ts_ms) > TIMEOUT_MS + OVERLAY_GRACE_MS,
            None => true,
        };
        if stale {
            break;
        }
        let s = st.unwrap();
        // Close the instant Ctrl is released — unless we're inside the grace
        // window right after the latest press, where the user is very likely
        // still mid-chord.
        if now.saturating_sub(s.ts_ms) > CTRL_RELEASE_GRACE_MS && !ctrl_down() {
            break;
        }
        if s.order != last_order {
            colors = build_color_map(&s.order);
            badges = query_session_badges(&s.order);
            last_order = s.order.clone();
        }

        let _ = term.draw(|f| draw(f, &s, &colors, &badges));
        thread::sleep(tick);
    }

    let _ = execute!(term.backend_mut(), LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
    let _ = fs::remove_file(heartbeat_path());
}

fn build_color_map(sessions: &[String]) -> HashMap<String, Color> {
    // Same canonical session colours as the status bar — keep the overlay
    // visually consistent with everything else mux paints.
    let alive: std::collections::HashSet<String> = sessions.iter().cloned().collect();
    let ordered = compute_order(&alive, true);
    let meta = GroupMeta::new(&ordered);
    compute_all_colors(&ordered, &meta)
        .into_iter()
        .map(|(name, hex, _)| (name, hex_to_color(&hex)))
        .collect()
}

// ── Drawing ────────────────────────────────────────────────────────────────

fn draw(
    f: &mut ratatui::Frame,
    s: &State,
    colors: &HashMap<String, Color>,
    badges: &HashMap<String, SessionBadges>,
) {
    let area = f.area();
    f.render_widget(Paragraph::new("").style(Style::default().bg(MANTLE)), area);

    if area.height < 5 {
        return;
    }
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top spacer
            Constraint::Length(1), // chip row 1 (name)
            Constraint::Length(1), // chip row 2 (agent)
            Constraint::Length(1), // spacer
            Constraint::Length(1), // hint
            Constraint::Min(0),    // bottom spacer
        ])
        .split(area);

    draw_chips(f, rows[1], rows[2], s, colors, badges);
    draw_hint(f, rows[4], s);
}

struct Chip {
    line1: Vec<Span<'static>>,
    line2: Vec<Span<'static>>,
    width: usize,
}

const CHIP_WIDTH: usize = 26;
const CHIP_GAP: usize = 1;

fn draw_chips(
    f: &mut ratatui::Frame,
    row1: Rect,
    row2: Rect,
    s: &State,
    colors: &HashMap<String, Color>,
    badges: &HashMap<String, SessionBadges>,
) {
    let total = s.order.len();
    if total == 0 || row1.width == 0 {
        return;
    }
    let cur = s.index.rem_euclid(total as isize) as usize;

    let chips: Vec<Chip> = s
        .order
        .iter()
        .enumerate()
        .map(|(i, name)| build_chip(name, colors.get(name).copied(), badges.get(name), i == cur))
        .collect();

    // Window outward from the cursor chip until we run out of horizontal room.
    let max_w = row1.width as usize;
    let mut start = cur;
    let mut end = cur;
    let mut used = chips[cur].width;
    while used <= max_w {
        let mut grew = false;
        if end + 1 < total {
            let need = chips[end + 1].width + CHIP_GAP;
            if used + need <= max_w {
                used += need;
                end += 1;
                grew = true;
            }
        }
        if start > 0 {
            let need = chips[start - 1].width + CHIP_GAP;
            if used + need <= max_w {
                used += need;
                start -= 1;
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }

    let leading = max_w.saturating_sub(used) / 2;
    let lead_span = || Span::styled(" ".repeat(leading), Style::default().bg(MANTLE));
    let gap_span = || Span::styled(" ".repeat(CHIP_GAP), Style::default().bg(MANTLE));

    let mut spans1: Vec<Span<'static>> = vec![lead_span()];
    let mut spans2: Vec<Span<'static>> = vec![lead_span()];
    for (offset, chip) in chips[start..=end].iter().enumerate() {
        if offset > 0 {
            spans1.push(gap_span());
            spans2.push(gap_span());
        }
        spans1.extend(chip.line1.clone());
        spans2.extend(chip.line2.clone());
    }
    f.render_widget(
        Paragraph::new(Line::from(spans1)).style(Style::default().bg(MANTLE)),
        row1,
    );
    f.render_widget(
        Paragraph::new(Line::from(spans2)).style(Style::default().bg(MANTLE)),
        row2,
    );
}

fn build_chip(
    name: &str,
    color: Option<Color>,
    badges: Option<&SessionBadges>,
    is_cursor: bool,
) -> Chip {
    let chip_bg = if is_cursor { SURFACE0 } else { BASE };
    let dot_color = color.unwrap_or(SURFACE1);
    let asking = badges.is_some_and(|b| b.attention || b.agents.iter().any(|a| a.asking));
    let dot = if asking { "★" } else { "●" };
    let name_fg = if is_cursor { TEXT } else { SUBTEXT0 };
    let name_mod = if is_cursor {
        Modifier::BOLD
    } else {
        Modifier::empty()
    };
    // Budget: " ● name …" with 1-cell right padding == 4 fixed cells.
    let name_max = CHIP_WIDTH.saturating_sub(4);
    let line1 = pad_to_chip_width(
        vec![
            Span::styled(" ".to_string(), Style::default().bg(chip_bg)),
            Span::styled(dot.to_string(), Style::default().fg(dot_color).bg(chip_bg)),
            Span::styled(" ".to_string(), Style::default().bg(chip_bg)),
            Span::styled(
                clip(name, name_max),
                Style::default()
                    .fg(name_fg)
                    .bg(chip_bg)
                    .add_modifier(name_mod),
            ),
        ],
        chip_bg,
    );
    let line2 = build_agent_line(badges.map(|b| b.agents.as_slice()).unwrap_or(&[]), chip_bg);

    Chip {
        line1,
        line2,
        width: CHIP_WIDTH,
    }
}

fn build_agent_line(agents: &[AgentBadge], chip_bg: Color) -> Vec<Span<'static>> {
    // Same 3-cell left indent as line1 (" ● ") so the glyph sits under the name.
    // 1 cell of right padding, so max content = CHIP_WIDTH - 4.
    let max_content = CHIP_WIDTH.saturating_sub(4);
    let mut spans: Vec<Span<'static>> = vec![Span::styled(
        "   ".to_string(),
        Style::default().bg(chip_bg),
    )];
    let mut used = 0usize;
    if agents.is_empty() {
        spans.push(Span::styled(
            "—".to_string(),
            Style::default().fg(SURFACE1).bg(chip_bg),
        ));
    } else {
        for (i, a) in agents.iter().enumerate() {
            let glyph = agent_glyph(&a.name);
            let gw = glyph.chars().count();
            let sep = if i > 0 { 1 } else { 0 };
            if used + sep + gw > max_content {
                break;
            }
            if sep > 0 {
                spans.push(Span::styled(" ".to_string(), Style::default().bg(chip_bg)));
                used += 1;
            }
            let glyph_color = agent_color(a.asking, a.gerund.is_some());
            spans.push(Span::styled(
                glyph.to_string(),
                Style::default().fg(glyph_color).bg(chip_bg),
            ));
            used += gw;
            if let Some(ref g) = a.gerund {
                let budget = max_content.saturating_sub(used + 1);
                if budget == 0 {
                    continue;
                }
                let g_clip = clip(g, budget);
                spans.push(Span::styled(
                    format!(" {}", g_clip),
                    Style::default().fg(SUBTEXT0).bg(chip_bg),
                ));
                used += 1 + g_clip.chars().count();
            }
        }
    }
    pad_to_chip_width(spans, chip_bg)
}

fn pad_to_chip_width(mut spans: Vec<Span<'static>>, chip_bg: Color) -> Vec<Span<'static>> {
    let current: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let pad = CHIP_WIDTH.saturating_sub(current);
    if pad > 0 {
        spans.push(Span::styled(" ".repeat(pad), Style::default().bg(chip_bg)));
    }
    spans
}

fn agent_glyph(name: &str) -> &'static str {
    match name.to_lowercase().as_str() {
        s if s.contains("claude") => "󰚩",
        s if s.contains("codex") => "",
        s if s.contains("copilot") => "",
        s if s.contains("aider") => "",
        _ => "",
    }
}

fn agent_color(asking: bool, active: bool) -> Color {
    if asking {
        Color::Rgb(0xF9, 0xE2, 0xAF) // yellow attention
    } else if active {
        Color::Rgb(0xA6, 0xE3, 0xA1) // green busy
    } else {
        OVERLAY0
    }
}

fn draw_hint(f: &mut ratatui::Frame, area: Rect, s: &State) {
    let total = s.order.len();
    let cur = s.index.rem_euclid(total as isize) as usize + 1;
    let txt = format!(" {}/{}    ⇥ cycle    ⇧⇥ back    ⎋ release ", cur, total);
    let pad = (area.width as usize).saturating_sub(txt.chars().count()) / 2;
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ".repeat(pad), Style::default().bg(MANTLE)),
            Span::styled(txt, Style::default().fg(PEACH).bg(MANTLE)),
        ]))
        .style(Style::default().bg(MANTLE)),
        area,
    );
}

fn clip(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let take = max.saturating_sub(1);
    let mut out: String = s.chars().take(take).collect();
    out.push('…');
    out
}
