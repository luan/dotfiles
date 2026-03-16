use std::collections::{HashMap, HashSet};
use std::process::Command;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use crate::group::{GroupMeta, session_group, session_suffix};
use crate::order::{compute_order, hidden_file, load_lines};
use crate::picker::{PickerAction, PickerConfig, PickerItem, run_picker};
use crate::tmux::{home, tmux};

// Catppuccin Mocha colors
const TEXT: Color = Color::Rgb(0xcd, 0xd6, 0xf4);
const OVERLAY0: Color = Color::Rgb(0x6c, 0x70, 0x86);
const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);

fn build_items(sessions: &[String], hidden: &HashSet<String>, cur: &str) -> Vec<PickerItem> {
    let meta = GroupMeta::new(sessions);
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

        if !group.is_empty() && gtotal > 1 {
            if group != last_group {
                items.push(PickerItem {
                    id: String::new(),
                    display: group.to_string(),
                    style: Style::default().fg(OVERLAY0),
                    selectable: false,
                });
            }
            idx += 1;
            let suffix = session_suffix(name);
            let display = if is_hidden {
                format!("  {idx}: {suffix} \u{f0513}")
            } else if is_current {
                format!("  {idx}: {suffix} \u{2190}")
            } else {
                format!("  {idx}: {suffix}")
            };
            let style = if is_hidden {
                Style::default().fg(YELLOW)
            } else {
                Style::default().fg(TEXT)
            };
            items.push(PickerItem {
                id: name.clone(),
                display,
                style,
                selectable: true,
            });
        } else {
            let flat = if !group.is_empty() {
                group
            } else {
                name.as_str()
            };
            idx += 1;
            let display = if is_hidden {
                format!("{idx}: {flat} \u{f0513}")
            } else if is_current {
                format!("{idx}: {flat} \u{2190}")
            } else {
                format!("{idx}: {flat}")
            };
            let style = if is_hidden {
                Style::default().fg(YELLOW)
            } else {
                Style::default().fg(TEXT)
            };
            items.push(PickerItem {
                id: name.clone(),
                display,
                style,
                selectable: true,
            });
        }
        last_group = group.to_string();
    }

    items
}

pub fn cmd_chooser_list() {
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
            let display = session_suffix(name);
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

pub fn cmd_chooser() {
    let scripts = home().join(".config/tmux/scripts");
    let hide_script = scripts
        .join("session-hide-toggle.sh")
        .to_string_lossy()
        .to_string();
    let self_bin =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("tmux-session"));
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

    let config = PickerConfig {
        prompt: " Session: ".to_string(),
        header: "alt-h: hidden │ alt-j/k: move".to_string(),
    };

    let mut custom_keys = HashMap::new();
    custom_keys.insert(
        (KeyCode::Char('h'), KeyModifiers::ALT),
        "toggle-hidden".to_string(),
    );
    custom_keys.insert(
        (KeyCode::Char('j'), KeyModifiers::ALT),
        "move-down".to_string(),
    );
    custom_keys.insert(
        (KeyCode::Char('k'), KeyModifiers::ALT),
        "move-up".to_string(),
    );

    let mut current_items = items;

    loop {
        let action = run_picker(current_items, config.clone(), custom_keys.clone());
        match action {
            PickerAction::Selected(id) => {
                if !id.is_empty() {
                    tmux(&["switch-client", "-t", &id]);
                }
                break;
            }
            PickerAction::Cancelled => break,
            PickerAction::Custom(action_name, id) => {
                if !id.is_empty() {
                    match action_name.as_str() {
                        "toggle-hidden" => {
                            let _ = Command::new("bash").args([&hide_script, &id]).output();
                        }
                        "move-down" => {
                            let _ = Command::new(&self_path)
                                .args(["move", "down", &id])
                                .output();
                        }
                        "move-up" => {
                            let _ = Command::new(&self_path).args(["move", "up", &id]).output();
                        }
                        _ => {}
                    }
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
