use std::collections::{HashMap, HashSet};
use std::process::Command;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use crate::group::{GroupMeta, session_group, session_suffix};
use crate::order::{compute_order, hidden_file, load_lines};
use crate::palette::{OVERLAY0, TEXT, group_glyph, hex_to_color, num_glyph};
use crate::picker::{PickerAction, PickerConfig, PickerItem, run_picker};
use crate::tmux::tmux;

const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);

fn build_items(sessions: &[String], hidden: &HashSet<String>, cur: &str) -> Vec<PickerItem> {
    let meta = GroupMeta::new(sessions);

    let color_list = crate::color::compute_session_colors(sessions, &meta);
    let session_colors: HashMap<&str, (Color, Color)> = color_list
        .iter()
        .map(|(n, c, d)| (n.as_str(), (hex_to_color(c), hex_to_color(d))))
        .collect();

    let mut items = Vec::new();
    let mut idx = 0usize;
    let mut last_group = String::new();

    for name in sessions {
        let group = session_group(name);
        let gtotal = if group.is_empty() {
            0
        } else {
            *meta.counts.get(group).unwrap_or(&0)
        };
        let is_hidden = hidden.contains(name);
        let is_current = name == cur;
        let (color, dim_color) = session_colors
            .get(name.as_str())
            .map_or((None, None), |(c, d)| (Some(*c), Some(*d)));

        if !group.is_empty() && gtotal > 1 {
            if group != last_group {
                let gg = group_glyph(gtotal, false);
                items.push(PickerItem {
                    id: format!("__group__{group}"),
                    display: format!("{gg} {group}"),
                    style: Style::default().fg(OVERLAY0),
                    selectable: true,
                    color,
                    dim_color,
                    right_label: String::new(),
                });
            }
            let glyph = num_glyph(idx, false);
            idx += 1;
            let suffix = {
                let s = session_suffix(name);
                if s.is_empty() { group } else { s }
            };
            let (display, style) = if is_hidden {
                (
                    format!("  {glyph} {suffix} \u{f0513}"),
                    Style::default().fg(YELLOW),
                )
            } else {
                (format!("  {glyph} {suffix}"), Style::default().fg(TEXT))
            };
            let right_label = if is_current {
                "\u{2190}".to_string()
            } else {
                String::new()
            };
            items.push(PickerItem {
                id: name.clone(),
                display,
                style,
                selectable: true,
                color,
                dim_color,
                right_label,
            });
        } else {
            let flat = if !group.is_empty() {
                group
            } else {
                name.as_str()
            };
            let glyph = num_glyph(idx, false);
            idx += 1;
            let (display, style) = if is_hidden {
                (
                    format!("{glyph} {flat} \u{f0513}"),
                    Style::default().fg(YELLOW),
                )
            } else {
                (format!("{glyph} {flat}"), Style::default().fg(TEXT))
            };
            let right_label = if is_current {
                "\u{2190}".to_string()
            } else {
                String::new()
            };
            items.push(PickerItem {
                id: name.clone(),
                display,
                style,
                selectable: true,
                color,
                dim_color,
                right_label,
            });
        }
        last_group = group.to_string();
    }

    items
}

pub(crate) fn cmd_chooser_list() {
    let cur = tmux(&["display-message", "-p", "#S"]);
    let alive: HashSet<String> = tmux(&["list-sessions", "-F", "#S"])
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    let sessions = compute_order(&alive, true);
    let hidden: HashSet<String> = load_lines(&hidden_file()).into_iter().collect();
    let meta = GroupMeta::new(&sessions);

    let gray = "\x1b[90m";
    let yellow = "\x1b[33m";
    let reset = "\x1b[0m";

    let mut idx = 0usize;
    let mut last_group = String::new();

    for name in &sessions {
        let group = session_group(name);
        let gtotal = if group.is_empty() {
            0
        } else {
            *meta.counts.get(group).unwrap_or(&0)
        };
        let is_hidden = hidden.contains(name);
        let is_current = name == &cur;

        if !group.is_empty() && gtotal > 1 {
            if group != last_group {
                println!("__header__\t{gray}  {group}{reset}");
            }
            let suffix = session_suffix(name);
            let display = if suffix.is_empty() { group } else { suffix };
            idx += 1;
            if is_hidden {
                println!("{name}\t{yellow}  {idx}: {display} \u{f0513}{reset}");
            } else if is_current {
                println!("{name}\t  {idx}: {display} {gray}\u{2190}{reset}");
            } else {
                println!("{name}\t  {idx}: {display}");
            }
        } else {
            let flat = if !group.is_empty() {
                group
            } else {
                name.as_str()
            };
            idx += 1;
            if is_hidden {
                println!("{name}\t{yellow}{idx}: {flat} \u{f0513}{reset}");
            } else if is_current {
                println!("{name}\t{idx}: {flat} {gray}\u{2190}{reset}");
            } else {
                println!("{name}\t{idx}: {flat}");
            }
        }
        last_group = group.to_string();
    }
}

pub(crate) fn cmd_chooser() {
    let self_bin = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("mux"));
    let self_path = self_bin.to_string_lossy().to_string();

    let cur = tmux(&["display-message", "-p", "#S"]);
    let alive: HashSet<String> = tmux(&["list-sessions", "-F", "#S"])
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    let sessions = compute_order(&alive, true);
    let hidden: HashSet<String> = load_lines(&hidden_file()).into_iter().collect();
    let items = build_items(&sessions, &hidden, &cur);

    let custom_keys = {
        let mut m = HashMap::new();
        m.insert(
            (KeyCode::Char('h'), KeyModifiers::ALT),
            "toggle-hidden".to_string(),
        );
        m.insert(
            (KeyCode::Char('j'), KeyModifiers::ALT),
            "move-down".to_string(),
        );
        m.insert(
            (KeyCode::Char('k'), KeyModifiers::ALT),
            "move-up".to_string(),
        );
        m
    };

    let mut current_items = items;
    let mut restore_id: Option<String> = None;

    loop {
        let config = PickerConfig {
            prompt: "Session".to_string(),
            footer: "alt-h hide \u{2502} alt-j/k move".to_string(),
            placeholder: "filter...".to_string(),
            initial_id: restore_id.take(),
        };

        let action = run_picker(current_items, config, custom_keys.clone());
        match action {
            PickerAction::Selected(id) => {
                // Group headers aren't switchable
                if !id.is_empty() && !id.starts_with("__group__") {
                    tmux(&["switch-client", "-t", &id]);
                }
                break;
            }
            PickerAction::Cancelled => break,
            PickerAction::Custom(action_name, id) => {
                if !id.is_empty() {
                    let is_group = id.starts_with("__group__");
                    let group_name = id.strip_prefix("__group__").unwrap_or("");

                    match action_name.as_str() {
                        "toggle-hidden" if !is_group => {
                            let _ = Command::new(&self_path).args(["hide-toggle", &id]).output();
                        }
                        "move-down" => {
                            if is_group {
                                let mut store = crate::order::SessionStore::load();
                                if store.move_group(group_name, "down") {
                                    store.save();
                                }
                            } else {
                                let _ = Command::new(&self_path)
                                    .args(["move", "down", &id])
                                    .output();
                            }
                        }
                        "move-up" => {
                            if is_group {
                                let mut store = crate::order::SessionStore::load();
                                if store.move_group(group_name, "up") {
                                    store.save();
                                }
                            } else {
                                let _ = Command::new(&self_path).args(["move", "up", &id]).output();
                            }
                        }
                        _ => {}
                    }
                    // Restore cursor to the same item after rebuild
                    restore_id = Some(id);
                }
                // Rebuild state after mutation
                let cur = tmux(&["display-message", "-p", "#S"]);
                let alive: HashSet<String> = tmux(&["list-sessions", "-F", "#S"])
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(String::from)
                    .collect();
                let sessions = compute_order(&alive, true);
                let hidden: HashSet<String> = load_lines(&hidden_file()).into_iter().collect();
                current_items = build_items(&sessions, &hidden, &cur);
            }
        }
    }
}
