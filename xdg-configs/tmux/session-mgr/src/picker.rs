use std::collections::{HashMap, HashSet};
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use nucleo_matcher::pattern::{Atom, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

// Catppuccin Mocha — matches sidebar bg
const BASE: Color = Color::Rgb(0x11, 0x11, 0x1b);
const SURFACE0: Color = Color::Rgb(0x31, 0x32, 0x44);
const SURFACE1: Color = Color::Rgb(0x45, 0x47, 0x5a);
const OVERLAY0: Color = Color::Rgb(0x6c, 0x70, 0x86);
const OVERLAY1: Color = Color::Rgb(0x7f, 0x84, 0x9c);
const TEXT: Color = Color::Rgb(0xcd, 0xd6, 0xf4);
const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const BLUE: Color = Color::Rgb(0x89, 0xb4, 0xfa);

fn picker_bg() -> Color {
    std::env::var("TMUX_SESSION_BG")
        .ok()
        .and_then(|value| {
            let hex = value.trim().trim_start_matches('#');
            if hex.len() != 6 {
                return None;
            }
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        })
        .unwrap_or(BASE)
}

#[derive(Clone)]
pub struct PickerItem {
    pub id: String,
    pub display: String,
    pub style: Style,
    pub selectable: bool,
    pub color: Option<Color>,
    pub dim_color: Option<Color>,
    pub right_label: String,
}

#[derive(Clone)]
pub struct PickerConfig {
    pub prompt: String,
    pub footer: String,
    pub placeholder: String,
    pub initial_id: Option<String>,
}

pub enum PickerAction {
    Selected(String),
    Custom(String, String),
    Cancelled,
}

struct PickerState {
    input: String,
    cursor: usize,
    selected: usize,
    offset: usize,
    filtered: Vec<FilteredItem>,
}

struct FilteredItem {
    idx: usize,
    score: u16,
    indices: Vec<u32>,
}

pub fn run_picker(
    items: Vec<PickerItem>,
    config: PickerConfig,
    custom_keys: HashMap<(KeyCode, KeyModifiers), String>,
) -> PickerAction {
    let mut state = PickerState {
        input: String::new(),
        cursor: 0,
        selected: 0,
        offset: 0,
        filtered: Vec::new(),
    };
    refilter(&items, &mut state);
    // Restore cursor to initial_id if provided, otherwise first selectable
    if let Some(ref id) = config.initial_id {
        let found = state.filtered.iter().position(|fi| items[fi.idx].id == *id);
        if let Some(pos) = found {
            state.selected = pos;
        } else {
            snap_to_first_selectable(&items, &mut state);
        }
    } else {
        snap_to_first_selectable(&items, &mut state);
    }

    terminal::enable_raw_mode().expect("enable raw mode");
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).expect("enter alt screen");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let result = run_loop(&mut terminal, &items, &config, &custom_keys, &mut state);

    terminal::disable_raw_mode().expect("disable raw mode");
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen).expect("leave alt screen");

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    items: &[PickerItem],
    config: &PickerConfig,
    custom_keys: &HashMap<(KeyCode, KeyModifiers), String>,
    state: &mut PickerState,
) -> PickerAction {
    loop {
        terminal.draw(|f| draw(f, items, config, state)).ok();

        let Ok(ev) = event::read() else {
            continue;
        };
        let Event::Key(key) = ev else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        // Check custom keys first
        let lookup_key = normalize_key(key);
        if let Some(action_name) = custom_keys.get(&lookup_key) {
            let id = selected_id(items, state);
            return PickerAction::Custom(action_name.clone(), id);
        }

        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                return PickerAction::Cancelled;
            }
            (KeyCode::Enter, _) => {
                let id = selected_id(items, state);
                if !id.is_empty() {
                    return PickerAction::Selected(id);
                }
            }
            (KeyCode::Up, _)
            | (KeyCode::Char('k'), KeyModifiers::CONTROL)
            | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                move_selection(items, state, -1);
            }
            (KeyCode::Down, _)
            | (KeyCode::Char('j'), KeyModifiers::CONTROL)
            | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                move_selection(items, state, 1);
            }
            (KeyCode::Char(c), m) if !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                state.input.insert(state.cursor, c);
                state.cursor += 1;
                refilter(items, state);
                snap_to_first_selectable(items, state);
            }
            (KeyCode::Backspace, _) => {
                if state.cursor > 0 {
                    state.cursor -= 1;
                    state.input.remove(state.cursor);
                    refilter(items, state);
                    snap_to_first_selectable(items, state);
                }
            }
            (KeyCode::Delete, _) => {
                if state.cursor < state.input.len() {
                    state.input.remove(state.cursor);
                    refilter(items, state);
                    snap_to_first_selectable(items, state);
                }
            }
            (KeyCode::Left, _) => {
                state.cursor = state.cursor.saturating_sub(1);
            }
            (KeyCode::Right, _) => {
                if state.cursor < state.input.len() {
                    state.cursor += 1;
                }
            }
            (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                state.cursor = 0;
            }
            (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                state.cursor = state.input.len();
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                state.input.clear();
                state.cursor = 0;
                refilter(items, state);
                snap_to_first_selectable(items, state);
            }
            _ => {}
        }
    }
}

fn normalize_key(key: KeyEvent) -> (KeyCode, KeyModifiers) {
    // Strip SHIFT from letter keys so Alt+Shift+H matches Alt+H
    let mods = match key.code {
        KeyCode::Char(_) => key.modifiers - KeyModifiers::SHIFT,
        _ => key.modifiers,
    };
    // Normalize letter to lowercase
    let code = match key.code {
        KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
        other => other,
    };
    (code, mods)
}

fn selected_id(items: &[PickerItem], state: &PickerState) -> String {
    state
        .filtered
        .get(state.selected)
        .and_then(|fi| {
            let item = &items[fi.idx];
            if item.selectable {
                Some(item.id.clone())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn refilter(items: &[PickerItem], state: &mut PickerState) {
    state.filtered.clear();
    if state.input.is_empty() {
        for (i, _) in items.iter().enumerate() {
            state.filtered.push(FilteredItem {
                idx: i,
                score: 0,
                indices: Vec::new(),
            });
        }
    } else {
        let mut matcher = Matcher::new(Config::DEFAULT);
        let atom = Atom::new(
            &state.input,
            CaseMatching::Ignore,
            Normalization::Smart,
            nucleo_matcher::pattern::AtomKind::Fuzzy,
            false,
        );
        let mut buf = Vec::new();
        let needle = atom.needle_text();
        for (i, item) in items.iter().enumerate() {
            if !item.selectable {
                continue;
            }
            let haystack = Utf32Str::new(&item.display, &mut buf);
            let mut indices = Vec::new();
            if let Some(score) = matcher.fuzzy_indices(haystack, needle, &mut indices) {
                state.filtered.push(FilteredItem {
                    idx: i,
                    score,
                    indices,
                });
            }
            buf.clear();
        }
        state
            .filtered
            .sort_by(|a, b| b.score.cmp(&a.score).then(a.idx.cmp(&b.idx)));

        // Re-insert headers: a header is shown if any item directly after it (before next header) is in the filtered set
        let filtered_indices: HashSet<usize> = state.filtered.iter().map(|f| f.idx).collect();
        let mut headers_to_insert: Vec<(usize, usize)> = Vec::new();
        for fi_pos in 0..state.filtered.len() {
            let item_idx = state.filtered[fi_pos].idx;
            if item_idx > 0 {
                let prev_idx = item_idx - 1;
                if !items[prev_idx].selectable
                    && !filtered_indices.contains(&prev_idx)
                    && !headers_to_insert.iter().any(|(_, hi)| *hi == prev_idx)
                {
                    headers_to_insert.push((fi_pos, prev_idx));
                }
            }
        }
        for (pos, header_idx) in headers_to_insert.into_iter().rev() {
            state.filtered.insert(
                pos,
                FilteredItem {
                    idx: header_idx,
                    score: 0,
                    indices: Vec::new(),
                },
            );
        }
    }
    state.selected = 0;
    state.offset = 0;
}

fn snap_to_first_selectable(items: &[PickerItem], state: &mut PickerState) {
    if state.filtered.is_empty() {
        return;
    }
    if state
        .filtered
        .get(state.selected)
        .is_some_and(|fi| items[fi.idx].selectable)
    {
        return;
    }
    for (i, fi) in state.filtered.iter().enumerate() {
        if items[fi.idx].selectable {
            state.selected = i;
            return;
        }
    }
}

fn move_selection(items: &[PickerItem], state: &mut PickerState, direction: i32) {
    if state.filtered.is_empty() {
        return;
    }
    let len = state.filtered.len();
    let mut pos = state.selected;
    loop {
        if direction > 0 {
            if pos >= len - 1 {
                break;
            }
            pos += 1;
        } else {
            if pos == 0 {
                break;
            }
            pos -= 1;
        }
        if items[state.filtered[pos].idx].selectable {
            state.selected = pos;
            break;
        }
    }
}

fn draw(f: &mut Frame, items: &[PickerItem], config: &PickerConfig, state: &mut PickerState) {
    let area = f.area();
    let base = picker_bg();
    f.render_widget(Clear, area);
    f.render_widget(Block::default().style(Style::default().bg(base)), area);

    // Borderless layout: title, sep, search, sep, list, (sep, footer)
    let has_footer = !config.footer.is_empty();
    let footer_height = if has_footer { 2 } else { 0 };

    let padded = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(1),
    };

    let chunks = Layout::vertical([
        Constraint::Length(1),             // title
        Constraint::Length(1),             // separator
        Constraint::Length(1),             // search input
        Constraint::Length(1),             // separator
        Constraint::Min(1),                // list
        Constraint::Length(footer_height), // footer area
    ])
    .split(padded);

    // Title
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            config.prompt.trim().to_string(),
            Style::default().fg(YELLOW).bold(),
        )))
        .style(Style::default().bg(base)),
        chunks[0],
    );
    // Separator below title
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(chunks[1].width as usize),
            Style::default().fg(SURFACE1),
        )))
        .style(Style::default().bg(base)),
        chunks[1],
    );

    // Search input
    let search_area = chunks[2];
    if state.input.is_empty() {
        let placeholder = Line::from(Span::styled(
            &config.placeholder,
            Style::default().fg(OVERLAY0).italic(),
        ));
        f.render_widget(
            Paragraph::new(placeholder).style(Style::default().bg(base)),
            search_area,
        );
    } else {
        let input_line = Line::from(Span::styled(&state.input, Style::default().fg(TEXT)));
        f.render_widget(
            Paragraph::new(input_line).style(Style::default().bg(base)),
            search_area,
        );
    }

    // Cursor
    let cursor_x = search_area.x + state.cursor as u16;
    f.set_cursor_position((cursor_x, search_area.y));

    // Separator between search and list
    let sep_width = chunks[3].width as usize;
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(sep_width),
        Style::default().fg(SURFACE1),
    )))
    .style(Style::default().bg(base));
    f.render_widget(sep, chunks[3]);

    // List area — reserve 1 col on right for scrollbar
    let list_area = chunks[4];
    let content_width = list_area.width.saturating_sub(1); // leave room for scrollbar
    let visible_height = list_area.height as usize;
    let total_items = state.filtered.len();

    // Adjust offset for scrolling
    if state.selected < state.offset {
        state.offset = state.selected;
    }
    if state.selected >= state.offset + visible_height {
        state.offset = state.selected - visible_height + 1;
    }

    // Build visible lines
    let visible: Vec<(usize, &FilteredItem)> = state
        .filtered
        .iter()
        .enumerate()
        .skip(state.offset)
        .take(visible_height)
        .collect();

    for (abs_idx, fi) in &visible {
        let item = &items[fi.idx];
        let is_selected = *abs_idx == state.selected;
        let row_y = list_area.y + (*abs_idx - state.offset) as u16;
        let row_area = Rect {
            x: list_area.x,
            y: row_y,
            width: content_width,
            height: 1,
        };

        if !item.selectable {
            // Group header: dim, no pointer, subtle indent
            let header_line = Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&item.display, Style::default().fg(OVERLAY0)),
            ]);
            let bg = Style::default().bg(base);
            f.render_widget(Paragraph::new(header_line).style(bg), row_area);
        } else {
            // Selectable item
            let bg_color = if is_selected { SURFACE0 } else { base };
            let accent = item.color.unwrap_or(BLUE);

            let mut spans = Vec::new();

            // Left bar for selected item
            if is_selected {
                spans.push(Span::styled("▌ ", Style::default().fg(accent).bg(bg_color)));
            } else {
                spans.push(Span::styled("  ", Style::default().bg(bg_color)));
            }

            // Build display text with match highlighting
            let dim = item.dim_color.unwrap_or(OVERLAY1);
            let text_style = if is_selected {
                item.style.fg(accent).bg(bg_color)
            } else {
                Style::default().fg(dim).bg(bg_color)
            };

            if !fi.indices.is_empty() && !state.input.is_empty() {
                let match_set: HashSet<u32> = fi.indices.iter().copied().collect();
                for (ci, ch) in item.display.chars().enumerate() {
                    if match_set.contains(&(ci as u32)) {
                        spans.push(Span::styled(
                            ch.to_string(),
                            text_style.fg(YELLOW).underlined(),
                        ));
                    } else {
                        spans.push(Span::styled(ch.to_string(), text_style));
                    }
                }
            } else {
                spans.push(Span::styled(&item.display, text_style));
            }

            // Right-aligned label
            if !item.right_label.is_empty() {
                let used: usize = spans.iter().map(|s| s.width()).sum();
                let label_width = item.right_label.chars().count();
                let available = content_width as usize;
                if used + label_width + 1 < available {
                    let padding = available - used - label_width;
                    spans.push(Span::styled(
                        " ".repeat(padding),
                        Style::default().bg(bg_color),
                    ));
                    spans.push(Span::styled(
                        &item.right_label,
                        Style::default().fg(OVERLAY0).bg(bg_color),
                    ));
                }
            }

            let line = Line::from(spans);
            f.render_widget(
                Paragraph::new(line).style(Style::default().bg(bg_color)),
                row_area,
            );
        }
    }

    // Scrollbar
    if total_items > visible_height {
        let scrollbar_area = Rect {
            x: list_area.x + content_width,
            y: list_area.y,
            width: 1,
            height: list_area.height,
        };
        let mut scrollbar_state =
            ScrollbarState::new(total_items.saturating_sub(visible_height)).position(state.offset);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some(" "))
                .track_style(Style::default().bg(base))
                .thumb_style(Style::default().fg(SURFACE1)),
            scrollbar_area,
            &mut scrollbar_state,
        );
    }

    // Footer
    if has_footer {
        let footer_area = chunks[5];
        let footer_chunks = Layout::vertical([
            Constraint::Length(1), // separator
            Constraint::Length(1), // text
        ])
        .split(footer_area);

        let fsep_width = footer_chunks[0].width as usize;
        let footer_sep = Paragraph::new(Line::from(Span::styled(
            "─".repeat(fsep_width),
            Style::default().fg(SURFACE1),
        )))
        .style(Style::default().bg(base));
        f.render_widget(footer_sep, footer_chunks[0]);

        let footer_text = Paragraph::new(Line::from(Span::styled(
            &config.footer,
            Style::default().fg(OVERLAY0),
        )))
        .style(Style::default().bg(base));
        f.render_widget(footer_text, footer_chunks[1]);
    }
}

// Text input widget for session name / worktree name
pub struct TextInputConfig {
    pub prompt: String,
    pub initial: String,
    pub placeholder: String,
    /// Fixed prefix displayed before the editable text (e.g., "repo/").
    /// The user cannot edit or delete this prefix.
    pub prefix: String,
}

pub enum TextInputAction {
    Confirmed(String),
    Cancelled,
}

pub fn run_text_input(config: TextInputConfig) -> TextInputAction {
    let mut input = config.initial.clone();
    let mut cursor = input.len();

    terminal::enable_raw_mode().expect("enable raw mode");
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).expect("enter alt screen");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let result = loop {
        terminal
            .draw(|f| {
                let area = f.area();
                let base = picker_bg();
                f.render_widget(Clear, area);
                f.render_widget(Block::default().style(Style::default().bg(base)), area);

                // Borderless: title (row 0), sep (row 1), input (row 2)
                let row_title = Rect {
                    x: area.x + 1,
                    y: area.y + 1,
                    width: area.width.saturating_sub(2),
                    height: 1,
                };
                let row_sep = Rect {
                    x: row_title.x,
                    y: row_title.y + 1,
                    width: row_title.width,
                    height: 1,
                };
                let row_input = Rect {
                    x: row_title.x,
                    y: row_title.y + 2,
                    width: row_title.width,
                    height: 1,
                };

                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        config.prompt.trim().to_string(),
                        Style::default().fg(YELLOW).bold(),
                    )))
                    .style(Style::default().bg(base)),
                    row_title,
                );
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        "─".repeat(row_sep.width as usize),
                        Style::default().fg(SURFACE1),
                    )))
                    .style(Style::default().bg(base)),
                    row_sep,
                );

                let prefix_len = config.prefix.chars().count() as u16;
                let line = if input.is_empty() {
                    Line::from(vec![
                        Span::styled(&config.prefix, Style::default().fg(OVERLAY1)),
                        Span::styled(&config.placeholder, Style::default().fg(OVERLAY0)),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(&config.prefix, Style::default().fg(OVERLAY1)),
                        Span::styled(&input, Style::default().fg(TEXT)),
                    ])
                };
                f.render_widget(
                    Paragraph::new(line).style(Style::default().bg(base)),
                    row_input,
                );

                let cx = row_input.x + prefix_len + cursor as u16;
                f.set_cursor_position((cx, row_input.y));
            })
            .ok();

        let Ok(ev) = event::read() else {
            continue;
        };
        let Event::Key(key) = ev else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                break TextInputAction::Cancelled;
            }
            (KeyCode::Enter, _) => {
                break TextInputAction::Confirmed(input.clone());
            }
            (KeyCode::Char(c), m) if !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                input.insert(cursor, c);
                cursor += 1;
            }
            (KeyCode::Backspace, _) => {
                if cursor > 0 {
                    cursor -= 1;
                    input.remove(cursor);
                }
            }
            (KeyCode::Delete, _) => {
                if cursor < input.len() {
                    input.remove(cursor);
                }
            }
            (KeyCode::Left, _) => {
                cursor = cursor.saturating_sub(1);
            }
            (KeyCode::Right, _) => {
                if cursor < input.len() {
                    cursor += 1;
                }
            }
            (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                cursor = 0;
            }
            (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                cursor = input.len();
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                input.clear();
                cursor = 0;
            }
            _ => {}
        }
    };

    terminal::disable_raw_mode().expect("disable raw mode");
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen).expect("leave alt screen");

    result
}

// ── Confirm dialog ──────────────────────────────────────────────────

pub struct ConfirmConfig {
    /// Lines to display above the prompt (checklist items, warnings, etc.)
    pub body: Vec<ConfirmLine>,
    /// The question to ask (e.g., "Remove worktree and kill session?")
    pub prompt: String,
}

#[derive(Clone)]
pub enum ConfirmLine {
    Ok(String),
    Warn(String),
    Error(String),
    Info(String),
}

/// Render a confirmation dialog with body lines and a y/n prompt.
/// Returns `true` if the user pressed Enter/y, `false` on Esc/n.
pub fn run_confirm(config: ConfirmConfig) -> bool {
    terminal::enable_raw_mode().expect("enable raw mode");
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).expect("enter alt screen");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let result = loop {
        terminal
            .draw(|f| {
                let area = f.area();
                let base = picker_bg();
                f.render_widget(Clear, area);
                f.render_widget(Block::default().style(Style::default().bg(base)), area);

                // Borderless: content starts at (x+1, y+1)
                let padded = Rect {
                    x: area.x + 1,
                    y: area.y + 1,
                    width: area.width.saturating_sub(2),
                    height: area.height.saturating_sub(1),
                };

                let green = Color::Rgb(0xa6, 0xe3, 0xa1);
                let yellow = Color::Rgb(0xf9, 0xe2, 0xaf);
                let red = Color::Rgb(0xf3, 0x8b, 0xa8);
                let overlay = OVERLAY1;

                let mut lines: Vec<Line> = config
                    .body
                    .iter()
                    .map(|item| match item {
                        ConfirmLine::Ok(msg) => Line::from(vec![
                            Span::styled("✓ ", Style::default().fg(green)),
                            Span::styled(msg.as_str(), Style::default().fg(TEXT)),
                        ]),
                        ConfirmLine::Warn(msg) => Line::from(vec![
                            Span::styled("! ", Style::default().fg(yellow)),
                            Span::styled(msg.as_str(), Style::default().fg(yellow)),
                        ]),
                        ConfirmLine::Error(msg) => Line::from(vec![
                            Span::styled("✗ ", Style::default().fg(red)),
                            Span::styled(msg.as_str(), Style::default().fg(red)),
                        ]),
                        ConfirmLine::Info(msg) => Line::from(vec![
                            Span::styled("  ", Style::default()),
                            Span::styled(msg.as_str(), Style::default().fg(overlay)),
                        ]),
                    })
                    .collect();

                lines.push(Line::default());
                lines.push(Line::from(Span::styled(
                    &config.prompt,
                    Style::default().fg(YELLOW).bold(),
                )));
                lines.push(Line::from(Span::styled(
                    "enter/esc",
                    Style::default().fg(OVERLAY0),
                )));

                f.render_widget(
                    Paragraph::new(lines)
                        .wrap(ratatui::widgets::Wrap { trim: false })
                        .style(Style::default().bg(base)),
                    padded,
                );
            })
            .ok();

        let Ok(ev) = event::read() else {
            continue;
        };
        let Event::Key(key) = ev else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match (key.code, key.modifiers) {
            (KeyCode::Enter, _) | (KeyCode::Char('y'), _) => break true,
            (KeyCode::Esc, _)
            | (KeyCode::Char('n'), _)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break false,
            _ => {}
        }
    };

    terminal::disable_raw_mode().expect("disable raw mode");
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen).expect("leave alt screen");

    result
}

/// Show a styled status message in the alternate screen while `f` runs.
pub fn run_with_status<T, F: FnOnce() -> T>(message: &str, f: F) -> T {
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).expect("enter alt screen");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("create terminal");

    terminal
        .draw(|frame| {
            let area = frame.area();
            let base = picker_bg();
            frame.render_widget(Clear, area);
            frame.render_widget(Block::default().style(Style::default().bg(base)), area);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(SURFACE1))
                .style(Style::default().bg(base));

            let inner = block.inner(area);
            frame.render_widget(block, area);

            let padded = Rect {
                x: inner.x + 1,
                y: inner.y,
                width: inner.width.saturating_sub(2),
                height: inner.height,
            };

            let line = Line::from(Span::styled(message, Style::default().fg(OVERLAY1)));
            frame.render_widget(
                Paragraph::new(line).style(Style::default().bg(base)),
                padded,
            );
        })
        .ok();

    let result = f();

    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen).expect("leave alt screen");

    result
}
