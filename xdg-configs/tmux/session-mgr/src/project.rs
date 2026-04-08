use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use crate::order::{load_lines, save_lines};
use crate::picker::{
    ConfirmConfig, ConfirmLine, PickerAction, PickerConfig, PickerItem, TextInputAction,
    TextInputConfig, run_confirm, run_picker, run_text_input, run_with_status,
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
            for name in ["dotfiles", ".claude", "blueprints"] {
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
            for name in ["dotfiles", ".claude", "blueprints"] {
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
                    WorktreeResult::Selected { path: dir, .. } => final_dir = dir,
                    WorktreeResult::NoWorktrees => {}
                    WorktreeResult::Cancelled => return,
                }
            }
        } else {
            match phase_worktree_picker(&selected_dir, entries) {
                WorktreeResult::Selected { path: dir, .. } => final_dir = dir,
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
        placeholder: "session name...".to_string(),
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

pub fn cmd_new_worktree() {
    // Get current pane path
    let pane_path = tmux(&["display-message", "-p", "#{pane_current_path}"]);
    if pane_path.is_empty() {
        return;
    }

    // Find the bare repo root: git-common-dir returns the shared .git dir
    let common_dir = Command::new("git")
        .args(["-C", &pane_path, "rev-parse", "--git-common-dir"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let Some(common) = common_dir else { return };

    // Resolve to absolute path
    let repo_dir = if Path::new(&common).is_absolute() {
        PathBuf::from(&common)
    } else {
        PathBuf::from(&pane_path).join(&common)
    };

    // For bare repos, git-common-dir is the repo root itself.
    // For non-bare repos, git-common-dir is ".git" — parent is the repo root.
    let is_bare = Command::new("git")
        .args(["-C", &pane_path, "rev-parse", "--is-bare-repository"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| String::from_utf8_lossy(&o.stdout).trim() == "true");

    let selected_dir = if is_bare {
        repo_dir
    } else {
        repo_dir.parent().map(Path::to_path_buf).unwrap_or(repo_dir)
    };

    let entries = list_worktrees(&selected_dir);
    let (final_dir, wt_branch) = match phase_worktree_picker(&selected_dir, entries) {
        WorktreeResult::Selected { path, branch } => (path, branch),
        WorktreeResult::NoWorktrees | WorktreeResult::Cancelled => return,
    };

    // Compute session name: repo_name is fixed prefix, suffix from branch
    let repo_name = selected_dir.file_name().map_or(String::new(), |n| {
        n.to_string_lossy()
            .replace(".git", "")
            .trim_start_matches('.')
            .to_string()
    });

    // Prefer branch name for suffix (avoids path-based names like "dotfiles.idk")
    let default_suffix = wt_branch
        .filter(|b| !b.is_empty() && *b != repo_name)
        .unwrap_or_default();

    // Session name input — only the suffix is editable, prefix is static
    let session_name = if repo_name.is_empty() {
        match run_text_input(TextInputConfig {
            prompt: "\u{f044}  Session".to_string(),
            initial: default_suffix,
            placeholder: "session name...".to_string(),
        }) {
            TextInputAction::Confirmed(s) if !s.is_empty() => s,
            _ => return,
        }
    } else {
        match run_text_input(TextInputConfig {
            prompt: format!("\u{f044}  {repo_name}/"),
            initial: default_suffix,
            placeholder: "branch name...".to_string(),
        }) {
            TextInputAction::Confirmed(s) if !s.is_empty() => format!("{repo_name}/{s}"),
            _ => return,
        }
    };

    // Check collision
    if Command::new("tmux")
        .args(["has-session", "-t", &format!("={session_name}")])
        .status()
        .is_ok_and(|s| s.success())
    {
        tmux(&["switch-client", "-t", &session_name]);
        return;
    }

    // Create session with 3 windows: ai, vi, sh
    // Use "=" prefix on -t to force exact session name match — without it,
    // tmux parses "/" in the name as a session/window separator.
    let dir_str = final_dir.to_str().unwrap_or(".");
    let exact = format!("={session_name}");
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
        &exact,
        "-n",
        "vi",
        "-c",
        dir_str,
        ";",
        "new-window",
        "-t",
        &exact,
        "-n",
        "sh",
        "-c",
        dir_str,
        ";",
        "select-window",
        "-t",
        &format!("={session_name}:ai"),
        ";",
        "switch-client",
        "-t",
        &exact,
    ]);
}

// ── Ditch session ───────────────────────────────────────────────────

fn git_str(dir: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn git_ok(dir: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub fn cmd_ditch() {
    let session = tmux(&["display-message", "-p", "#S"]);
    if session.is_empty() {
        return;
    }

    let mut body: Vec<ConfirmLine> = Vec::new();

    // All pane dirs must be the same
    let raw_dirs = tmux(&[
        "list-panes",
        "-s",
        "-t",
        &session,
        "-F",
        "#{pane_current_path}",
    ]);
    let dirs: Vec<&str> = raw_dirs
        .lines()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if dirs.len() != 1 {
        body.push(ConfirmLine::Error(
            "Windows are in different directories".into(),
        ));
        for d in &dirs {
            body.push(ConfirmLine::Info(
                d.strip_prefix(&home().to_string_lossy().to_string())
                    .map(|rest| format!("~{rest}"))
                    .unwrap_or_else(|| d.to_string()),
            ));
        }
        run_confirm(ConfirmConfig {
            body,
            prompt: "Cannot ditch — press any key".into(),
        });
        return;
    }

    let dir = PathBuf::from(dirs[0]);
    let home_str = home().to_string_lossy().to_string();
    let short_dir = dir
        .to_string_lossy()
        .strip_prefix(&home_str)
        .map(|rest| format!("~{rest}"))
        .unwrap_or_else(|| dir.to_string_lossy().to_string());
    body.push(ConfirmLine::Ok(format!("All windows in: {short_dir}")));

    // Must be a git repo
    if !git_ok(&dir, &["rev-parse", "--git-dir"]) {
        body.push(ConfirmLine::Error("Not a git repository".into()));
        run_confirm(ConfirmConfig {
            body,
            prompt: "Cannot ditch — press any key".into(),
        });
        return;
    }
    body.push(ConfirmLine::Ok("Git repository".into()));

    // Check uncommitted changes
    let has_staged = !git_ok(&dir, &["diff", "--cached", "--quiet"]);
    let has_unstaged = !git_ok(&dir, &["diff", "--quiet", "HEAD"]);
    if has_staged || has_unstaged {
        body.push(ConfirmLine::Error("Uncommitted changes".into()));
        if let Some(status) = git_str(&dir, &["status", "--short"]) {
            for line in status.lines().take(5) {
                body.push(ConfirmLine::Info(line.to_string()));
            }
        }
        run_confirm(ConfirmConfig {
            body,
            prompt: "Cannot ditch — press any key".into(),
        });
        return;
    }
    // Warn about untracked files (non-blocking)
    if let Some(untracked) =
        git_str(&dir, &["ls-files", "--others", "--exclude-standard"]).filter(|s| !s.is_empty())
    {
        body.push(ConfirmLine::Warn("Untracked files (will be kept)".into()));
        for line in untracked.lines().take(5) {
            body.push(ConfirmLine::Info(line.to_string()));
        }
    }

    body.push(ConfirmLine::Ok("No uncommitted changes".into()));

    // Check unpushed commits
    let branch = git_str(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
    if branch != "HEAD"
        && let Some(upstream) = git_str(&dir, &["rev-parse", "--abbrev-ref", "@{upstream}"])
        && let Some(unpushed) = git_str(&dir, &["log", "--oneline", &format!("{upstream}..HEAD")])
        && !unpushed.is_empty()
    {
        body.push(ConfirmLine::Error(format!("Unpushed commits on {branch}")));
        for line in unpushed.lines().take(5) {
            body.push(ConfirmLine::Info(line.to_string()));
        }
        run_confirm(ConfirmConfig {
            body,
            prompt: "Cannot ditch — press any key".into(),
        });
        return;
    }
    body.push(ConfirmLine::Ok("No unpushed changes".into()));

    // Detect worktree
    let common_dir = git_str(&dir, &["rev-parse", "--git-common-dir"]);
    let is_worktree = common_dir
        .as_deref()
        .is_some_and(|c| c != ".git" && c != format!("{}/.git", dir.display()));

    if is_worktree {
        let is_bare =
            git_str(&dir, &["rev-parse", "--is-bare-repository"]).as_deref() == Some("true");

        // Detect default branch
        let default_branch = git_str(&dir, &["symbolic-ref", "refs/remotes/origin/HEAD"])
            .and_then(|s| s.strip_prefix("refs/remotes/origin/").map(String::from))
            .unwrap_or_else(|| "main".into());

        // Check if branch is merged / adds nothing
        let branch_merged = branch != "HEAD"
            && branch != default_branch
            && git_ok(
                &dir,
                &["diff", "--quiet", &format!("{default_branch}...HEAD")],
            );

        if branch_merged {
            body.push(ConfirmLine::Ok(format!(
                "Branch adds nothing over {default_branch} — safe to remove"
            )));
            if !run_confirm(ConfirmConfig {
                body,
                prompt: format!("Remove worktree '{branch}' and kill session '{session}'?"),
            }) {
                return;
            }
            // wt remove handles worktree + branch cleanup
            let _ = Command::new("wt")
                .args([
                    "remove",
                    &branch,
                    "-y",
                    "--force",
                    "--foreground",
                    "-C",
                    dir.to_str().unwrap_or("."),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        } else {
            let skip_detach = !is_bare && branch == "main";
            if skip_detach {
                if !run_confirm(ConfirmConfig {
                    body,
                    prompt: format!("Kill session '{session}'?"),
                }) {
                    return;
                }
            } else {
                if !run_confirm(ConfirmConfig {
                    body,
                    prompt: format!("Detach HEAD and kill session '{session}'?"),
                }) {
                    return;
                }
                let _ = Command::new("git")
                    .args(["-C", dir.to_str().unwrap_or("."), "checkout", "--detach"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
    } else if !run_confirm(ConfirmConfig {
        body,
        prompt: format!("Kill session '{session}'?"),
    }) {
        return;
    }

    tmux(&["kill-session", "-t", &format!("={session}")]);
}

// ── Directory picker ────────────────────────────────────────────────

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
    Selected {
        path: PathBuf,
        branch: Option<String>,
    },
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
                    placeholder: "branch name...".to_string(),
                }) {
                    TextInputAction::Confirmed(wt_name) if !wt_name.is_empty() => {
                        let dir_arg = selected_dir.to_str().unwrap_or(".").to_string();
                        let branch = wt_name.clone();
                        let result: Result<(), String> =
                            run_with_status(&format!("Creating worktree {wt_name}..."), || {
                                let output = Command::new("wt")
                                    .args([
                                        "switch", "--create", &branch, "--no-cd", "-y", "-C",
                                        &dir_arg,
                                    ])
                                    .stdin(Stdio::null())
                                    .stdout(Stdio::null())
                                    .output();
                                match output {
                                    Ok(o) if o.status.success() => Ok(()),
                                    Ok(o) => {
                                        let msg =
                                            String::from_utf8_lossy(&o.stderr).trim().to_string();
                                        Err(if msg.is_empty() {
                                            format!("wt exited with {}", o.status)
                                        } else {
                                            msg
                                        })
                                    }
                                    Err(e) => Err(format!("failed to run wt: {e}")),
                                }
                            });
                        match result {
                            Ok(()) => {
                                let new_entries = list_worktrees(selected_dir);
                                // Match by branch name, fall back to directory name
                                let found = new_entries
                                    .iter()
                                    .find(|e| e.branch.as_deref() == Some(&wt_name))
                                    .or_else(|| {
                                        new_entries.iter().find(|e| {
                                            PathBuf::from(&e.path)
                                                .file_name()
                                                .is_some_and(|n| n.to_string_lossy() == wt_name)
                                        })
                                    });
                                match found {
                                    Some(e) if PathBuf::from(&e.path).is_dir() => {
                                        WorktreeResult::Selected {
                                            path: PathBuf::from(&e.path),
                                            branch: e.branch.clone(),
                                        }
                                    }
                                    _ => WorktreeResult::Cancelled,
                                }
                            }
                            Err(msg) => {
                                run_with_status(&format!("Error: {msg}"), || {
                                    std::thread::sleep(Duration::from_secs(3));
                                });
                                WorktreeResult::Cancelled
                            }
                        }
                    }
                    _ => WorktreeResult::Cancelled,
                }
            } else {
                // Look up the branch from the original entries
                let branch = entries
                    .iter()
                    .find(|e| e.path == id)
                    .and_then(|e| e.branch.clone());
                WorktreeResult::Selected {
                    path: PathBuf::from(id),
                    branch,
                }
            }
        }
        PickerAction::Cancelled => WorktreeResult::Cancelled,
        PickerAction::Custom(..) => WorktreeResult::Cancelled,
    }
}
