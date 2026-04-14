use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::palette::{BASE, GREEN, OVERLAY0, PEACH, SUBTEXT0, SURFACE0, SURFACE1, TEXT};

use super::super::meta::{agent_color, agent_glyph};
use super::super::overlay::RenameOverlay;
use super::super::tree::{Item, ItemKind, Tree};
use super::super::truncate;
use crate::palette::{
    CTX_EMPTY_COLOR, CTX_POS_COLORS, age_color, ctx_label_color, dim_color, format_age, seg_number,
};

const TREE_COLOR: Color = Color::Rgb(0x2e, 0x2f, 0x40);

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
        ItemKind::Agent { name, age } => {
            let color = if is_cur { agent_color(name) } else { SURFACE1 };
            let age_str = age.map(format_age).unwrap_or_default();
            let age_width = if age_str.is_empty() {
                0
            } else {
                age_str.chars().count() + 1 // " " separator
            };
            let name_w = content_w.saturating_sub(age_width);
            let disp = match agent_glyph(name) {
                Some(g) => g.to_string(),
                None => truncate(name, name_w).to_string(),
            };
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

            for (i, &pos_color) in CTX_POS_COLORS.iter().enumerate().take(BAR_WIDTH) {
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
                        pos_color
                    } else {
                        dim_color(pos_color)
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
