use std::cmp::min;

use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::palette::{BLUE, OVERLAY0, PEACH, SUBTEXT0, SURFACE0, SURFACE1, TEXT};
use crate::picker::PickerItem;

use super::super::overlay::SidebarOverlay;
use super::super::truncate;
use super::render_at;

const TREE_COLOR: Color = Color::Rgb(0x2e, 0x2f, 0x40);

pub(in crate::sidebar) fn overlay_height(overlay: &SidebarOverlay, max_list_h: u16) -> u16 {
    let desired = match overlay {
        SidebarOverlay::Rename(rename) => 1 + u16::from(rename.error.is_some()),
        SidebarOverlay::SessionName(session) => 1 + u16::from(session.error.is_some()),
        SidebarOverlay::Project(project) => 1 + min(project.items.len(), 4) as u16,
        SidebarOverlay::Worktree(worktree) => {
            u16::from(worktree.error.is_some()) + worktree.items.len() as u16
        }
        SidebarOverlay::Ditch(list) => {
            u16::from(list.error.is_some()) + min(list.items.len(), 4) as u16
        }
    };
    desired.clamp(1, max_list_h.max(1))
}

pub(in crate::sidebar) fn render_overlay(
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
            &TextOverlayCtx {
                title: &session.title,
                prefix: &session.prefix,
                input: &session.input,
                cursor: session.cursor,
                error: session.error.as_deref(),
                focused,
                bg,
            },
        ),
        SidebarOverlay::Project(project) => render_picker_overlay(
            f,
            inner,
            &mut PickerOverlayCtx {
                filter: Some((&project.filter, project.cursor)),
                selected: &mut project.selected,
                offset: &mut project.offset,
                items: &project.items,
                error: None,
                focused,
                bg,
            },
        ),
        SidebarOverlay::Worktree(worktree) => render_picker_overlay(
            f,
            inner,
            &mut PickerOverlayCtx {
                filter: None,
                selected: &mut worktree.selected,
                offset: &mut worktree.offset,
                items: &worktree.items,
                error: worktree.error.as_deref(),
                focused,
                bg,
            },
        ),
        SidebarOverlay::Ditch(list) => render_picker_overlay(
            f,
            inner,
            &mut PickerOverlayCtx {
                filter: None,
                selected: &mut list.selected,
                offset: &mut list.offset,
                items: &list.items,
                error: list.error.as_deref(),
                focused,
                bg,
            },
        ),
    }
}

struct TextOverlayCtx<'a> {
    title: &'a str,
    prefix: &'a str,
    input: &'a str,
    cursor: usize,
    error: Option<&'a str>,
    focused: bool,
    bg: Color,
}

fn render_text_overlay(f: &mut Frame, area: Rect, ctx: &TextOverlayCtx<'_>) {
    let TextOverlayCtx {
        title,
        prefix,
        input,
        cursor,
        error,
        focused,
        bg,
    } = *ctx;
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(if error.is_some() { 1 } else { 0 }),
    ])
    .split(area);

    let input_bg = bg;
    let placeholder = if prefix.is_empty() {
        format!("{}...", title.to_lowercase())
    } else {
        "session name...".to_string()
    };
    let content = if input.is_empty() {
        Line::from(vec![
            Span::styled(
                prefix.to_string(),
                Style::default().fg(SUBTEXT0).bg(input_bg),
            ),
            Span::styled(
                placeholder,
                Style::default().fg(OVERLAY0).italic().bg(input_bg),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                prefix.to_string(),
                Style::default().fg(SUBTEXT0).bg(input_bg),
            ),
            Span::styled(input.to_string(), Style::default().fg(TEXT).bg(input_bg)),
        ])
    };
    let line = Line::from(
        std::iter::once(Span::styled("▌", Style::default().fg(BLUE).bg(input_bg)))
            .chain(std::iter::once(Span::styled(
                " ",
                Style::default().bg(input_bg),
            )))
            .chain(content.spans)
            .collect::<Vec<_>>(),
    );
    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(input_bg)),
        chunks[0],
    );

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

struct PickerOverlayCtx<'a> {
    filter: Option<(&'a str, usize)>,
    selected: &'a mut usize,
    offset: &'a mut usize,
    items: &'a [PickerItem],
    error: Option<&'a str>,
    focused: bool,
    bg: Color,
}

fn render_picker_overlay(f: &mut Frame, area: Rect, ctx: &mut PickerOverlayCtx<'_>) {
    let overlay_bg = ctx.bg;
    let chunks = Layout::vertical([
        Constraint::Length(if ctx.filter.is_some() { 1 } else { 0 }),
        Constraint::Length(if ctx.error.is_some() { 1 } else { 0 }),
        Constraint::Min(1),
    ])
    .split(area);
    if let Some((query, cursor)) = ctx.filter {
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
        f.render_widget(
            Paragraph::new(line).style(Style::default().bg(input_bg)),
            chunks[0],
        );
        if ctx.focused {
            let max_x = chunks[0].x + chunks[0].width.saturating_sub(1);
            let cursor_x = (chunks[0].x + 2 + cursor as u16).min(max_x);
            f.set_cursor_position((cursor_x, chunks[0].y));
        }
    }

    if let Some(error) = ctx.error {
        let row = if ctx.filter.is_some() {
            chunks[1]
        } else {
            chunks[0]
        };
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

    let selected = &mut *ctx.selected;
    let offset = &mut *ctx.offset;
    let items = ctx.items;
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

        let row_bg = if idx == *selected {
            SURFACE0
        } else {
            overlay_bg
        };
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
        let main_width = text_width
            .saturating_sub(label_width.saturating_add(if label_width > 0 { 1 } else { 0 }));
        spans.push(Span::styled(
            truncate(&item.display, main_width),
            text_style,
        ));
        if label_width > 0 {
            let used = spans
                .iter()
                .map(|span| span.width())
                .sum::<usize>()
                .saturating_sub(2);
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
