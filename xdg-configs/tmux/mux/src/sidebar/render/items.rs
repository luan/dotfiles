use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::palette::{BASE, OVERLAY0, PEACH, SUBTEXT0, SURFACE0, SURFACE1, TEXT};

use super::super::meta::{agent_color, agent_glyph};
use super::super::overlay::RenameOverlay;
use super::super::tree::{Item, ItemKind, Tree};
use super::super::truncate;
use crate::palette::{age_color, ctx_label_color, dim_color, format_age};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TREE_COLOR: Color = Color::Rgb(0x2e, 0x2f, 0x40);

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
/// Fast pulse for "asking" attention state.
const ASK_PULSE_MS: u128 = 800;
const ASK_COLOR: Color = Color::Rgb(0xF9, 0xE2, 0xAF); // yellow — attention

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

/// Linear blend of two RGB colors. `t` = 0 → a, `t` = 1 → b.
fn blend_color(a: Color, b: Color, t: f32) -> Color {
    let (ar, ag, ab) = match a {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0xff, 0xff, 0xff),
    };
    let (br, bg, bb) = match b {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0xff, 0xff, 0xff),
    };
    let mix = |x: u8, y: u8| ((x as f32) * (1.0 - t) + (y as f32) * t).round() as u8;
    Color::Rgb(mix(ar, br), mix(ag, bg), mix(ab, bb))
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

            if *asking {
                // ── Asking: loud attention pulse ──
                let pulse = triangle_wave(now_ms, ASK_PULSE_MS, 0.0, 1.0);
                let glyph_fg = blend_color(agent_col, ASK_COLOR, pulse);
                line.push(Span::styled(
                    glyph_str,
                    Style::default().fg(glyph_fg).bg(row_bg),
                ));
                line.push(Span::styled(
                    " Waiting…",
                    Style::default().fg(ASK_COLOR).bg(row_bg),
                ));
            } else if let Some(gerund_str) = gerund.as_deref() {
                // ── Active: glyph brightness pulse + percolating gerund ──
                let perc_step = (now_ms / PERC_MS) as usize;
                let brightness = triangle_wave(now_ms, GLYPH_PULSE_MS, PULSE_MIN, PULSE_MAX);
                line.push(Span::styled(
                    glyph_str,
                    Style::default()
                        .fg(scale_brightness(agent_col, brightness))
                        .bg(row_bg),
                ));
                line.push(Span::styled(" ", Style::default().bg(row_bg)));

                let (base, shine) = if name == "claude" {
                    (PERC_BASE, PERC_SHINE)
                } else {
                    (
                        scale_brightness(agent_col, 0.6),
                        scale_brightness(agent_col, 1.5),
                    )
                };

                let word: &str = match name.as_str() {
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
