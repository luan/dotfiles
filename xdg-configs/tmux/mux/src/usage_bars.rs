use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use rusqlite::{Connection, OpenFlags};

const FIVE_HOURS: i64 = 5 * 3600;
const SEVEN_DAYS: i64 = 7 * 24 * 3600;
const THIRTY_DAYS: i64 = 30 * 24 * 3600;

const DIM: Color = Color::Rgb(0x6c, 0x70, 0x86);
const WHITE: Color = Color::Rgb(0xff, 0xff, 0xff);
const BRIGHT_RED: Color = Color::Rgb(0xff, 0x00, 0x00);
const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const ORANGE: Color = Color::Rgb(0xfa, 0xb3, 0x87);
const RED: Color = Color::Rgb(0xef, 0x44, 0x44);

// Provider identity colors (used for labels and to tint the burn color).
const CLAUDE_COLOR: Color = Color::Rgb(0xd8, 0x7b, 0x4a); // warm amber
const COPILOT_COLOR: Color = Color::Rgb(0xcb, 0xa6, 0xf7); // mauve
const CODEX_COLOR: Color = Color::Rgb(0x74, 0xc7, 0xec); // sky blue

// Font Awesome 7 Brands glyphs (resolved via WezTerm's FA 7 Brands fallback).
const CLAUDE_GLYPH: &str = "\u{e861}";
const CODEX_GLYPH: &str = "\u{e7cf}";
const COPILOT_GLYPH: &str = "\u{f113}";

pub(crate) struct Bar {
    /// Stable identity key for pulse hashmap + cross-file lookups. Retains the
    /// textual provider name ("claude 5h", "copilot", ...) so external callers
    /// (sidebar/mod.rs) can reason by name.
    pub(crate) label: String,
    /// Rendered prefix — provider glyph plus optional window suffix
    /// ("\u{e861} 5h"). Distinct from `label` so display tweaks don't ripple
    /// into identity.
    pub(crate) display: String,
    pub(crate) pct: f64,
    pub(crate) window_secs: i64,
    pub(crate) reset_ts: i64,
    /// Timestamp of most recent sample — drives recency dimming.
    pub(crate) last_ts: i64,
    pub(crate) provider: Color,
    /// Dollar overage for this bar's window (Claude only). None = don't render.
    pub(crate) overage: Option<f64>,
    /// Cache hit rate in [0.0, 1.0] for this bar's window (Claude only).
    /// None = don't render.
    pub(crate) hit_rate: Option<f64>,
}

/// Claude-specific overage data read from
/// `~/.local/state/claude-statusline/usage.db` (SQLite `windows` table).
#[derive(Clone, Debug, Default)]
pub(crate) struct ClaudeOverage {
    pub(crate) five_h: f64,
    pub(crate) seven_d: f64,
    pub(crate) month: f64,
    pub(crate) total: f64,
}

fn open_claude_db() -> Option<Connection> {
    let path = crate::tmux::home().join(".local/state/claude-statusline/usage.db");
    if !path.exists() {
        return None;
    }
    Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()
}

fn load_overage() -> Option<ClaudeOverage> {
    let conn = open_claude_db()?;
    // Pull the latest (max reset_ts) overage per kind in one query.
    let query = "SELECT \
         COALESCE((SELECT overage FROM windows WHERE kind='5h'    ORDER BY reset_ts DESC LIMIT 1), 0), \
         COALESCE((SELECT overage FROM windows WHERE kind='7d'    ORDER BY reset_ts DESC LIMIT 1), 0), \
         COALESCE((SELECT overage FROM windows WHERE kind='month' ORDER BY reset_ts DESC LIMIT 1), 0), \
         COALESCE((SELECT overage FROM windows WHERE kind='total' ORDER BY reset_ts DESC LIMIT 1), 0)";
    conn.query_row(query, [], |row| {
        Ok(ClaudeOverage {
            five_h: row.get(0)?,
            seven_d: row.get(1)?,
            month: row.get(2)?,
            total: row.get(3)?,
        })
    })
    .ok()
}

/// Cache hit rate in [0.0, 1.0] over the event window `ts > since_ts`, read
/// from claude-statusline's `events` table. `since_ts` is typically the bar's
/// `reset_ts - window_secs` (start of the current billing window).
fn load_hit_rate(since_ts: i64) -> Option<f64> {
    let conn = open_claude_db()?;
    let query = "SELECT SUM(cache_read_tokens) * 1.0 / \
         NULLIF(SUM(cache_read_tokens + cache_creation_tokens + input_tokens), 0) \
         FROM events WHERE ts > ?1";
    conn.query_row(query, [since_ts], |row| row.get::<_, Option<f64>>(0))
        .ok()
        .flatten()
}

fn fmt_usd(v: f64) -> String {
    format!("+${:.2}", v.max(0.0))
}

/// Color anchor for dollar overage values. Ramps DIM → YELLOW → ORANGE as the
/// value climbs toward $100. Saturates at ORANGE for values ≥ $100.
fn overage_color(v: f64) -> Color {
    let t = (v / 100.0).clamp(0.0, 1.0) as f32;
    if t <= 0.0 {
        DIM
    } else if t < 0.5 {
        blend(DIM, YELLOW, t * 2.0)
    } else {
        blend(YELLOW, ORANGE, (t - 0.5) * 2.0)
    }
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

/// Map urgency in [0, 1] onto the provider identity.
/// 0 → provider (no urgency), mid → orange-tinted provider, 1 → red-tinted
/// provider. The red endpoint keeps a 25% provider hint so each provider's
/// "critical" reads distinctly instead of converging on one shared red.
fn urgency_tint(provider: Color, urgency: f32) -> Color {
    let u = urgency.clamp(0.0, 1.0);
    let red_end = blend(RED, provider, 0.25);
    if u < 0.5 {
        blend(provider, ORANGE, u * 2.0 * 0.75)
    } else {
        let mid = blend(provider, ORANGE, 0.75);
        blend(mid, red_end, (u - 0.5) * 2.0)
    }
}

fn pace_color(secs: i64, window: i64, provider: Color) -> Color {
    if secs >= 0 {
        return provider;
    }
    let pct = (secs.unsigned_abs() as f64) / (window as f64) * 100.0;
    // Map deficit % to urgency: 0% → 0, 15%+ → 1.
    let urgency = (pct / 15.0).clamp(0.0, 1.0) as f32;
    urgency_tint(provider, urgency)
}

fn quota_color(used: f64, remaining: i64, window: i64, provider: Color) -> Color {
    if window <= 0 || remaining <= 0 {
        // Window closed: urgency = used / 100.
        let urgency = (used / 100.0).clamp(0.0, 1.0) as f32;
        return urgency_tint(provider, urgency);
    }
    let elapsed_pct = ((window - remaining) as f64 / window as f64) * 100.0;
    // Overshoot ratio: 1.0 == on-pace. Map 1.0→0 urgency, 1.5→1.0 urgency.
    let ratio = if elapsed_pct > 0.0 {
        used / elapsed_pct
    } else if used > 0.0 {
        f64::INFINITY
    } else {
        1.0
    };
    let urgency = ((ratio - 1.0) / 0.5).clamp(0.0, 1.0) as f32;
    urgency_tint(provider, urgency)
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

fn claude_usage_db() -> PathBuf {
    crate::tmux::home().join(".local/state/claude-statusline/usage.db")
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
fn load_dual_sqlite(db_path: &std::path::Path) -> Vec<DualSample> {
    if !db_path.exists() {
        return Vec::new();
    }
    let Ok(conn) = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) else {
        return Vec::new();
    };
    let Ok(mut stmt) = conn
        .prepare("SELECT ts, fh_used, fh_reset, sd_used, sd_reset FROM usage_samples ORDER BY ts ASC")
    else {
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
        ))
    }) else {
        return Vec::new();
    };
    const SANE_MAX_TS: i64 = 4_102_444_800; // 2100-01-01 UTC
    let mut samples = Vec::with_capacity(256);
    for row in rows.flatten() {
        let (ts, fh_used, fh_reset, sd_used, sd_reset) = row;
        if !(0..SANE_MAX_TS).contains(&fh_reset) || !(0..SANE_MAX_TS).contains(&sd_reset) {
            continue;
        }
        samples.push(DualSample {
            ts,
            p_pct: fh_used as f64,
            p_reset: fh_reset,
            s_pct: sd_used as f64,
            s_reset: sd_reset,
        });
    }
    samples
}

fn load_dual(path: &std::path::Path) -> Vec<DualSample> {
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
            pr.parse::<i64>(),
            sp.parse::<f64>(),
            sr.parse::<i64>(),
        ) else {
            // Row corrupt (concurrent-write collision concatenates columns,
            // overflows i64). Skip rather than poison downstream max() reducers.
            continue;
        };
        // Reject rows with out-of-range reset timestamps — year 2100 is a
        // sanity ceiling. Real values are always current-era unix seconds.
        const SANE_MAX_TS: i64 = 4_102_444_800; // 2100-01-01 UTC
        if !(0..SANE_MAX_TS).contains(&p_reset) || !(0..SANE_MAX_TS).contains(&s_reset) {
            continue;
        }
        out.push(DualSample {
            ts,
            p_pct,
            p_reset,
            s_pct,
            s_reset,
        });
    }
    out
}

struct SimpleSample {
    ts: i64,
    pct: f64,
    reset: i64,
}

fn load_simple(path: &std::path::Path) -> Vec<SimpleSample> {
    let data = fs::read_to_string(path).unwrap_or_default();
    let mut out = Vec::with_capacity(64);
    for line in data.lines() {
        let mut it = line.split('\t');
        let (Some(t), Some(p), Some(r)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        let (Ok(ts), Ok(pct), Ok(reset)) = (t.parse::<i64>(), p.parse::<f64>(), r.parse::<i64>())
        else {
            continue;
        };
        out.push(SimpleSample { ts, pct, reset });
    }
    out
}

/// Timestamp of the last sample whose pct strictly exceeded the previous
/// sample's, within the same reset window. Heartbeat samples (no change) and
/// reset-window boundaries (pct drops) don't count as activity. Falls back to
/// the first sample's ts if no increase was ever observed.
fn last_change_ts<I: IntoIterator<Item = (i64, f64, i64)>>(series: I) -> i64 {
    let mut prev: Option<(f64, i64)> = None;
    let mut latest_change = 0_i64;
    let mut first_ts = 0_i64;
    for (ts, pct, reset) in series {
        if first_ts == 0 {
            first_ts = ts;
        }
        if let Some((p_pct, p_reset)) = prev
            && reset == p_reset
            && pct > p_pct
        {
            latest_change = ts;
        }
        prev = Some((pct, reset));
    }
    if latest_change > 0 {
        latest_change
    } else {
        first_ts
    }
}

fn dual_window_bars(
    samples: &[DualSample],
    label_prefix: &str,
    glyph: &str,
    provider: Color,
) -> Vec<Bar> {
    if samples.is_empty() {
        return Vec::new();
    }
    let p_reset = samples.iter().map(|s| s.p_reset).max().unwrap_or(0);
    let s_reset = samples.iter().map(|s| s.s_reset).max().unwrap_or(0);
    let p_change = last_change_ts(samples.iter().map(|s| (s.ts, s.p_pct, s.p_reset)));
    let s_change = last_change_ts(samples.iter().map(|s| (s.ts, s.s_pct, s.s_reset)));
    let last_ts = p_change.max(s_change);
    let p_pct = samples
        .iter()
        .filter(|s| s.p_reset == p_reset && s.ts >= p_reset - FIVE_HOURS)
        .map(|s| s.p_pct)
        .fold(0.0_f64, f64::max);
    let s_pct = samples
        .iter()
        .filter(|s| s.s_reset == s_reset && s.ts >= s_reset - SEVEN_DAYS)
        .map(|s| s.s_pct)
        .fold(0.0_f64, f64::max);
    vec![
        Bar {
            label: format!("{label_prefix} 5h"),
            display: format!("{glyph} 5h"),
            pct: p_pct,
            window_secs: FIVE_HOURS,
            reset_ts: p_reset,
            last_ts,
            provider,
            overage: None,
            hit_rate: None,
        },
        Bar {
            label: format!("{label_prefix} 7d"),
            display: format!("{glyph} 7d"),
            pct: s_pct,
            window_secs: SEVEN_DAYS,
            reset_ts: s_reset,
            last_ts,
            provider,
            overage: None,
            hit_rate: None,
        },
    ]
}

fn copilot_bars() -> Vec<Bar> {
    let samples = load_simple(&copilot_log());
    let Some(last) = samples.last() else {
        return Vec::new();
    };
    let last_ts = last_change_ts(samples.iter().map(|s| (s.ts, s.pct, s.reset)));
    vec![Bar {
        label: "copilot".into(),
        // Trailing space: the FA 7 Brands github-alt glyph renders slightly
        // wider than one cell and optically collides with the following pct
        // number. Other rows have "5h"/"7d" soaking that up; copilot doesn't.
        display: format!("{COPILOT_GLYPH} "),
        pct: last.pct,
        window_secs: THIRTY_DAYS,
        reset_ts: last.reset,
        last_ts,
        provider: COPILOT_COLOR,
        overage: None,
        hit_rate: None,
    }]
}

pub(crate) struct Snapshot {
    pub(crate) bars: Vec<Bar>,
    pub(crate) overage: Option<ClaudeOverage>,
}

pub(crate) fn collect() -> Snapshot {
    let overage = load_overage();
    let claude_samples = load_dual_sqlite(&claude_usage_db());
    let codex_samples = load_dual(&codex_log());
    let mut bars = Vec::new();
    let mut claude = dual_window_bars(&claude_samples, "claude", CLAUDE_GLYPH, CLAUDE_COLOR);
    // Attach overage per claude window. Hide $0 — only non-zero shows.
    if let Some(ref ov) = overage {
        if let Some(b) = claude.get_mut(0)
            && ov.five_h > 0.0
        {
            b.overage = Some(ov.five_h);
        }
        if let Some(b) = claude.get_mut(1)
            && ov.seven_d > 0.0
        {
            b.overage = Some(ov.seven_d);
        }
    }
    // Cache hit rate over each bar's window start (reset_ts - window).
    if let Some(b) = claude.get_mut(0) {
        b.hit_rate = load_hit_rate(b.reset_ts - FIVE_HOURS);
    }
    if let Some(b) = claude.get_mut(1) {
        b.hit_rate = load_hit_rate(b.reset_ts - SEVEN_DAYS);
    }
    bars.extend(claude);
    bars.extend(copilot_bars());
    bars.extend(dual_window_bars(
        &codex_samples,
        "codex",
        CODEX_GLYPH,
        CODEX_COLOR,
    ));
    Snapshot { bars, overage }
}

// ── Rendering ────────────────────────────────────────────────────────────

pub(crate) const ROWS_PER_BAR: u16 = 2;
const BAR_TRACK: Color = Color::Rgb(0x3a, 0x3d, 0x4e);

/// Weight of bg when dimming every fg in the usage section. Scales linearly
/// from `MIN_MIX` (fresh, vivid) to `MAX_MIX` (stale, very dim) across a 4h
/// horizon. Lower = brighter. Tightened from 24h: at 24h the spread was
/// perceptually subtle on dark themes; "haven't used in a few hours" should
/// already register as visibly faded.
fn recency_mix(age_secs: i64) -> f32 {
    const MIN_MIX: f32 = 0.18;
    const MAX_MIX: f32 = 0.85;
    const HORIZON: f32 = 4.0 * 3600.0;
    let t = (age_secs.max(0) as f32 / HORIZON).clamp(0.0, 1.0);
    MIN_MIX + (MAX_MIX - MIN_MIX) * t
}

/// Softer ramp for the provider glyph — the icon is the row's identity anchor,
/// so it should stay recognizable even for stale providers. Same horizon,
/// tighter MAX so inactive providers read as present-but-quiet rather than
/// near-invisible.
fn glyph_recency_mix(age_secs: i64) -> f32 {
    const MIN_MIX: f32 = 0.10;
    const MAX_MIX: f32 = 0.45;
    const HORIZON: f32 = 4.0 * 3600.0;
    let t = (age_secs.max(0) as f32 / HORIZON).clamp(0.0, 1.0);
    MIN_MIX + (MAX_MIX - MIN_MIX) * t
}

/// Pulse animation: triangle wave 0→1→0 over PULSE_DURATION. Returns the
/// blend weight to apply when blending the fill color toward WHITE. Returns
/// None if no pulse is active.
pub(crate) const PULSE_DURATION: Duration = Duration::from_millis(1500);
fn pulse_factor(started: Instant, now: Instant) -> Option<f32> {
    let elapsed = now.duration_since(started);
    if elapsed >= PULSE_DURATION {
        return None;
    }
    let t = elapsed.as_secs_f32() / PULSE_DURATION.as_secs_f32();
    Some(1.0 - (2.0 * t - 1.0).abs())
}

fn dim(fg: Color, bg: Color, mix: f32) -> Color {
    blend(fg, bg, mix)
}

fn bar_cells(
    width: usize,
    used_pct: f64,
    elapsed_pct: f64,
    fill_color: Color,
    bg: Color,
    mix: f32,
    pulse: Option<f32>,
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
        // Pulse: blend filled cells (fill + tick on filled side) toward white.
        // Track cells stay quiet — pulsing the empty track is just visual noise.
        let pulsed = match (filled, pulse) {
            (true, Some(p)) => blend(fg, WHITE, p),
            _ => fg,
        };
        spans.push(Span::styled(
            ch.to_string(),
            Style::default().fg(dim(pulsed, bg, mix)).bg(bg),
        ));
    }
    spans
}

fn draw_bar(
    f: &mut Frame,
    rect: Rect,
    bg: Color,
    b: &Bar,
    pulse: Option<f32>,
    over_pulse: Option<f32>,
) {
    if rect.height < ROWS_PER_BAR || rect.width < 12 {
        return;
    }
    let now = now_ts();
    let age = now - b.last_ts;
    let mix = recency_mix(age);
    let glyph_mix = glyph_recency_mix(age);
    let remaining = (b.reset_ts - now).max(0);
    let elapsed_pct = if b.window_secs > 0 {
        ((b.window_secs - remaining) as f64 / b.window_secs as f64) * 100.0
    } else {
        0.0
    };
    // `burn` already carries the provider identity; urgency shifts it toward
    // orange/red. Use it directly as the fill so the number and the bar agree.
    let burn = quota_color(b.pct, remaining, b.window_secs, b.provider);
    let fill = burn;
    let pace = pace_balance_secs(b.pct, remaining, b.window_secs);

    // Row 1 — stats: `label   pct  [+$over]   ...   pace ↺reset`
    // pct shown is REMAINING (100 → 0), not used.
    let total_w = rect.width as usize;
    let remaining_pct = (100.0 - b.pct).clamp(0.0, 100.0);
    let pct_txt = format!("{}%", remaining_pct.round() as i64);
    let pace_txt = pace.map(fmt_pace).unwrap_or_default();
    let reset_txt = if remaining > 0 {
        format!("↺{}", fmt_reset(remaining))
    } else {
        String::new()
    };
    let over_txt = b.overage.map(fmt_usd).unwrap_or_default();
    let base_over_color = b.overage.map(overage_color).unwrap_or(DIM);
    let over_color = match over_pulse {
        Some(p) => blend(base_over_color, BRIGHT_RED, p),
        None => base_over_color,
    };
    let over_block_w = if over_txt.is_empty() {
        0
    } else {
        over_txt.chars().count() + 1 // " " separator before the amount
    };
    // Hide near-perfect cache rates — 99%+ is the boring healthy baseline,
    // only the shortfall earns pixels.
    let hit_txt = b
        .hit_rate
        .filter(|r| *r < 0.99)
        .map(|r| format!("󰆼 {}%", (r * 100.0).round() as i64))
        .unwrap_or_default();
    let hit_block_w = if hit_txt.is_empty() {
        0
    } else {
        hit_txt.chars().count() + 1 // " " separator
    };
    let left_len =
        1 + b.display.chars().count() + 2 + pct_txt.chars().count() + over_block_w + hit_block_w;
    let right_len = if !pace_txt.is_empty() && !reset_txt.is_empty() {
        pace_txt.chars().count() + 1 + reset_txt.chars().count() + 1
    } else {
        pace_txt.chars().count() + reset_txt.chars().count() + 1
    };
    let pad = total_w.saturating_sub(left_len + right_len);

    let mut stats: Vec<Span<'static>> =
        vec![Span::styled(" ".to_string(), Style::default().bg(bg))];
    stats.push(Span::styled(
        b.display.clone(),
        Style::default().fg(dim(b.provider, bg, glyph_mix)).bg(bg),
    ));
    stats.push(Span::styled("  ".to_string(), Style::default().bg(bg)));
    stats.push(Span::styled(
        pct_txt,
        Style::default().fg(dim(burn, bg, mix)).bg(bg),
    ));
    if !over_txt.is_empty() {
        stats.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
        stats.push(Span::styled(
            over_txt,
            Style::default().fg(dim(over_color, bg, mix)).bg(bg),
        ));
    }
    if !hit_txt.is_empty() {
        stats.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
        stats.push(Span::styled(
            hit_txt,
            Style::default().fg(dim(DIM, bg, mix)).bg(bg),
        ));
    }
    stats.push(Span::styled(" ".repeat(pad), Style::default().bg(bg)));
    if let Some(p) = pace {
        stats.push(Span::styled(
            fmt_pace(p),
            Style::default()
                .fg(dim(pace_color(p, b.window_secs, b.provider), bg, mix))
                .bg(bg),
        ));
        stats.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
    }
    stats.push(Span::styled(
        reset_txt,
        Style::default().fg(dim(DIM, bg, mix)).bg(bg),
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
    bar_spans.extend(bar_cells(bar_w, b.pct, elapsed_pct, fill, bg, mix, pulse));
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

pub(crate) fn height(n_bars: usize, has_overage_footer: bool) -> u16 {
    n_bars as u16 * ROWS_PER_BAR + if has_overage_footer { 1 } else { 0 }
}

pub(crate) fn draw(
    f: &mut Frame,
    area: Rect,
    bg: Color,
    bars: &[Bar],
    pulses: &HashMap<String, Instant>,
    overage: Option<&ClaudeOverage>,
) {
    if bars.is_empty() || area.height == 0 {
        return;
    }
    let now = Instant::now();
    let last_claude = bars.iter().rposition(|b| b.label.starts_with("claude"));
    let has_footer = overage
        .map(|ov| ov.month > 0.0 || ov.total > 0.0)
        .unwrap_or(false)
        && last_claude.is_some();
    let mut y = area.y;
    let end_y = area.y + area.height;
    for (i, b) in bars.iter().enumerate() {
        if y + ROWS_PER_BAR > end_y {
            return;
        }
        let row = Rect {
            x: area.x,
            y,
            width: area.width,
            height: ROWS_PER_BAR,
        };
        let pulse = pulses.get(&b.label).and_then(|s| pulse_factor(*s, now));
        let over_key = format!("over:{}", b.label);
        let over_pulse = pulses.get(&over_key).and_then(|s| pulse_factor(*s, now));
        draw_bar(f, row, bg, b, pulse, over_pulse);
        y += ROWS_PER_BAR;
        if has_footer && Some(i) == last_claude {
            if y + 1 > end_y {
                return;
            }
            if let Some(ov) = overage {
                let mo_pulse = pulses.get("over:mo").and_then(|s| pulse_factor(*s, now));
                let total_pulse = pulses.get("over:total").and_then(|s| pulse_factor(*s, now));
                draw_claude_overage_footer(
                    f,
                    Rect {
                        x: area.x,
                        y,
                        width: area.width,
                        height: 1,
                    },
                    bg,
                    ov,
                    mo_pulse,
                    total_pulse,
                );
            }
            y += 1;
        }
    }
}

fn draw_claude_overage_footer(
    f: &mut Frame,
    rect: Rect,
    bg: Color,
    ov: &ClaudeOverage,
    mo_pulse: Option<f32>,
    total_pulse: Option<f32>,
) {
    let show_mo = ov.month > 0.0;
    let show_total = ov.total > 0.0;
    if !show_mo && !show_total {
        return;
    }
    let mo_txt = if show_mo {
        format!("mo {}", fmt_usd(ov.month))
    } else {
        String::new()
    };
    let total_txt = if show_total {
        format!("total {}", fmt_usd(ov.total))
    } else {
        String::new()
    };
    let mo_color = match mo_pulse {
        Some(p) => blend(overage_color(ov.month), BRIGHT_RED, p),
        None => overage_color(ov.month),
    };
    let total_color = match total_pulse {
        Some(p) => blend(overage_color(ov.total), BRIGHT_RED, p),
        None => overage_color(ov.total),
    };
    let total_w = rect.width as usize;
    let mut spans: Vec<Span<'static>> =
        vec![Span::styled(" ".to_string(), Style::default().bg(bg))];
    if show_mo && show_total {
        let gap = total_w
            .saturating_sub(1 + mo_txt.chars().count() + total_txt.chars().count() + 1)
            .max(2);
        spans.push(Span::styled(mo_txt, Style::default().fg(mo_color).bg(bg)));
        spans.push(Span::styled(" ".repeat(gap), Style::default().bg(bg)));
        spans.push(Span::styled(
            total_txt,
            Style::default().fg(total_color).bg(bg),
        ));
    } else if show_mo {
        spans.push(Span::styled(mo_txt, Style::default().fg(mo_color).bg(bg)));
    } else {
        spans.push(Span::styled(
            total_txt,
            Style::default().fg(total_color).bg(bg),
        ));
    }
    spans.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
        rect,
    );
}
