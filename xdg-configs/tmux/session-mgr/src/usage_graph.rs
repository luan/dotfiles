use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Paragraph};

const FIVE_HOURS: i64 = 5 * 3600;
const SEVEN_DAYS: i64 = 7 * 24 * 3600;

const DIM: Color = Color::Rgb(0x6c, 0x70, 0x86);
const CYAN: Color = Color::Rgb(0x89, 0xdc, 0xeb);
const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const ORANGE: Color = Color::Rgb(0xfa, 0xb3, 0x87);
const RED: Color = Color::Rgb(0xf3, 0x8b, 0xa8);
const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
const BLUE: Color = Color::Rgb(0x89, 0xb4, 0xfa);

pub const HEIGHT: u16 = 12;

fn log_path() -> PathBuf {
    env::temp_dir().join("claude-usage-log.tsv")
}

#[derive(Clone, Copy)]
struct Sample {
    ts: i64,
    fh_pct: f64,
    fh_reset: i64,
    sd_pct: f64,
    sd_reset: i64,
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn load_samples() -> Vec<Sample> {
    let data = fs::read_to_string(log_path()).unwrap_or_default();
    let mut out = Vec::with_capacity(256);
    for line in data.lines() {
        let mut it = line.split('\t');
        let (Some(t), Some(fp), Some(fr), Some(sp), Some(sr)) =
            (it.next(), it.next(), it.next(), it.next(), it.next())
        else {
            continue;
        };
        let (Ok(ts), Ok(fh_pct), Ok(fh_reset), Ok(sd_pct), Ok(sd_reset)) = (
            t.parse::<i64>(),
            fp.parse::<f64>(),
            fr.parse::<i64>(),
            sp.parse::<f64>(),
            sr.parse::<i64>(),
        ) else {
            continue;
        };
        out.push(Sample {
            ts,
            fh_pct,
            fh_reset,
            sd_pct,
            sd_reset,
        });
    }
    out
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

fn pace_balance_secs(used: f64, remaining: i64, window: i64) -> Option<i64> {
    let elapsed = window - remaining;
    if elapsed < 60 {
        return None;
    }
    let bal_pct = (100.0 - used) - (remaining as f64 / window as f64) * 100.0;
    Some((bal_pct * window as f64 / 100.0) as i64)
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
        "0m".to_string()
    };
    format!("{sign}{txt}")
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

// ── Chart ────────────────────────────────────────────────────────────────

struct Block<'a> {
    label: &'a str,
    window: i64,
    points: &'a [(i64, f64)],
    latest_reset: i64,
    latest_pct: f64,
    color: Color,
}

fn draw_header(f: &mut Frame, rect: Rect, bg: Color, b: &Block<'_>, remaining: i64) {
    let pct_col = quota_color(b.latest_pct, remaining, b.window);
    let pace = pace_balance_secs(b.latest_pct, remaining, b.window);
    let mut spans: Vec<Span<'static>> = vec![
        Span::styled(b.label.to_string(), Style::default().fg(DIM).bg(bg)),
        Span::styled("  ".to_string(), Style::default().bg(bg)),
        Span::styled(
            format!("{}%", b.latest_pct.round() as i64),
            Style::default().fg(pct_col).bg(bg),
        ),
    ];
    if let Some(p) = pace {
        spans.push(Span::styled(" ".to_string(), Style::default().bg(bg)));
        spans.push(Span::styled(
            fmt_pace(p),
            Style::default().fg(pace_color(p, b.window)).bg(bg),
        ));
    }
    if remaining > 0 {
        spans.push(Span::styled(
            format!("  ↺{}", fmt_reset(remaining)),
            Style::default().fg(DIM).bg(bg),
        ));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
        rect,
    );
}

/// Dedupe concurrent samples: bucket by timestamp, take max pct. Ensures a
/// monotonic single-valued series for a clean line.
fn dedupe_max(points: &[(i64, f64)]) -> Vec<(f64, f64)> {
    let mut by_ts: std::collections::BTreeMap<i64, f64> = std::collections::BTreeMap::new();
    for &(t, p) in points {
        by_ts.entry(t).and_modify(|v| *v = v.max(p)).or_insert(p);
    }
    // Enforce monotonicity: quota within a window only ever goes up. Carry
    // the running max forward so stale lower reports never create dips.
    let mut running = 0.0_f64;
    by_ts
        .into_iter()
        .map(|(t, p)| {
            running = running.max(p);
            (t as f64, running)
        })
        .collect()
}

fn draw_chart(f: &mut Frame, rect: Rect, bg: Color, b: &Block<'_>) {
    if rect.height < 2 {
        return;
    }
    let start = (b.latest_reset - b.window) as f64;
    let end = b.latest_reset as f64;

    let mut data = dedupe_max(b.points);
    // Ground the line at the window opening if the first sample is near it.
    if let Some(&(first_x, _)) = data.first()
        && first_x - start < b.window as f64 * 0.1
    {
        data.insert(0, (start, 0.0));
    }

    let usage_ds = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(b.color))
        .data(&data);

    let x_axis = Axis::default()
        .bounds([start, end])
        .style(Style::default().fg(bg));
    let y_axis = Axis::default()
        .bounds([0.0, 100.0])
        .style(Style::default().fg(bg));

    let chart = Chart::new(vec![usage_ds])
        .x_axis(x_axis)
        .y_axis(y_axis)
        .style(Style::default().bg(bg));

    f.render_widget(chart, rect);
}

fn draw_block(f: &mut Frame, area: Rect, bg: Color, b: Block<'_>) {
    if area.height < 2 {
        return;
    }
    let now = now_ts();
    let remaining = (b.latest_reset - now).max(0);

    draw_header(
        f,
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        },
        bg,
        &b,
        remaining,
    );
    draw_chart(
        f,
        Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height - 1,
        },
        bg,
        &b,
    );
}

pub fn draw(f: &mut Frame, area: Rect, bg: Color) {
    if area.height < 4 || area.width < 6 {
        return;
    }
    let samples = load_samples();
    if samples.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                " no usage data yet",
                Style::default().fg(DIM),
            )))
            .style(Style::default().bg(bg)),
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 1,
            },
        );
        return;
    }

    // Pick the most recent reset timestamps — identifies the current window.
    let fh_reset = samples.iter().map(|s| s.fh_reset).max().unwrap_or(0);
    let sd_reset = samples.iter().map(|s| s.sd_reset).max().unwrap_or(0);

    let fh_points: Vec<(i64, f64)> = samples
        .iter()
        .filter(|s| s.fh_reset == fh_reset && s.ts >= fh_reset - FIVE_HOURS)
        .map(|s| (s.ts, s.fh_pct))
        .collect();
    let sd_points: Vec<(i64, f64)> = samples
        .iter()
        .filter(|s| s.sd_reset == sd_reset && s.ts >= sd_reset - SEVEN_DAYS)
        .map(|s| (s.ts, s.sd_pct))
        .collect();

    // Usage is monotonic within a window; the max is the true current value.
    let fh_pct = fh_points
        .iter()
        .map(|&(_, p)| p)
        .fold(0.0_f64, f64::max);
    let sd_pct = sd_points
        .iter()
        .map(|&(_, p)| p)
        .fold(0.0_f64, f64::max);

    let half = area.height / 2;
    let top = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: half,
    };
    let bot = Rect {
        x: area.x,
        y: area.y + half,
        width: area.width,
        height: area.height - half,
    };

    draw_block(
        f,
        top,
        bg,
        Block {
            label: "5h",
            window: FIVE_HOURS,
            points: &fh_points,
            latest_reset: fh_reset,
            latest_pct: fh_pct,
            color: GREEN,
        },
    );
    draw_block(
        f,
        bot,
        bg,
        Block {
            label: "7d",
            window: SEVEN_DAYS,
            points: &sd_points,
            latest_reset: sd_reset,
            latest_pct: sd_pct,
            color: BLUE,
        },
    );
}
