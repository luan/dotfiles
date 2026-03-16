use std::collections::HashMap;
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use nucleo_matcher::pattern::{Atom, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};

// Catppuccin Mocha
const BASE: Color = Color::Rgb(0x1e, 0x1e, 0x2e);
const SURFACE0: Color = Color::Rgb(0x31, 0x32, 0x44);
const TEXT: Color = Color::Rgb(0xcd, 0xd6, 0xf4);
const OVERLAY0: Color = Color::Rgb(0x6c, 0x70, 0x86);
const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const RED: Color = Color::Rgb(0xf3, 0x8b, 0xa8);

pub struct PickerItem {
    pub id: String,
    pub display: String,
    pub style: Style,
    pub selectable: bool,
}

#[derive(Clone)]
pub struct PickerConfig {
    pub prompt: String,
    pub header: String,
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
    snap_to_first_selectable(&items, &mut state);

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
                // Keep headers if any of their group items match
                // We'll handle this in a second pass
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
        // Sort by score descending, then by original order
        state
            .filtered
            .sort_by(|a, b| b.score.cmp(&a.score).then(a.idx.cmp(&b.idx)));

        // Re-insert headers: a header is shown if any item directly after it (before next header) is in the filtered set
        let filtered_indices: std::collections::HashSet<usize> =
            state.filtered.iter().map(|f| f.idx).collect();
        let mut headers_to_insert: Vec<(usize, usize)> = Vec::new(); // (insert_pos_in_filtered, item_idx)
        for fi_pos in 0..state.filtered.len() {
            let item_idx = state.filtered[fi_pos].idx;
            // Look backwards from this item to find its header
            if item_idx > 0 {
                let prev_idx = item_idx - 1;
                if !items[prev_idx].selectable && !filtered_indices.contains(&prev_idx) {
                    // Check we haven't already inserted this header
                    if !headers_to_insert.iter().any(|(_, hi)| *hi == prev_idx) {
                        headers_to_insert.push((fi_pos, prev_idx));
                    }
                }
            }
        }
        // Insert headers in reverse order so positions don't shift
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
    // Find first selectable
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
    f.render_widget(Clear, area);
    f.render_widget(Block::default().style(Style::default().bg(BASE)), area);

    // Layout: prompt (1) + header (1) + separator (1) + list (rest)
    let header_lines = if config.header.is_empty() { 0 } else { 1 };
    let chunks = Layout::vertical([
        Constraint::Length(1),            // prompt
        Constraint::Length(header_lines), // header
        Constraint::Length(1),            // separator
        Constraint::Min(1),               // list
    ])
    .split(area);

    // Prompt line
    let prompt_text = Line::from(vec![
        Span::styled(&config.prompt, Style::default().fg(YELLOW).bold()),
        Span::styled(&state.input, Style::default().fg(TEXT)),
    ]);
    f.render_widget(
        Paragraph::new(prompt_text).style(Style::default().bg(BASE)),
        chunks[0],
    );

    // Cursor
    let cursor_x = chunks[0].x + config.prompt.chars().count() as u16 + state.cursor as u16;
    f.set_cursor_position((cursor_x, chunks[0].y));

    // Header
    if !config.header.is_empty() {
        let header = Paragraph::new(Line::from(Span::styled(
            &config.header,
            Style::default().fg(OVERLAY0),
        )))
        .style(Style::default().bg(BASE));
        f.render_widget(header, chunks[1]);
    }

    // Separator
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(OVERLAY0),
    )))
    .style(Style::default().bg(BASE));
    f.render_widget(sep, chunks[2]);

    // List
    let list_area = chunks[3];
    let visible_height = list_area.height as usize;

    // Adjust offset for scrolling
    if state.selected < state.offset {
        state.offset = state.selected;
    }
    if state.selected >= state.offset + visible_height {
        state.offset = state.selected - visible_height + 1;
    }

    let list_items: Vec<ListItem> = state
        .filtered
        .iter()
        .skip(state.offset)
        .take(visible_height)
        .enumerate()
        .map(|(vi, fi)| {
            let item = &items[fi.idx];
            let is_selected = vi + state.offset == state.selected;

            let mut spans = Vec::new();
            if is_selected && item.selectable {
                spans.push(Span::styled("❯ ", Style::default().fg(RED)));
            } else {
                spans.push(Span::raw("  "));
            }

            // Build display with highlighted match characters
            if !fi.indices.is_empty() && !state.input.is_empty() {
                let match_set: std::collections::HashSet<u32> =
                    fi.indices.iter().copied().collect();
                for (ci, ch) in item.display.chars().enumerate() {
                    if match_set.contains(&(ci as u32)) {
                        spans.push(Span::styled(
                            ch.to_string(),
                            item.style.fg(YELLOW).underlined(),
                        ));
                    } else {
                        spans.push(Span::styled(ch.to_string(), item.style));
                    }
                }
            } else {
                spans.push(Span::styled(&item.display, item.style));
            }

            let bg = if is_selected && item.selectable {
                SURFACE0
            } else {
                BASE
            };
            ListItem::new(Line::from(spans)).style(Style::default().bg(bg))
        })
        .collect();

    let list = List::new(list_items).style(Style::default().bg(BASE));
    f.render_widget(list, list_area);
}

// Text input widget for session name / worktree name
pub struct TextInputConfig {
    pub prompt: String,
    pub initial: String,
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
                f.render_widget(Clear, area);
                f.render_widget(Block::default().style(Style::default().bg(BASE)), area);

                let chunks =
                    Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);

                let line = Line::from(vec![
                    Span::styled(&config.prompt, Style::default().fg(YELLOW).bold()),
                    Span::styled(&input, Style::default().fg(TEXT)),
                ]);
                f.render_widget(
                    Paragraph::new(line).style(Style::default().bg(BASE)),
                    chunks[0],
                );

                let cx = chunks[0].x + config.prompt.chars().count() as u16 + cursor as u16;
                f.set_cursor_position((cx, chunks[0].y));
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
