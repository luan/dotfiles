use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::palette::{MANTLE, SUBTEXT0, SURFACE1, TEXT};
use crate::usage_bars;

use super::super::SidebarState;
use super::super::overlay::SidebarOverlay;
use super::hints::{
    chooser_footer_hints, footer_hints, overlay_footer_hints, unfocused_footer_hints,
};
use super::items::{InlineRenameCtx, render_inline_rename_item, render_item};
use super::overlays::{overlay_height, render_overlay};
use super::render_at;

/// Returns (list_y_start, list_height) for mouse mapping.
pub(in crate::sidebar) fn draw(f: &mut Frame, state: &mut SidebarState) -> (u16, u16) {
    let area = f.area();
    f.render_widget(ratatui::widgets::Clear, area);

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

    // Compute footer hints up front. Reserve a fixed 3-row footer across all
    // modes so the list above doesn't reflow when focus toggles or an overlay
    // opens; shorter hint sets bottom-align with empty padding rows on top.
    const FOOTER_H: u16 = 3;
    let mut hint_lines: Vec<Line<'static>> = if state.overlay_active() {
        overlay_footer_hints(content_w as usize)
    } else if state.chooser_active() {
        chooser_footer_hints(content_w as usize, state.show_hidden)
    } else if !state.focused {
        unfocused_footer_hints(content_w as usize)
    } else {
        footer_hints(content_w as usize, state.show_hidden)
    };
    while hint_lines.len() < FOOTER_H as usize {
        hint_lines.insert(0, Line::default());
    }
    let footer_h = FOOTER_H;

    let bars = &state.usage_bars_cache;
    let has_overage_footer = state
        .overage
        .as_ref()
        .map(|ov| ov.month > 0.0 || ov.total > 0.0)
        .unwrap_or(false)
        && bars.iter().any(|b| b.label.starts_with("claude"));
    let wanted_bars_h = usage_bars::height(bars.len(), has_overage_footer);
    // Only show bars when the list still has room for at least 4 rows after.
    // Add 2 extra rows for dim separator lines above and below the bars.
    let bars_h =
        if wanted_bars_h > 0 && area.height >= 4 + 1 + notch_h + wanted_bars_h + 2 + footer_h {
            wanted_bars_h
        } else {
            0
        };
    let sep_h = if bars_h > 0 { 2u16 } else { 0 };
    let list_h = area
        .height
        .saturating_sub(1 + notch_h + footer_h + bars_h + sep_h);

    // Render footer hints
    for (i, line) in hint_lines.into_iter().enumerate() {
        let y = area.y + area.height - footer_h + i as u16;
        render_at(f, area.x, y, content_w, line, bg);
    }

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
        usage_bars::draw(
            f,
            bars_rect,
            bg,
            bars,
            &state.pulse_starts,
            state.overage.as_ref(),
        );
        state.last_bars_y = bars_y;
        state.last_bars_h = bars_h;
    } else {
        state.last_bars_y = 0;
        state.last_bars_h = 0;
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
                    &mut InlineRenameCtx {
                        item,
                        rename,
                        is_hover,
                        is_cur: belongs_to_current,
                        focused: state.focused,
                    },
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

    if let Some(overlay) = state.overlay.as_mut()
        && !matches!(overlay, SidebarOverlay::Rename(_))
    {
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

    (list_y, list_h)
}

fn render_filter_row(f: &mut Frame, row: Rect, state: &SidebarState, bg: Color) {
    if row.width == 0 {
        return;
    }

    let prefix = "/ ";
    let line = if state.filter.is_empty() {
        Line::from(vec![
            Span::styled(prefix, Style::default().fg(SUBTEXT0).bg(bg)),
            Span::styled(
                "search...",
                Style::default()
                    .fg(crate::palette::OVERLAY0)
                    .italic()
                    .bg(bg),
            ),
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
        let cursor_x =
            (row.x + prefix.chars().count() as u16 + state.filter_cursor as u16).min(max_x);
        f.set_cursor_position((cursor_x, row.y));
    }
}
