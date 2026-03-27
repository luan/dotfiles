use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use crate::order::{load_lines, save_lines};
use crate::picker::{
    PickerAction, PickerConfig, PickerItem, TextInputAction, TextInputConfig, run_picker,
    run_text_input,
};
use crate::tmux::{git_toplevel, home, tmux};

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
            for name in ["dotfiles", ".claude"] {
                let d = h.join(name);
                if d.is_dir() {
                    dirs.push(d);
                }
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
            for name in ["dotfiles", ".claude"] {
                let d = h.join(name);
                if d.is_dir() {
                    dirs.push(d);
                }
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
            color: Some(YELLOW),
            dim_color: None,
            right_label: String::new(),
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
            color: None,
            dim_color: None,
            right_label: String::new(),
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
    let (selected_dir, auto_worktree) = match phase_directory_picker() {
        Some(DirectoryPickerResult::Normal(d)) => (d, false),
        Some(DirectoryPickerResult::AutoWorktree(d)) => (d, true),
        None => return,
    };

    // Record LRU
    touch_lru(selected_dir.to_str().unwrap_or(""));

    // Detect bare repo / worktrees
    let is_bare = selected_dir.extension().is_some_and(|e| e == "git") && selected_dir.is_dir();
    let has_git = selected_dir.join(".git").exists();
    let mut final_dir = selected_dir.clone();

    if has_git || is_bare {
        let entries = list_worktrees(&selected_dir);
        if auto_worktree {
            if let Some(wt_dir) = find_detached_worktree(&entries) {
                final_dir = wt_dir;
            } else {
                match phase_worktree_picker(&selected_dir, entries) {
                    WorktreeResult::Selected(dir) => final_dir = dir,
                    WorktreeResult::NoWorktrees => {}
                    WorktreeResult::Cancelled => return,
                }
            }
        } else {
            match phase_worktree_picker(&selected_dir, entries) {
                WorktreeResult::Selected(dir) => final_dir = dir,
                WorktreeResult::NoWorktrees => {}
                WorktreeResult::Cancelled => return,
            }
        }
    }

    // Compute session name
    let repo_name = if is_bare {
        selected_dir.file_name().map_or(String::new(), |n| {
            n.to_string_lossy()
                .replace(".git", "")
                .trim_start_matches('.')
                .to_string()
        })
    } else {
        git_toplevel(final_dir.to_str().unwrap_or(""))
            .and_then(|tl| {
                PathBuf::from(tl)
                    .file_name()
                    .map(|n| n.to_string_lossy().trim_start_matches('.').to_string())
            })
            .unwrap_or_default()
    };

    let suffix = final_dir.file_name().map_or(String::new(), |n| {
        n.to_string_lossy()
            .replace(".git", "")
            .trim_start_matches('.')
            .to_string()
    });

    let default_name = if !repo_name.is_empty() && repo_name != suffix {
        format!("{repo_name}/{suffix}")
    } else if !repo_name.is_empty() {
        repo_name
    } else {
        suffix
    };

    // Phase 3: Session name input
    let session_name = match run_text_input(TextInputConfig {
        prompt: "\u{f044}  Session".to_string(),
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

fn phase_directory_picker() -> Option<DirectoryPickerResult> {
    let self_bin =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("tmux-session"));
    let self_path = self_bin.to_string_lossy().to_string();
    let mut current_filter = "all".to_string();

    loop {
        let items = build_project_items(&current_filter);

        let config = PickerConfig {
            prompt: "Project".to_string(),
            footer:
                "ctrl-f \u{f005} \u{2502} 1 ~ \u{2502} 2 ~/.config \u{2502} 3 ~/src \u{2502} 0 all \u{2502} alt-enter worktree"
                    .to_string(),
            placeholder: "filter...".to_string(),
            initial_id: None,
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
        custom_keys.insert(
            (KeyCode::Enter, KeyModifiers::ALT),
            "auto-worktree".to_string(),
        );

        match run_picker(items, config, custom_keys) {
            PickerAction::Selected(id) => {
                if id.is_empty() {
                    return None;
                }
                return Some(DirectoryPickerResult::Normal(PathBuf::from(id)));
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
                "auto-worktree" => {
                    if !id.is_empty() {
                        return Some(DirectoryPickerResult::AutoWorktree(PathBuf::from(id)));
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

enum DirectoryPickerResult {
    Normal(PathBuf),
    AutoWorktree(PathBuf),
}

enum WorktreeResult {
    Selected(PathBuf),
    NoWorktrees,
    Cancelled,
}

// --- git worktree integration ---

struct WtEntry {
    path: String,
    branch: Option<String>,
    detached: bool,
}

fn git_output(dir: &Path, args: &[&str]) -> Option<Vec<u8>> {
    let mut child = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let out = child.wait_with_output().ok()?;
                return out.status.success().then_some(out.stdout);
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(_) => return None,
        }
    }
}

fn list_worktrees(dir: &Path) -> Vec<WtEntry> {
    let Some(stdout) = git_output(dir, &["worktree", "list", "--porcelain"]) else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&stdout);

    let mut entries = Vec::new();
    let mut path: Option<String> = None;
    let mut branch: Option<String> = None;
    let mut detached = false;
    let mut bare = false;

    for line in text.lines() {
        if line.is_empty() {
            if let Some(p) = path.take() {
                if !bare {
                    entries.push(WtEntry {
                        path: p,
                        branch: branch.take(),
                        detached,
                    });
                } else {
                    branch = None;
                }
                detached = false;
                bare = false;
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("worktree ") {
            path = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("branch ") {
            branch = Some(rest.strip_prefix("refs/heads/").unwrap_or(rest).to_string());
        } else if line == "detached" {
            detached = true;
        } else if line == "bare" {
            bare = true;
        }
    }
    // Handle last entry (no trailing blank line)
    if let Some(p) = path.filter(|_| !bare) {
        entries.push(WtEntry {
            path: p,
            branch,
            detached,
        });
    }
    entries
}

fn find_detached_worktree(entries: &[WtEntry]) -> Option<PathBuf> {
    entries
        .iter()
        .find(|e| e.detached)
        .map(|e| PathBuf::from(&e.path))
        .filter(|p| p.is_dir())
}

fn phase_worktree_picker(selected_dir: &Path, entries: Vec<WtEntry>) -> WorktreeResult {
    if entries.is_empty() {
        return WorktreeResult::NoWorktrees;
    }

    let mut items = Vec::new();
    items.push(PickerItem {
        id: "__new__".to_string(),
        display: "+ New worktree".to_string(),
        style: Style::default().fg(CYAN),
        selectable: true,
        color: Some(CYAN),
        dim_color: None,
        right_label: String::new(),
    });

    for entry in &entries {
        let name = PathBuf::from(&entry.path)
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().to_string());
        let branch = entry.branch.as_deref().unwrap_or("");

        let display = if entry.detached {
            format!("{name} (detached)")
        } else if !branch.is_empty() {
            format!("{name} \u{2190} {branch}")
        } else {
            name
        };

        items.push(PickerItem {
            id: entry.path.clone(),
            display,
            style: Style::default().fg(TEXT),
            selectable: true,
            color: None,
            dim_color: None,
            right_label: String::new(),
        });
    }

    let config = PickerConfig {
        prompt: "Worktree".to_string(),
        footer: String::new(),
        initial_id: None,
        placeholder: "filter...".to_string(),
    };

    match run_picker(items, config, HashMap::new()) {
        PickerAction::Selected(id) => {
            if id == "__new__" {
                match run_text_input(TextInputConfig {
                    prompt: "\u{f067}  Worktree".to_string(),
                    initial: String::new(),
                }) {
                    TextInputAction::Confirmed(wt_name) if !wt_name.is_empty() => {
                        let result = Command::new("wt")
                            .args([
                                "switch",
                                "--create",
                                &wt_name,
                                "--no-cd",
                                "-y",
                                "-C",
                                selected_dir.to_str().unwrap_or("."),
                            ])
                            .output();
                        match result {
                            Ok(o) if o.status.success() => {
                                let new_entries = list_worktrees(selected_dir);
                                new_entries
                                    .iter()
                                    .find(|e| e.branch.as_deref() == Some(&wt_name))
                                    .map(|e| PathBuf::from(&e.path))
                                    .filter(|p| p.is_dir())
                                    .map_or(WorktreeResult::Cancelled, WorktreeResult::Selected)
                            }
                            _ => WorktreeResult::Cancelled,
                        }
                    }
                    _ => WorktreeResult::Cancelled,
                }
            } else {
                WorktreeResult::Selected(PathBuf::from(id))
            }
        }
        PickerAction::Cancelled => WorktreeResult::Cancelled,
        PickerAction::Custom(..) => WorktreeResult::Cancelled,
    }
}
