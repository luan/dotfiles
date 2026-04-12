use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

const FIVE_HOURS: i64 = 5 * 3600;
const SEVEN_DAYS: i64 = 7 * 24 * 3600;
const THIRTY_DAYS: i64 = 30 * 24 * 3600;

const DIM: Color = Color::Rgb(0x6c, 0x70, 0x86);
const CYAN: Color = Color::Rgb(0x89, 0xdc, 0xeb);
const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const ORANGE: Color = Color::Rgb(0xfa, 0xb3, 0x87);
const RED: Color = Color::Rgb(0xf3, 0x8b, 0xa8);

// Provider identity colors (used for labels and to tint the burn color).
const CLAUDE_COLOR: Color = Color::Rgb(0xd8, 0x7b, 0x4a); // warm amber
const COPILOT_COLOR: Color = Color::Rgb(0xcb, 0xa6, 0xf7); // mauve
const CODEX_COLOR: Color = Color::Rgb(0x74, 0xc7, 0xec); // sky blue

pub struct Bar {
    pub label: String,
    pub pct: f64,
    pub window_secs: i64,
    pub reset_ts: i64,
    pub provider: Color,
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn fmt_reset(secs: i64) -> String {
    let a = secs.max(0);
    if a >= 86400 {
        format!("{}d{}h", a / 86400, (a % 86400) / 3600)
    } else if a >= 3600 {
        format!("{}h{:02}m", a / 3600, (a % 3600) / 60)
    } else {
        format!("{}m", a / 60)
    }
}

fn fmt_pace(secs: i64) -> String {
    let a = secs.unsigned_abs();
    let sign = if secs >= 0 { '+' } else { '-' };
    let txt = if a >= 86400 {
        format!("{}d{}h", a / 86400, (a % 86400) / 3600)
    } else if a >= 3600 {
        format!("{}h{:02}m", a / 3600, (a % 3600) / 60)
    } else if a >= 60 {
        format!("{}m", a / 60)
    } else {
        "0m".into()
    };
    format!("{sign}{txt}")
}

fn pace_balance_secs(used: f64, remaining: i64, window: i64) -> Option<i64> {
    let elapsed = window - remaining;
    if elapsed < 60 {
        return None;
    }
    let bal_pct = (100.0 - used) - (remaining as f64 / window as f64) * 100.0;
    Some((bal_pct * window as f64 / 100.0) as i64)
}

fn pace_color(secs: i64, window: i64) -> Color {
    if secs >= 0 {
        return CYAN;
    }
    let pct = (secs.unsigned_abs() as f64) / (window as f64) * 100.0;
    if pct >= 15.0 {
        RED
    } else if pct >= 8.0 {
        ORANGE
    } else {
        YELLOW
    }
}

fn quota_color(used: f64, remaining: i64, window: i64) -> Color {
    if window <= 0 || remaining <= 0 {
        if used >= 80.0 {
            return RED;
        }
        if used >= 50.0 {
            return ORANGE;
        }
        return CYAN;
    }
    let elapsed_pct = ((window - remaining) as f64 / window as f64) * 100.0;
    if used <= elapsed_pct * 1.1 {
        CYAN
    } else if used <= elapsed_pct * 1.5 {
        ORANGE
    } else {
        RED
    }
}

fn rgb_of(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0xff, 0xff, 0xff),
    }
}

/// Linear blend of two RGB colors. `t` is the weight of `b` (0.0 → a, 1.0 → b).
fn blend(a: Color, b: Color, t: f32) -> Color {
    let (ar, ag, ab) = rgb_of(a);
    let (br, bg, bb) = rgb_of(b);
    let mix = |x: u8, y: u8| ((x as f32) * (1.0 - t) + (y as f32) * t).round() as u8;
    Color::Rgb(mix(ar, br), mix(ag, bg), mix(ab, bb))
}

// ── Collectors ───────────────────────────────────────────────────────────

fn claude_log() -> PathBuf {
    env::temp_dir().join("claude-usage-log.tsv")
}

fn copilot_log() -> PathBuf {
    env::temp_dir().join("copilot-usage-log.tsv")
}

fn codex_log() -> PathBuf {
    env::temp_dir().join("codex-usage-log.tsv")
}

struct DualSample {
    ts: i64,
    p_pct: f64,
    p_reset: i64,
    s_pct: f64,
    s_reset: i64,
}

/// Parse a 5-field TSV `ts\tprimary_pct\tprimary_reset\tsecondary_pct\tsecondary_reset`.
fn load_dual(path: &PathBuf) -> Vec<DualSample> {
    let data = fs::read_to_string(path).unwrap_or_default();
    let mut out = Vec::with_capacity(256);
    for line in data.lines() {
        let mut it = line.split('\t');
        let (Some(t), Some(pp), Some(pr), Some(sp), Some(sr)) =
            (it.next(), it.next(), it.next(), it.next(), it.next())
        else {
            continue;
        };
        let (Ok(ts), Ok(p_pct), Ok(p_reset), Ok(s_pct), Ok(s_reset)) = (
            t.parse::<i64>(),
            pp.parse::<f64>(),
            pr.parse::<f64>(),
            sp.parse::<f64>(),
            sr.parse::<i64>(),
        ) else {
            continue;
        };
        out.push(DualSample {
            ts,
            p_pct,
            p_reset: p_reset as i64,
            s_pct,
            s_reset,
        });
    }
    out
}

fn load_simple(path: &PathBuf) -> Option<(f64, i64)> {
    let data = fs::read_to_string(path).ok()?;
    let line = data.lines().next_back()?;
    let mut it = line.split('\t');
    let _ts: i64 = it.next()?.parse().ok()?;
    let pct: f64 = it.next()?.parse().ok()?;
    let reset: i64 = it.next()?.parse().ok()?;
    Some((pct, reset))
}

fn dual_window_bars(
    path: &PathBuf,
    label_prefix: &str,
    provider: Color,
    short_window: i64,
    short_label: &str,
    long_window: i64,
    long_label: &str,
) -> Vec<Bar> {
    let samples = load_dual(path);
    if samples.is_empty() {
        return Vec::new();
    }
    let p_reset = samples.iter().map(|s| s.p_reset).max().unwrap_or(0);
    let s_reset = samples.iter().map(|s| s.s_reset).max().unwrap_or(0);
    let p_pct = samples
        .iter()
        .filter(|s| s.p_reset == p_reset && s.ts >= p_reset - short_window)
        .map(|s| s.p_pct)
        .fold(0.0_f64, f64::max);
    let s_pct = samples
        .iter()
        .filter(|s| s.s_reset == s_reset && s.ts >= s_reset - long_window)
        .map(|s| s.s_pct)
        .fold(0.0_f64, f64::max);
    vec![
        Bar {
            label: format!("{label_prefix} {short_label}"),
            pct: p_pct,
            window_secs: short_window,
            reset_ts: p_reset,
            provider,
        },
        Bar {
            label: format!("{label_prefix} {long_label}"),
            pct: s_pct,
            window_secs: long_window,
            reset_ts: s_reset,
            provider,
        },
    ]
}

fn copilot_bars() -> Vec<Bar> {
    let Some((pct, reset)) = load_simple(&copilot_log()) else {
        return Vec::new();
    };
    vec![Bar {
        label: "copilot".into(),
        pct,
        window_secs: THIRTY_DAYS,
        reset_ts: reset,
        provider: COPILOT_COLOR,
    }]
}

pub fn collect() -> Vec<Bar> {
    let mut out = Vec::new();
    out.extend(dual_window_bars(
        &claude_log(),
        "claude",
        CLAUDE_COLOR,
        FIVE_HOURS,
        "5h",
        SEVEN_DAYS,
        "7d",
    ));
    out.extend(copilot_bars());
    out.extend(dual_window_bars(
        &codex_log(),
        "codex",
        CODEX_COLOR,
        FIVE_HOURS,
        "5h",
        SEVEN_DAYS,
        "7d",
    ));
    out
}

// ── Rendering ────────────────────────────────────────────────────────────

const ROWS_PER_BAR: u16 = 2;
const BAR_TRACK: Color = Color::Rgb(0x3a, 0x3d, 0x4e);
/// Weight of provider color when blending with burn-state color for the fill.
const TINT: f32 = 0.35;
/// Weight of bg when dimming every fg in the usage section (higher = dimmer).
const DIM_MIX: f32 = 0.32;

fn dim(fg: Color, bg: Color) -> Color {
    blend(fg, bg, DIM_MIX)
}

fn bar_cells(
    width: usize,
    used_pct: f64,
    elapsed_pct: f64,
    fill_color: Color,
    bg: Color,
) -> Vec<Span<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let remaining_pct = (100.0 - used_pct).clamp(0.0, 100.0);
    let expected_remaining_pct = (100.0 - elapsed_pct).clamp(0.0, 100.0);
    let remaining_cells = ((remaining_pct / 100.0) * width as f64)
        .round()
        .clamp(0.0, width as f64) as usize;
    let tick_cell = ((expected_remaining_pct / 100.0) * width as f64)
        .round()
        .clamp(0.0, (width.saturating_sub(1)) as f64) as usize;

    // Tick color reflects pacing: green if fill is past the tick (surplus),
    // orange for small deficit (<3%), red beyond.
    let deficit = used_pct - elapsed_pct;
    let tick_color = if deficit <= 0.0 {
        GREEN
    } else if deficit < 3.0 {
        ORANGE
    } else {
        RED
    };

    let mut spans: Vec<Span<'static>> = Vec::with_capacity(width);
    for i in 0..width {
        let is_tick = i == tick_cell;
        let filled = i < remaining_cells;
        let (ch, fg) = match (filled, is_tick) {
            (true, true) => ("│", tick_color),
            (true, false) => ("▓", fill_color),
            (false, true) => ("│", tick_color),
            (false, false) => ("░", BAR_TRACK),
        };
        spans.push(Span::styled(
            ch.to_string(),
            Style::default().fg(dim(fg, bg)).bg(bg),
        ));
    }
    spans
}

fn draw_bar(f: &mut Frame, rect: Rect, bg: Color, b: &Bar) {
    if rect.height < ROWS_PER_BAR || rect.width < 12 {
        return;
    }
    let now = now_ts();
    let remaining = (b.reset_ts - now).max(0);
    let elapsed_pct = if b.window_secs > 0 {
        ((b.window_secs - remaining) as f64 / b.window_secs as f64) * 100.0
    } else {
        0.0
    };
    let burn = quota_color(b.pct, remaining, b.window_secs);
    let fill = blend(burn, b.provider, TINT);
    let pace = pace_balance_secs(b.pct, remaining, b.window_secs);

    // Row 1 — stats: `label   pct   ...   pace ↺reset`
    let total_w = rect.width as usize;
    let pct_txt = format!("{}%", b.pct.round() as i64);
    let pace_txt = pace.map(fmt_pace).unwrap_or_default();
    let reset_txt = if remaining > 0 {
        format!("↺{}", fmt_reset(remaining))
    } else {
        String::new()
    };
    let left_len = 1 + b.label.chars().count() + 2 + pct_txt.chars().count();
    let right_len = if !pace_txt.is_empty() && !reset_txt.is_empty() {
        pace_txt.chars().count() + 1 + reset_txt.chars().count() + 1
    } else {
        pace_txt.chars().count() + reset_txt.chars().count() + 1
    };
    let pad = total_w.saturating_sub(left_len + right_len);

    let mut stats: Vec<Span<'static>> = Vec::new();
    stats.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
    stats.push(Span::styled(
        b.label.clone(),
        Style::default().fg(dim(b.provider, bg)).bg(bg),
    ));
    stats.push(Span::styled("  ".to_string(), Style::default().bg(bg)));
    stats.push(Span::styled(
        pct_txt,
        Style::default().fg(dim(burn, bg)).bg(bg),
    ));
    stats.push(Span::styled(" ".repeat(pad), Style::default().bg(bg)));
    if let Some(p) = pace {
        stats.push(Span::styled(
            fmt_pace(p),
            Style::default()
                .fg(dim(pace_color(p, b.window_secs), bg))
                .bg(bg),
        ));
        stats.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
    }
    stats.push(Span::styled(
        reset_txt,
        Style::default().fg(dim(DIM, bg)).bg(bg),
    ));
    stats.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
    f.render_widget(
        Paragraph::new(Line::from(stats)).style(Style::default().bg(bg)),
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 1,
        },
    );

    // Row 2 — full-width depleting bar with 1-cell side padding.
    let bar_w = total_w.saturating_sub(2);
    let mut bar_spans: Vec<Span<'static>> =
        vec![Span::styled(" ".to_string(), Style::default().bg(bg))];
    bar_spans.extend(bar_cells(bar_w, b.pct, elapsed_pct, fill, bg));
    bar_spans.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
    f.render_widget(
        Paragraph::new(Line::from(bar_spans)).style(Style::default().bg(bg)),
        Rect {
            x: rect.x,
            y: rect.y + 1,
            width: rect.width,
            height: 1,
        },
    );
}

pub fn height(n_bars: usize) -> u16 {
    n_bars as u16 * ROWS_PER_BAR
}

pub fn draw(f: &mut Frame, area: Rect, bg: Color, bars: &[Bar]) {
    if bars.is_empty() || area.height == 0 {
        return;
    }
    let capacity = (area.height / ROWS_PER_BAR) as usize;
    for (i, b) in bars.iter().take(capacity).enumerate() {
        let row = Rect {
            x: area.x,
            y: area.y + (i as u16) * ROWS_PER_BAR,
            width: area.width,
            height: ROWS_PER_BAR,
        };
        draw_bar(f, row, bg, b);
    }
}
