use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::palette::{BASE, OVERLAY0, PEACH, SUBTEXT0, SURFACE0, SURFACE1, TEXT};

use super::super::meta::{
    PullRequestCheckStatus, PullRequestCiState, PullRequestMeta, PullRequestReviewState,
};
use super::super::meta::{agent_color, agent_glyph};
use super::super::overlay::RenameOverlay;
use super::super::tree::{Item, ItemKind, Tree};
use super::super::truncate;
use crate::palette::{age_color, ctx_label_color, dim_color, format_age};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TREE_COLOR: Color = Color::Rgb(0x2e, 0x2f, 0x40);
const WAITING_COLOR: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const PR_DRAFT_PASSING: Color = Color::Rgb(0x7f, 0x84, 0x9c);
const PR_DRAFT_FAILING: Color = Color::Rgb(0xb8, 0xad, 0x7d);
const PR_REVIEW_PASSING: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const PR_REVIEW_FAILING: Color = Color::Rgb(0xfa, 0xb3, 0x87);
const PR_CHANGES_PASSING: Color = Color::Rgb(0xf3, 0x8b, 0xa8);
const PR_CHANGES_FAILING: Color = Color::Rgb(0xf5, 0xc2, 0xe7);
const PR_APPROVED_PASSING: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
const PR_APPROVED_FAILING: Color = Color::Rgb(0xcb, 0xa6, 0xf7);

// ── Agent activity animation ─────────────────────────────────
// Claude's percolation palette (warm amber).
const PERC_BASE: Color = Color::Rgb(0xD7, 0x87, 0x87);
const PERC_SHINE: Color = Color::Rgb(0xFF, 0xAF, 0x87);
/// Width of the travelling shine window (in characters).
const PERC_WIDTH: usize = 3;
/// Milliseconds per percolation step (shine slides one char right).
const PERC_MS: u128 = 80;
/// Milliseconds for one full glyph brightness cycle (dim → bright → dim).
const GLYPH_PULSE_MS: u128 = 2000;
/// Floor/ceiling for glyph brightness pulse — avoids full black or full white.
const PULSE_MIN: f32 = 0.15;
const PULSE_MAX: f32 = 1.3;
const CODEX_VERBS: &[&str] = &["Codexing…", "Working…", "Thingamabobbing…"];
const OPENCODE_VERBS: &[&str] = &["Opencodding…", "Opendoing…", "Shming Shmopenig…"];
const PI_VERBS: &[&str] = &["Purring…", "Noodling…", "Tinkering…", "Scribbling…"];

/// Scale an RGB color's brightness by `factor`. 0.0 = black, 1.0 = original,
/// 1.5 = 50% brighter (clamped to 255).
fn scale_brightness(c: Color, factor: f32) -> Color {
    let (r, g, b) = match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0xff, 0xff, 0xff),
    };
    let s = |v: u8| ((v as f32 * factor).round().clamp(0.0, 255.0)) as u8;
    Color::Rgb(s(r), s(g), s(b))
}

fn format_cpu_pct(cpu_pct: f32) -> String {
    let cpu_pct = cpu_pct.max(0.0);
    if cpu_pct < 9.95 {
        format!("{cpu_pct:.1}%")
    } else {
        format!("{cpu_pct:.0}%")
    }
}

fn format_mem(bytes: u64) -> String {
    let mib = bytes as f64 / 1024.0 / 1024.0;
    if mib >= 1024.0 {
        format!("{:.1}G", mib / 1024.0)
    } else {
        format!("{mib:.0}M")
    }
}

fn process_icon_and_color(name: &str) -> (&'static str, Color) {
    match name {
        "nvim" => ("\u{e6ae}", Color::Rgb(0xa6, 0xe3, 0xa1)),
        "lazygit" => ("\u{e702}", Color::Rgb(0xfa, 0xb3, 0x87)),
        "rustc" | "cargo" => ("\u{e7a8}", Color::Rgb(0xce, 0x41, 0x22)),
        "node" => ("\u{ed0d}", Color::Rgb(0xa6, 0xe3, 0xa1)),
        "ninja" => ("\u{ed0d}", OVERLAY0),
        "ruby" | "bundle" => ("\u{e791}", Color::Rgb(0xf3, 0x8b, 0xa8)),
        _ if name.starts_with("swift") => ("\u{e755}", Color::Rgb(0xfa, 0xb3, 0x87)),
        _ if name.starts_with("python") => ("\u{e73c}", Color::Rgb(0xf9, 0xe2, 0xaf)),
        _ if name.starts_with("go") => ("\u{e724}", Color::Rgb(0x74, 0xc7, 0xec)),
        _ if name.starts_with("java") => ("\u{e738}", Color::Rgb(0xf9, 0xe2, 0xaf)),
        _ => ("\u{e795}", OVERLAY0),
    }
}

const CPU_ICON: &str = "\u{f2db}";
const MEM_ICON: &str = "\u{efc5}";

fn cpu_stat_color(cpu_pct: f32) -> Color {
    if cpu_pct >= 100.0 {
        PEACH
    } else if cpu_pct >= 20.0 {
        Color::Rgb(0xf9, 0xe2, 0xaf)
    } else {
        SURFACE1
    }
}

fn mem_stat_color(mem_bytes: u64) -> Color {
    if mem_bytes >= 2 * 1024 * 1024 * 1024 {
        PEACH
    } else if mem_bytes >= 1024 * 1024 * 1024 {
        Color::Rgb(0xf9, 0xe2, 0xaf)
    } else {
        SURFACE1
    }
}

pub(in crate::sidebar) fn pr_base_color(pr: &PullRequestMeta) -> Color {
    use PullRequestCiState::*;
    use PullRequestReviewState::*;
    let failing = matches!(pr.ci_state, Failing | RunningFailed);
    match (pr.review_state, failing) {
        (Draft, false) => PR_DRAFT_PASSING,
        (Draft, true) => PR_DRAFT_FAILING,
        (InReview, false) => PR_REVIEW_PASSING,
        (InReview, true) => PR_REVIEW_FAILING,
        (ChangesRequested, false) => PR_CHANGES_PASSING,
        (ChangesRequested, true) => PR_CHANGES_FAILING,
        (Approved, false) => PR_APPROVED_PASSING,
        (Approved, true) => PR_APPROVED_FAILING,
    }
}

fn pr_display_color(pr: &PullRequestMeta, now_ms: u128) -> Color {
    let base = pr_base_color(pr);
    if matches!(
        pr.ci_state,
        PullRequestCiState::RunningClean | PullRequestCiState::RunningFailed
    ) {
        scale_brightness(
            base,
            triangle_wave(now_ms, GLYPH_PULSE_MS, PULSE_MIN, PULSE_MAX),
        )
    } else {
        base
    }
}

fn pr_status_glyph(pr: &PullRequestMeta, now_ms: u128) -> &'static str {
    match pr.ci_state {
        PullRequestCiState::Passing => "",
        PullRequestCiState::Failing => "",
        PullRequestCiState::RunningClean | PullRequestCiState::RunningFailed => {
            const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
            SPINNER[(now_ms / 120) as usize % SPINNER.len()]
        }
    }
}

fn pr_number_label(pr: &PullRequestMeta) -> String {
    format!(" #{}", pr.number)
}

fn aggregate_stat_spans(cpu_pct: f32, mem_bytes: u64, row_bg: Color) -> Vec<Span<'static>> {
    let cpu_color = cpu_stat_color(cpu_pct);
    let mem_color = mem_stat_color(mem_bytes);
    vec![
        Span::styled(" ", Style::default().bg(row_bg)),
        Span::styled(CPU_ICON, Style::default().fg(cpu_color).bg(row_bg)),
        Span::styled(" ", Style::default().bg(row_bg)),
        Span::styled(
            format_cpu_pct(cpu_pct),
            Style::default().fg(cpu_color).bg(row_bg),
        ),
        Span::styled(" ", Style::default().bg(row_bg)),
        Span::styled(MEM_ICON, Style::default().fg(mem_color).bg(row_bg)),
        Span::styled(" ", Style::default().bg(row_bg)),
        Span::styled(
            format_mem(mem_bytes),
            Style::default().fg(mem_color).bg(row_bg),
        ),
    ]
}

/// Triangle wave `lo → hi → lo` over `period_ms`.
fn triangle_wave(now_ms: u128, period_ms: u128, lo: f32, hi: f32) -> f32 {
    let t = (now_ms % period_ms) as f32 / period_ms as f32;
    let tri = 1.0 - (2.0 * t - 1.0).abs(); // 0→1→0
    lo + tri * (hi - lo)
}

pub(in crate::sidebar) fn tree_prefix_spans(
    tree: Tree,
    indent: usize,
    row_bg: Color,
) -> Vec<Span<'static>> {
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

fn bar_span<'a>(item: &'a Item, is_sel: bool, row_bg: Color) -> Span<'a> {
    if is_sel {
        Span::styled("▌", Style::default().fg(item.color).bg(row_bg))
    } else {
        Span::styled(" ", Style::default().bg(row_bg))
    }
}

const HOVER_BG: Color = Color::Rgb(0x28, 0x29, 0x3a);

pub(in crate::sidebar) fn render_item(
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
        ItemKind::Session {
            diff,
            cpu_pct,
            mem_bytes,
        } => {
            let fg = if is_sel || is_cur {
                item.color
            } else {
                item.dim_color
            };

            let mut spans: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
            spans.extend(tree_prefix_spans(item.tree, indent, row_bg));

            let mut right: Vec<Span<'static>> = Vec::new();
            if let Some(diff) = diff {
                right.push(Span::styled(
                    format!("+{}", diff.added),
                    Style::default().fg(Color::Rgb(0xa6, 0xe3, 0xa1)).bg(row_bg),
                ));
                right.push(Span::styled(" ", Style::default().bg(row_bg)));
                right.push(Span::styled(
                    format!("-{}", diff.removed),
                    Style::default().fg(Color::Rgb(0xf3, 0x8b, 0xa8)).bg(row_bg),
                ));
            }

            let stats = aggregate_stat_spans(*cpu_pct, *mem_bytes, row_bg);
            let stats_w: usize = stats.iter().map(|s| s.width()).sum();
            let right_w: usize = right.iter().map(|s| s.width()).sum();
            let mut reserved = right_w;
            if is_cur {
                reserved += 2;
            }
            let name_w = content_w.saturating_sub(reserved + stats_w);
            let name = truncate(&item.display, name_w);

            let name_style = if is_cur {
                Style::default().fg(fg).bold().bg(row_bg)
            } else {
                Style::default().fg(fg).bg(row_bg)
            };
            spans.push(Span::styled(name, name_style));
            spans.extend(stats);

            let used: usize = spans.iter().skip(1).map(|s| s.width()).sum();
            let pad = (w - bar_w).saturating_sub(used + reserved);
            if pad > 0 {
                spans.push(Span::styled(" ".repeat(pad), Style::default().bg(row_bg)));
            }

            spans.extend(right);

            if is_cur {
                spans.push(Span::styled("←", Style::default().fg(SUBTEXT0).bg(row_bg)));
                spans.push(Span::styled(" ", Style::default().bg(row_bg)));
            }

            f.render_widget(
                Paragraph::new(Line::from(spans)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Process(process) => {
            let (icon, icon_color) = process_icon_and_color(&process.name);
            let icon_color = if is_cur {
                icon_color
            } else {
                dim_color(icon_color)
            };
            let name_color = if is_cur { SUBTEXT0 } else { SURFACE1 };
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            line.push(Span::styled(
                icon,
                Style::default().fg(icon_color).bg(row_bg),
            ));
            line.push(Span::styled(" ", Style::default().bg(row_bg)));

            let right = vec![
                Span::styled(
                    CPU_ICON,
                    Style::default()
                        .fg(cpu_stat_color(process.cpu_pct))
                        .bg(row_bg),
                ),
                Span::styled(" ", Style::default().bg(row_bg)),
                Span::styled(
                    format_cpu_pct(process.cpu_pct),
                    Style::default()
                        .fg(cpu_stat_color(process.cpu_pct))
                        .bg(row_bg),
                ),
                Span::styled(" ", Style::default().bg(row_bg)),
                Span::styled(
                    MEM_ICON,
                    Style::default()
                        .fg(mem_stat_color(process.mem_bytes))
                        .bg(row_bg),
                ),
                Span::styled(" ", Style::default().bg(row_bg)),
                Span::styled(
                    format_mem(process.mem_bytes),
                    Style::default()
                        .fg(mem_stat_color(process.mem_bytes))
                        .bg(row_bg),
                ),
            ];
            let right_w: usize = right.iter().map(|s| s.width()).sum();
            let left_w: usize = line.iter().map(|s| s.width()).sum();
            let name_w = w.saturating_sub(left_w + right_w + 1);
            line.push(Span::styled(
                truncate(&process.name, name_w),
                Style::default().fg(name_color).bg(row_bg),
            ));
            let used: usize = line.iter().map(|s| s.width()).sum();
            let pad = w.saturating_sub(used + right_w);
            if pad > 0 {
                line.push(Span::styled(" ".repeat(pad), Style::default().bg(row_bg)));
            }
            line.extend(right);
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Branch { pr } => {
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let (pr_label, pr_color) = pr
                .as_ref()
                .map(|pr| (pr_number_label(pr), pr_display_color(pr, now_ms)))
                .unwrap_or_default();
            let pr_color = if is_cur {
                pr_color
            } else {
                dim_color(pr_color)
            };
            let right_w = if pr_label.is_empty() {
                0
            } else {
                pr_label.chars().count() + 1
            };
            let status_glyph = pr.as_ref().map(|pr| pr_status_glyph(pr, now_ms));
            let status_w = status_glyph.map(|s| s.chars().count() + 1).unwrap_or(0);
            if let Some(status_glyph) = status_glyph {
                line.push(Span::styled(
                    status_glyph,
                    Style::default().fg(pr_color).bg(row_bg),
                ));
                line.push(Span::styled(" ", Style::default().bg(row_bg)));
            }
            let disp = truncate(&item.display, content_w.saturating_sub(right_w + status_w));
            line.push(Span::styled(
                disp,
                Style::default()
                    .fg(pr.as_ref().map(|_| pr_color).unwrap_or(SURFACE1))
                    .italic()
                    .bg(row_bg),
            ));
            if !pr_label.is_empty() {
                let used: usize = line.iter().skip(1).map(|s| s.width()).sum();
                let pad = content_w.saturating_sub(used + pr_label.chars().count());
                if pad > 0 {
                    line.push(Span::styled(" ".repeat(pad), Style::default().bg(row_bg)));
                }
                line.push(Span::styled(
                    pr_label,
                    Style::default().fg(pr_color).underlined().bold().bg(row_bg),
                ));
            }
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::PullRequestCheck { check, pr } => {
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
            let (glyph, color) = match check.status {
                PullRequestCheckStatus::Running => (
                    spinner[(now_ms / 120) as usize % spinner.len()],
                    scale_brightness(
                        pr_base_color(pr),
                        triangle_wave(now_ms, GLYPH_PULSE_MS, 0.7, 1.35),
                    ),
                ),
                PullRequestCheckStatus::Failing => ("", pr_base_color(pr)),
            };
            let age_str = format_age(check.elapsed);
            line.push(Span::styled(glyph, Style::default().fg(color).bg(row_bg)));
            line.push(Span::styled(" ", Style::default().bg(row_bg)));
            let right_w = age_str.chars().count() + 1;
            let left_w: usize = line.iter().skip(1).map(|s| s.width()).sum();
            let name_w = content_w.saturating_sub(left_w + right_w);
            line.push(Span::styled(
                truncate(&check.name, name_w),
                Style::default().fg(color).bg(row_bg),
            ));
            let used: usize = line.iter().skip(1).map(|s| s.width()).sum();
            let pad = content_w.saturating_sub(used + age_str.chars().count());
            if pad > 0 {
                line.push(Span::styled(" ".repeat(pad), Style::default().bg(row_bg)));
            }
            line.push(Span::styled(
                age_str,
                Style::default().fg(SURFACE1).bg(row_bg),
            ));
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::PullRequestUnresolved { count, pr } => {
            let color = if is_cur {
                pr_base_color(pr)
            } else {
                dim_color(pr_base_color(pr))
            };
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));
            line.push(Span::styled("", Style::default().fg(color).bg(row_bg)));
            line.push(Span::styled("  ", Style::default().bg(row_bg)));
            line.push(Span::styled(
                format!("{count} unresolved"),
                Style::default().fg(color).bg(row_bg),
            ));
            f.render_widget(
                Paragraph::new(Line::from(line)).style(Style::default().bg(row_bg)),
                row,
            );
        }
        ItemKind::Agent {
            name,
            age,
            gerund,
            ctx,
            asking,
        } => {
            let color = if is_cur {
                agent_color(name)
            } else {
                dim_color(agent_color(name))
            };
            let age_str = age.map(format_age).unwrap_or_default();
            let mut line: Vec<Span<'_>> = vec![bar_span(item, is_sel, row_bg)];
            line.extend(tree_prefix_spans(item.tree, indent, row_bg));

            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();

            let glyph_str = agent_glyph(name).unwrap_or(name).to_string();
            let agent_col = agent_color(name);

            if *asking || gerund.is_some() {
                // ── Active: glyph brightness pulse + percolating gerund ──
                let perc_step = (now_ms / PERC_MS) as usize;
                let brightness = triangle_wave(now_ms, GLYPH_PULSE_MS, PULSE_MIN, PULSE_MAX);
                let active_col = if *asking { WAITING_COLOR } else { agent_col };
                line.push(Span::styled(
                    glyph_str,
                    Style::default()
                        .fg(scale_brightness(active_col, brightness))
                        .bg(row_bg),
                ));
                line.push(Span::styled(" ", Style::default().bg(row_bg)));

                let (base, shine) = if *asking {
                    (
                        scale_brightness(WAITING_COLOR, 0.7),
                        scale_brightness(WAITING_COLOR, 1.25),
                    )
                } else if name == "claude" {
                    (PERC_BASE, PERC_SHINE)
                } else {
                    (
                        scale_brightness(agent_col, 0.6),
                        scale_brightness(agent_col, 1.5),
                    )
                };

                let word: &str = if *asking {
                    "Waiting…"
                } else {
                    let gerund_str = gerund.as_deref().unwrap_or_default();
                    match name.as_str() {
                        "codex" => {
                            let idx = (now_ms / 8000) as usize % CODEX_VERBS.len();
                            CODEX_VERBS[idx]
                        }
                        "opencode" => {
                            let idx = (now_ms / 8000) as usize % OPENCODE_VERBS.len();
                            OPENCODE_VERBS[idx]
                        }
                        "pi" => {
                            let idx = (now_ms / 8000) as usize % PI_VERBS.len();
                            PI_VERBS[idx]
                        }
                        _ => gerund_str,
                    }
                };

                let chars: Vec<char> = word.chars().collect();
                let cycle = chars.len() + PERC_WIDTH;
                let pos = perc_step % cycle;
                for (i, ch) in chars.iter().enumerate() {
                    let in_shine = i >= pos.saturating_sub(PERC_WIDTH) && i < pos;
                    let fg = if in_shine { shine } else { base };
                    line.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(fg).bg(row_bg),
                    ));
                }
            } else {
                // ── Idle ──
                line.push(Span::styled(
                    glyph_str,
                    Style::default().fg(color).bg(row_bg),
                ));
                let show_age = age.is_some_and(|d| d >= Duration::from_secs(300));
                if show_age {
                    line.push(Span::styled(
                        " Idle for ",
                        Style::default().fg(SURFACE1).bg(row_bg),
                    ));
                    let a_color = if is_cur {
                        age.map(age_color).unwrap_or(SURFACE1)
                    } else {
                        age.map(|d| dim_color(age_color(d))).unwrap_or(SURFACE1)
                    };
                    line.push(Span::styled(
                        format!("{age_str}."),
                        Style::default().fg(a_color).bg(row_bg),
                    ));
                } else {
                    line.push(Span::styled(
                        " Idle.",
                        Style::default().fg(SURFACE1).bg(row_bg),
                    ));
                }
            }

            // Right-aligned section: [ctx] [age]
            // Compute right-side width, insert padding, then render right spans.
            let mut right: Vec<Span<'static>> = Vec::new();
            if let Some((pct, tokens)) = ctx {
                // pct=0 means "no usage data" (e.g. codex) — show tokens only.
                if *pct > 0 {
                    let label_color = if is_cur {
                        ctx_label_color(*pct)
                    } else {
                        dim_color(ctx_label_color(*pct))
                    };
                    right.push(Span::styled(
                        format!("{pct}\u{066A}"),
                        Style::default().fg(label_color).bg(row_bg),
                    ));
                    if !tokens.is_empty() {
                        right.push(Span::styled(" ", Style::default().bg(row_bg)));
                    }
                }
                if !tokens.is_empty() {
                    let tok_color = if is_cur { OVERLAY0 } else { SURFACE1 };
                    right.push(Span::styled(
                        tokens.clone(),
                        Style::default().fg(tok_color).bg(row_bg),
                    ));
                }
            }
            if !right.is_empty() {
                let right_w: usize = right.iter().map(|s| s.width()).sum();
                let left_w: usize = line.iter().map(|s| s.width()).sum();
                let pad = w.saturating_sub(left_w + right_w + 1);
                line.push(Span::styled(
                    " ".repeat(pad.max(1)),
                    Style::default().bg(row_bg),
                ));
                line.extend(right);
                line.push(Span::styled(" ", Style::default().bg(row_bg)));
            }
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

pub(in crate::sidebar) struct InlineRenameCtx<'a> {
    pub(in crate::sidebar) item: &'a Item,
    pub(in crate::sidebar) rename: &'a mut RenameOverlay,
    pub(in crate::sidebar) is_hover: bool,
    pub(in crate::sidebar) is_cur: bool,
    pub(in crate::sidebar) focused: bool,
}

pub(in crate::sidebar) fn render_inline_rename_item(
    f: &mut Frame,
    row: Rect,
    ctx: &mut InlineRenameCtx<'_>,
) {
    let item = ctx.item;
    let rename = &mut *ctx.rename;
    let is_hover = ctx.is_hover;
    let is_cur = ctx.is_cur;
    let focused = ctx.focused;
    let w = row.width as usize;
    if w == 0 {
        return;
    }

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
