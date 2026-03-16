use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use crate::order::{load_lines, save_lines};
use crate::picker::{
    PickerAction, PickerConfig, PickerItem, TextInputAction, TextInputConfig, run_picker,
    run_text_input,
};
use crate::tmux::{git_branch, git_toplevel, home, tmux};

// Catppuccin Mocha colors
const TEXT: Color = Color::Rgb(0xcd, 0xd6, 0xf4);
const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const CYAN: Color = Color::Rgb(0x89, 0xb4, 0xfa);

fn favorites_file() -> PathBuf {
    home().join(".config/tmux/.session-favorites")
}

fn lru_file() -> PathBuf {
    home().join(".config/tmux/.project-lru")
}

fn load_favorites() -> HashSet<String> {
    load_lines(&favorites_file()).into_iter().collect()
}

/// Record a project directory as most recently used (moves it to the top).
fn touch_lru(dir: &str) {
    let path = lru_file();
    let mut lines = load_lines(&path);
    lines.retain(|l| l != dir);
    lines.insert(0, dir.to_string());
    // Keep at most 100 entries
    lines.truncate(100);
    save_lines(&path, &lines);
}

/// Sort directories by LRU order. Dirs not in LRU go to the end, alphabetically.
fn sort_by_lru(dirs: &mut [PathBuf]) {
    let lru = load_lines(&lru_file());
    let rank: HashMap<String, usize> = lru
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();
    dirs.sort_by(|a, b| {
        let ra = rank.get(a.to_str().unwrap_or(""));
        let rb = rank.get(b.to_str().unwrap_or(""));
        match (ra, rb) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(b),
        }
    });
}

fn collect_dirs(filter: &str) -> Vec<PathBuf> {
    let h = home();
    let mut dirs: Vec<PathBuf> = Vec::new();
    match filter {
        "home" => {
            dirs.push(h.clone());
            let d = h.join("dotfiles");
            if d.is_dir() {
                dirs.push(d);
            }
        }
        "config" => {
            if let Ok(entries) = fs::read_dir(h.join(".config")) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        dirs.push(p);
                    }
                }
            }
        }
        "src" => {
            if let Ok(entries) = fs::read_dir(h.join("src")) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        dirs.push(p);
                    }
                }
            }
        }
        _ => {
            dirs.push(h.clone());
            let d = h.join("dotfiles");
            if d.is_dir() {
                dirs.push(d);
            }
            for parent in ["src", ".config"] {
                if let Ok(entries) = fs::read_dir(h.join(parent)) {
                    for e in entries.flatten() {
                        let p = e.path();
                        if p.is_dir() {
                            dirs.push(p);
                        }
                    }
                }
            }
            dirs.dedup();
        }
    }
    sort_by_lru(&mut dirs);
    dirs
}

fn build_project_items(filter: &str) -> Vec<PickerItem> {
    let favs = load_favorites();
    let h = home();
    let mut items = Vec::new();

    // Favorites first
    for fav in load_lines(&favorites_file()) {
        let p = PathBuf::from(&fav);
        if !p.is_dir() {
            continue;
        }
        let display = fav.strip_prefix(h.to_str().unwrap_or("")).unwrap_or(&fav);
        items.push(PickerItem {
            id: fav.clone(),
            display: format!("\u{f005}  ~{display}"),
            style: Style::default().fg(YELLOW),
            selectable: true,
        });
    }

    let dirs = collect_dirs(filter);
    let home_str = h.to_str().unwrap_or("");
    for dir in &dirs {
        let s = dir.to_str().unwrap_or("");
        if favs.contains(s) {
            continue;
        }
        let display = s.strip_prefix(home_str).unwrap_or(s);
        items.push(PickerItem {
            id: s.to_string(),
            display: format!("   ~{display}"),
            style: Style::default().fg(TEXT),
            selectable: true,
        });
    }

    items
}

pub fn cmd_project_list(args: &[String]) {
    let filter = args.first().map_or("all", String::as_str);
    let favs = load_favorites();
    let h = home();
    let yellow = "\x1b[33m";
    let gray = "\x1b[90m";
    let reset = "\x1b[0m";
    let icon_fav = "\u{f005}  ";

    for fav in load_lines(&favorites_file()) {
        let p = PathBuf::from(&fav);
        if !p.is_dir() {
            continue;
        }
        let display = fav.strip_prefix(h.to_str().unwrap_or("")).unwrap_or(&fav);
        println!("{yellow}{icon_fav}{reset}{gray}~{reset}{display}");
    }

    let dirs = collect_dirs(filter);
    let home_str = h.to_str().unwrap_or("");
    for dir in &dirs {
        let s = dir.to_str().unwrap_or("");
        if favs.contains(s) {
            continue;
        }
        let display = s.strip_prefix(home_str).unwrap_or(s);
        println!("   {gray}~{reset}{display}");
    }
}

pub fn cmd_toggle_favorite(args: &[String]) {
    let Some(raw) = args.first() else {
        return;
    };
    let dir = raw.replace('~', home().to_str().unwrap_or(""));
    let path = favorites_file();
    let _ = fs::OpenOptions::new().create(true).append(true).open(&path);
    let mut lines = load_lines(&path);
    if let Some(pos) = lines.iter().position(|l| l == &dir) {
        lines.remove(pos);
    } else {
        lines.push(dir);
    }
    save_lines(&path, &lines);
}

pub fn cmd_new_session() {
    // Phase 1: Directory picker
    let selected_dir = match phase_directory_picker() {
        Some(d) => d,
        None => return,
    };

    // Record LRU
    touch_lru(selected_dir.to_str().unwrap_or(""));

    // Detect bare repo / worktrees
    let is_bare = selected_dir.extension().is_some_and(|e| e == "git") && selected_dir.is_dir();
    let has_git = selected_dir.join(".git").exists();
    let mut final_dir = selected_dir.clone();
    let mut branch_name = String::new();

    if has_git || is_bare {
        match phase_worktree_picker(&selected_dir, is_bare) {
            WorktreeResult::Selected(dir, branch) => {
                final_dir = dir;
                branch_name = branch;
            }
            WorktreeResult::NoWorktrees => {
                branch_name = git_branch(selected_dir.to_str().unwrap_or(""));
            }
            WorktreeResult::Cancelled => return,
        }
    }

    // Compute session name
    let repo_name = if is_bare {
        selected_dir
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().replace(".git", ""))
    } else {
        git_toplevel(final_dir.to_str().unwrap_or(""))
            .and_then(|tl| {
                PathBuf::from(tl)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .unwrap_or_default()
    };

    let suffix = if !branch_name.is_empty() {
        branch_name.clone()
    } else {
        final_dir
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().replace(".git", ""))
    };

    let default_name = if !repo_name.is_empty() && repo_name != suffix {
        format!("{repo_name}/{suffix}")
    } else if !repo_name.is_empty() {
        repo_name
    } else {
        suffix
    };

    // Phase 3: Session name input
    let session_name = match run_text_input(TextInputConfig {
        prompt: "\u{f044}  Session: ".to_string(),
        initial: default_name.clone(),
    }) {
        TextInputAction::Confirmed(s) => {
            if s.is_empty() {
                default_name
            } else {
                s
            }
        }
        TextInputAction::Cancelled => return,
    };

    // Check collision
    if Command::new("tmux")
        .args(["has-session", "-t", &format!("={session_name}")])
        .status()
        .is_ok_and(|s| s.success())
    {
        eprintln!("\x1b[33mSession '{session_name}' exists, switching...\x1b[0m");
        tmux(&["switch-client", "-t", &session_name]);
        return;
    }

    // Create session with 3 windows
    let dir_str = final_dir.to_str().unwrap_or(".");
    tmux(&[
        "new-session",
        "-d",
        "-s",
        &session_name,
        "-n",
        "ai",
        "-c",
        dir_str,
        ";",
        "new-window",
        "-t",
        &session_name,
        "-n",
        "vi",
        "-c",
        dir_str,
        ";",
        "new-window",
        "-t",
        &session_name,
        "-n",
        "sh",
        "-c",
        dir_str,
        ";",
        "select-window",
        "-t",
        &format!("{session_name}:ai"),
        ";",
        "switch-client",
        "-t",
        &session_name,
    ]);
}

fn phase_directory_picker() -> Option<PathBuf> {
    let self_bin =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("tmux-session"));
    let self_path = self_bin.to_string_lossy().to_string();
    let mut current_filter = "all".to_string();

    loop {
        let items = build_project_items(&current_filter);

        let config = PickerConfig {
            prompt: "\u{f07b}  Project: ".to_string(),
            header: "ctrl-f: toggle \u{f005} │ 1: ~ │ 2: ~/.config │ 3: ~/src │ 0: all".to_string(),
        };

        let mut custom_keys = HashMap::new();
        custom_keys.insert(
            (KeyCode::Char('f'), KeyModifiers::CONTROL),
            "toggle-fav".to_string(),
        );
        custom_keys.insert(
            (KeyCode::Char('1'), KeyModifiers::NONE),
            "filter-home".to_string(),
        );
        custom_keys.insert(
            (KeyCode::Char('2'), KeyModifiers::NONE),
            "filter-config".to_string(),
        );
        custom_keys.insert(
            (KeyCode::Char('3'), KeyModifiers::NONE),
            "filter-src".to_string(),
        );
        custom_keys.insert(
            (KeyCode::Char('0'), KeyModifiers::NONE),
            "filter-all".to_string(),
        );

        match run_picker(items, config, custom_keys) {
            PickerAction::Selected(id) => {
                if id.is_empty() {
                    return None;
                }
                return Some(PathBuf::from(id));
            }
            PickerAction::Cancelled => return None,
            PickerAction::Custom(action, id) => match action.as_str() {
                "toggle-fav" => {
                    if !id.is_empty() {
                        let _ = Command::new(&self_path)
                            .args(["toggle-favorite", &id])
                            .output();
                    }
                }
                "filter-home" => current_filter = "home".to_string(),
                "filter-config" => current_filter = "config".to_string(),
                "filter-src" => current_filter = "src".to_string(),
                "filter-all" => current_filter = "all".to_string(),
                _ => {}
            },
        }
    }
}

enum WorktreeResult {
    Selected(PathBuf, String),
    NoWorktrees,
    Cancelled,
}

fn phase_worktree_picker(selected_dir: &Path, is_bare: bool) -> WorktreeResult {
    let wt_output = Command::new("git")
        .args([
            "-C",
            selected_dir.to_str().unwrap_or("."),
            "worktree",
            "list",
        ])
        .output();
    let worktrees: Vec<String> = wt_output
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty() && (!is_bare || !l.contains("(bare)")))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();

    if (!is_bare && worktrees.len() <= 1) || worktrees.is_empty() {
        return WorktreeResult::NoWorktrees;
    }

    let mut items = Vec::new();
    items.push(PickerItem {
        id: "__new__".to_string(),
        display: "+ New worktree".to_string(),
        style: Style::default().fg(CYAN),
        selectable: true,
    });

    for wt in &worktrees {
        let wt_path = wt.split_whitespace().next().unwrap_or("");
        let wt_name = PathBuf::from(wt_path)
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().to_string());
        let br = git_branch(wt_path);
        let display = if br.is_empty() {
            wt_name.clone()
        } else {
            format!("{wt_name} \u{2190} {br}")
        };
        items.push(PickerItem {
            id: wt_path.to_string(),
            display,
            style: Style::default().fg(TEXT),
            selectable: true,
        });
    }

    let config = PickerConfig {
        prompt: "\u{f126}  Worktree: ".to_string(),
        header: String::new(),
    };

    match run_picker(items, config, HashMap::new()) {
        PickerAction::Selected(id) => {
            if id == "__new__" {
                match run_text_input(TextInputConfig {
                    prompt: "\u{f067}  Worktree name: ".to_string(),
                    initial: String::new(),
                }) {
                    TextInputAction::Confirmed(wt_name) if !wt_name.is_empty() => {
                        let gg = home().join("bin/gg-create-worktree");
                        let result = Command::new(gg)
                            .args([
                                "--repo",
                                selected_dir.to_str().unwrap_or("."),
                                "--name",
                                &wt_name,
                            ])
                            .output();
                        match result {
                            Ok(o) if o.status.success() => {
                                let p = String::from_utf8_lossy(&o.stdout)
                                    .lines()
                                    .last()
                                    .unwrap_or("")
                                    .trim()
                                    .to_string();
                                if !p.is_empty() && PathBuf::from(&p).is_dir() {
                                    let branch = git_branch(&p);
                                    WorktreeResult::Selected(PathBuf::from(p), branch)
                                } else {
                                    WorktreeResult::Cancelled
                                }
                            }
                            _ => WorktreeResult::Cancelled,
                        }
                    }
                    _ => WorktreeResult::Cancelled,
                }
            } else {
                let branch = git_branch(&id);
                WorktreeResult::Selected(PathBuf::from(id), branch)
            }
        }
        PickerAction::Cancelled => WorktreeResult::Cancelled,
        PickerAction::Custom(..) => WorktreeResult::Cancelled,
    }
}
