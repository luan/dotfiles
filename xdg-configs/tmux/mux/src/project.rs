use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use tracing::warn;

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
pub(crate) fn touch_lru(dir: &str) {
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

pub(crate) fn collect_dirs(filter: &str) -> Vec<PathBuf> {
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

pub(crate) fn build_project_items(filter: &str) -> Vec<PickerItem> {
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

pub(crate) fn repo_display_name(dir: &Path) -> String {
    dir.file_name().map_or(String::new(), |n| {
        n.to_string_lossy()
            .replace(".git", "")
            .trim_start_matches('.')
            .to_string()
    })
}

pub(crate) fn default_session_name(selected_dir: &Path, final_dir: &Path) -> String {
    let is_bare = selected_dir.extension().is_some_and(|e| e == "git") && selected_dir.is_dir();
    let repo_name = if is_bare {
        repo_display_name(selected_dir)
    } else {
        git_toplevel(final_dir.to_str().unwrap_or(""))
            .and_then(|tl| {
                PathBuf::from(tl)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .map(|name| name.trim_start_matches('.').to_string())
            .unwrap_or_default()
    };

    let suffix = repo_display_name(final_dir);

    if !repo_name.is_empty() && repo_name != suffix {
        format!("{repo_name}/{suffix}")
    } else if !repo_name.is_empty() {
        repo_name
    } else {
        suffix
    }
}

pub(crate) fn worktree_name_parts(selected_dir: &Path, branch: Option<&str>) -> (String, String) {
    let repo_name = repo_display_name(selected_dir);
    let default_suffix = branch
        .filter(|b| !b.is_empty() && *b != repo_name)
        .unwrap_or_default()
        .to_string();
    (repo_name, default_suffix)
}

pub(crate) fn create_session_at_dir(session_name: &str, dir: &Path) {
    let exact = format!("={session_name}");
    let dir_str = dir.to_str().unwrap_or(".");
    tmux(&[
        "new-session",
        "-d",
        "-s",
        session_name,
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

pub(crate) fn resolve_selected_dir_from_session(target: Option<&str>) -> Option<PathBuf> {
    let pane_target = target.map(|s| format!("={s}"));

    let pane_path = if let Some(target) = pane_target.as_deref() {
        tmux(&[
            "list-panes",
            "-t",
            target,
            "-F",
            "#{window_active}\t#{pane_active}\t#{pane_current_path}",
        ])
        .lines()
        .find_map(|line| {
            let mut parts = line.splitn(3, '\t');
            match (parts.next(), parts.next(), parts.next()) {
                (Some("1"), Some("1"), Some(path)) => Some(path.to_string()),
                _ => None,
            }
        })
        .unwrap_or_default()
    } else {
        tmux(&["display-message", "-p", "#{pane_current_path}"])
    };

    if pane_path.is_empty() {
        return None;
    }

    let common_dir = Command::new("git")
        .args(["-C", &pane_path, "rev-parse", "--git-common-dir"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    let repo_dir = if Path::new(&common_dir).is_absolute() {
        PathBuf::from(&common_dir)
    } else {
        PathBuf::from(&pane_path).join(&common_dir)
    };

    let is_bare = Command::new("git")
        .args(["-C", &pane_path, "rev-parse", "--is-bare-repository"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| String::from_utf8_lossy(&o.stdout).trim() == "true");

    Some(if is_bare {
        repo_dir
    } else {
        repo_dir.parent().map(Path::to_path_buf).unwrap_or(repo_dir)
    })
}

pub(crate) fn cmd_project_list(args: &[String]) {
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

pub(crate) fn cmd_toggle_favorite(args: &[String]) {
    let Some(raw) = args.first() else {
        return;
    };
    toggle_favorite(raw);
}

pub(crate) fn toggle_favorite(raw: &str) {
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

pub(crate) fn rename_parts(name: &str) -> (String, String) {
    if let Some(slash) = name.find('/') {
        (
            format!("{}/", &name[..slash]),
            name[slash + 1..].to_string(),
        )
    } else {
        (String::new(), name.to_string())
    }
}

pub(crate) fn rename_session(old_name: &str, new_name: &str) -> Result<(), String> {
    if new_name == old_name {
        return Ok(());
    }

    if Command::new("tmux")
        .args(["has-session", "-t", &format!("={new_name}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
    {
        return Err(format!("Session '{new_name}' already exists"));
    }

    tmux(&["rename-session", "-t", &format!("={old_name}"), new_name]);

    let mut store = crate::order::SessionStore::load();
    store.rename(old_name, new_name);
    store.save();
    Ok(())
}

pub(crate) fn cmd_new_session() {
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
        let result = if auto_worktree {
            if let Some(wt_dir) = find_detached_worktree(&entries) {
                final_dir = wt_dir;
                WorktreeResult::NoWorktrees
            } else {
                phase_worktree_picker(entries.clone())
            }
        } else {
            phase_worktree_picker(entries.clone())
        };
        match result {
            WorktreeResult::Selected { path: dir, .. } => final_dir = dir,
            WorktreeResult::NewRequested => match prompt_and_create_worktree(&selected_dir, &entries) {
                Some(dir) => final_dir = dir,
                None => return,
            },
            WorktreeResult::NoWorktrees => {}
            WorktreeResult::Cancelled => return,
        }
    }

    let default_name = default_session_name(&selected_dir, &final_dir);

    // Phase 3: Session name input
    let session_name = match run_text_input(TextInputConfig {
        prompt: "\u{f044}  Session".to_string(),
        initial: default_name.clone(),
        placeholder: "session name...".to_string(),
        prefix: String::new(),
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

    let exact = format!("={session_name}");

    // Check collision
    if Command::new("tmux")
        .args(["has-session", "-t", &format!("={session_name}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
    {
        warn!(name = %session_name, "session already exists");
        tmux(&["switch-client", "-t", &exact]);
        return;
    }

    create_session_at_dir(&session_name, &final_dir);
}

pub(crate) fn cmd_new_worktree(args: &[String]) {
    let target = args.first().filter(|s| !s.is_empty()).cloned();
    let Some(selected_dir) = resolve_selected_dir_from_session(target.as_deref()) else {
        return;
    };

    let entries = list_worktrees(&selected_dir);
    let (existing_dir, repo_name, default_suffix) =
        match phase_worktree_picker(entries.clone()) {
            WorktreeResult::Selected { path, branch } => {
                let (r, s) = worktree_name_parts(&selected_dir, branch.as_deref());
                (Some(path), r, s)
            }
            WorktreeResult::NewRequested => {
                let common_dir = resolve_common_dir(&selected_dir);
                let repo = common_dir
                    .as_deref()
                    .map(|cd| repo_root_name(&selected_dir, cd))
                    .unwrap_or_default();
                let suffix = common_dir
                    .as_deref()
                    .map(|cd| next_wt_suffix(&selected_dir, cd, &entries))
                    .unwrap_or_else(|| "wt1".to_string());
                (None, repo, suffix)
            }
            WorktreeResult::NoWorktrees | WorktreeResult::Cancelled => return,
        };

    // Session name input — only the suffix is editable, prefix is static
    let (session_name, suffix) = if repo_name.is_empty() {
        match run_text_input(TextInputConfig {
            prompt: "\u{f044}  Session".to_string(),
            initial: default_suffix,
            placeholder: "session name...".to_string(),
            prefix: String::new(),
        }) {
            TextInputAction::Confirmed(s) if !s.is_empty() => (s.clone(), s),
            _ => return,
        }
    } else {
        match run_text_input(TextInputConfig {
            prompt: format!("\u{f044}  {repo_name}/"),
            initial: default_suffix,
            placeholder: "session name...".to_string(),
            prefix: String::new(),
        }) {
            TextInputAction::Confirmed(s) if !s.is_empty() => (format!("{repo_name}/{s}"), s),
            _ => return,
        }
    };

    let final_dir = match existing_dir {
        Some(p) => p,
        None => {
            let result = run_with_status(&format!("Creating worktree {suffix}..."), || {
                create_new_worktree(&selected_dir, &suffix)
            });
            match result {
                Ok((path, _)) => path,
                Err(msg) => {
                    run_with_status(&format!("Error: {msg}"), || {
                        std::thread::sleep(Duration::from_secs(3));
                    });
                    return;
                }
            }
        }
    };

    // Check collision
    if Command::new("tmux")
        .args(["has-session", "-t", &format!("={session_name}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
    {
        tmux(&["switch-client", "-t", &format!("={session_name}")]);
        return;
    }

    create_session_at_dir(&session_name, &final_dir);
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

pub(crate) fn build_ditch_plan(session: &str) -> Option<DitchPlan> {
    if session.is_empty() {
        return None;
    }

    let mut body: Vec<ConfirmLine> = Vec::new();
    let raw_dirs = tmux(&[
        "list-panes",
        "-s",
        "-t",
        session,
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
            body.push(ConfirmLine::Info(shorten_home(d)));
        }
        return Some(DitchPlan {
            session: session.to_string(),
            dir: None,
            body,
            actions: vec![ditch_item("kill", "\u{f0513}  Kill session only", TEXT)],
        });
    }

    let dir = PathBuf::from(dirs[0]);
    body.push(ConfirmLine::Ok(format!(
        "All windows in: {}",
        shorten_home(&dir.to_string_lossy())
    )));

    let is_git = git_ok(&dir, &["rev-parse", "--git-dir"]);
    if !is_git {
        body.push(ConfirmLine::Warn("Not a git repository".into()));
        return Some(DitchPlan {
            session: session.to_string(),
            dir: None,
            body,
            actions: vec![ditch_item("kill", "\u{f0513}  Kill session only", TEXT)],
        });
    }
    body.push(ConfirmLine::Ok("Git repository".into()));

    let dirty = !git_ok(&dir, &["diff", "--cached", "--quiet"])
        || !git_ok(&dir, &["diff", "--quiet", "HEAD"]);
    if dirty {
        body.push(ConfirmLine::Error("Uncommitted changes".into()));
        if let Some(status) = git_str(&dir, &["status", "--short"]) {
            for line in status.lines().take(5) {
                body.push(ConfirmLine::Info(line.to_string()));
            }
        }
    } else {
        if let Some(untracked) =
            git_str(&dir, &["ls-files", "--others", "--exclude-standard"]).filter(|s| !s.is_empty())
        {
            body.push(ConfirmLine::Warn("Untracked files (will be kept)".into()));
            for line in untracked.lines().take(5) {
                body.push(ConfirmLine::Info(line.to_string()));
            }
        }
        body.push(ConfirmLine::Ok("No uncommitted changes".into()));
    }

    let branch = git_str(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
    let has_unpushed = branch != "HEAD"
        && git_str(&dir, &["rev-parse", "--abbrev-ref", "@{upstream}"])
            .and_then(|upstream| git_str(&dir, &["log", "--oneline", &format!("{upstream}..HEAD")]))
            .is_some_and(|log| !log.is_empty());
    if has_unpushed {
        body.push(ConfirmLine::Error(format!("Unpushed commits on {branch}")));
    } else {
        body.push(ConfirmLine::Ok("No unpushed changes".into()));
    }

    let common_dir = git_str(&dir, &["rev-parse", "--git-common-dir"]);
    let is_worktree = common_dir
        .as_deref()
        .is_some_and(|c| c != ".git" && c != format!("{}/.git", dir.display()));

    let default_branch = git_str(&dir, &["symbolic-ref", "refs/remotes/origin/HEAD"])
        .and_then(|s| s.strip_prefix("refs/remotes/origin/").map(String::from))
        .unwrap_or_else(|| "main".into());

    let branch_merged = is_worktree
        && branch != "HEAD"
        && branch != default_branch
        && git_ok(
            &dir,
            &["diff", "--quiet", &format!("{default_branch}...HEAD")],
        );

    if branch_merged {
        body.push(ConfirmLine::Ok(format!(
            "Branch adds nothing over {default_branch}"
        )));
    }

    let dir_str = dir.to_str().unwrap_or(".").to_string();
    let green = Color::Rgb(0xa6, 0xe3, 0xa1);
    let red = Color::Rgb(0xf3, 0x8b, 0xa8);
    let overlay = Color::Rgb(0x7f, 0x84, 0x9c);

    let mut actions: Vec<PickerItem> = body
        .iter()
        .map(|item| {
            let (display, color) = match item {
                ConfirmLine::Ok(msg) => (format!("✓ {msg}"), green),
                ConfirmLine::Warn(msg) => (format!("! {msg}"), YELLOW),
                ConfirmLine::Error(msg) => (format!("✗ {msg}"), red),
                ConfirmLine::Info(msg) => (format!("  {msg}"), overlay),
            };
            PickerItem {
                id: String::new(),
                display,
                style: Style::default().fg(color),
                selectable: false,
                color: None,
                dim_color: None,
                right_label: String::new(),
            }
        })
        .collect();

    actions.push(PickerItem {
        id: String::new(),
        display: String::new(),
        style: Style::default(),
        selectable: false,
        color: None,
        dim_color: None,
        right_label: String::new(),
    });

    let safe = !dirty && !has_unpushed;

    if is_worktree {
        // Detach HEAD is always the preferred first action for worktrees —
        // non-destructive: branch and commits survive.
        actions.push(ditch_item(
            "detach",
            "\u{f0e2}  Detach HEAD + kill session",
            CYAN,
        ));
    }

    if is_worktree && branch_merged && safe {
        actions.push(ditch_item(
            "remove_wt",
            "\u{f00d}  Remove worktree + kill session",
            TEXT,
        ));
    }

    if is_worktree && !branch_merged && safe {
        actions.push(ditch_item(
            "remove_wt_keep_branch",
            "\u{f00d}  Remove worktree, keep branch + kill session",
            TEXT,
        ));
    }
    actions.push(ditch_item("kill", "\u{f0513}  Kill session only", TEXT));
    if is_worktree && !branch_merged {
        actions.push(ditch_item(
            "force_remove_wt",
            "\u{f071}  Force remove worktree + delete branch",
            YELLOW,
        ));
    }
    if dirty {
        actions.push(ditch_item(
            "discard_kill",
            "\u{f071}  Discard changes + kill session",
            YELLOW,
        ));
        if is_worktree {
            actions.push(ditch_item(
                "discard_remove_wt",
                "\u{f071}  Discard changes + remove worktree",
                YELLOW,
            ));
        }
    }

    Some(DitchPlan {
        session: session.to_string(),
        dir: Some(dir_str),
        body,
        actions,
    })
}

pub(crate) fn cmd_ditch(args: &[String]) {
    let session = args
        .first()
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_else(|| tmux(&["display-message", "-p", "#S"]));
    let Some(plan) = build_ditch_plan(&session) else {
        return;
    };

    if plan.dir.is_none() {
        let prompt = if matches!(plan.body.first(), Some(ConfirmLine::Error(_))) {
            format!("Kill session '{session}' anyway?")
        } else {
            format!("Kill session '{session}'?")
        };
        if run_confirm(ConfirmConfig {
            body: plan.body.clone(),
            prompt,
        }) {
            let _ = execute_ditch_action(&plan, "kill");
        }
        return;
    }

    let config = PickerConfig {
        prompt: format!("Ditch '{session}'"),
        footer: String::new(),
        placeholder: String::new(),
        initial_id: None,
    };

    let action = run_picker(plan.actions.clone(), config, HashMap::new());
    let id = match action {
        PickerAction::Selected(id) => id,
        _ => return,
    };

    let _ = execute_ditch_action(&plan, &id);
}

pub(crate) fn execute_ditch_action(plan: &DitchPlan, id: &str) -> Result<(), String> {
    let dir_str = plan.dir.as_deref().unwrap_or(".");

    match id {
        "kill" => {}
        "detach" => {
            let _ = Command::new("git")
                .args(["-C", dir_str, "checkout", "--detach"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        "remove_wt" => wt_remove(dir_str, false, false),
        "remove_wt_keep_branch" => wt_remove(dir_str, false, true),
        "force_remove_wt" => wt_remove(dir_str, true, false),
        "discard_kill" => discard_changes(dir_str),
        "discard_remove_wt" => {
            discard_changes(dir_str);
            wt_remove(dir_str, true, false);
        }
        _ => return Err(format!("unknown ditch action: {id}")),
    }

    tmux(&["kill-session", "-t", &format!("={}", plan.session)]);
    Ok(())
}

pub(crate) fn shorten_home(path: &str) -> String {
    let home_str = home().to_string_lossy().to_string();
    path.strip_prefix(&home_str)
        .map(|rest| format!("~{rest}"))
        .unwrap_or_else(|| path.to_string())
}

pub(crate) fn ditch_item(id: &str, display: &str, color: Color) -> PickerItem {
    PickerItem {
        id: id.to_string(),
        display: display.to_string(),
        style: Style::default().fg(color),
        selectable: true,
        color: Some(color),
        dim_color: None,
        right_label: String::new(),
    }
}

fn discard_changes(dir: &str) {
    let _ = Command::new("git")
        .args(["-C", dir, "checkout", "--", "."])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("git")
        .args(["-C", dir, "reset", "HEAD", "--", "."])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn wt_remove(dir: &str, force_delete: bool, keep_branch: bool) {
    let mut args = vec!["-C", dir, "remove", "-y", "--force", "--foreground"];
    if force_delete {
        args.push("-D");
    }
    if keep_branch {
        args.push("--no-delete-branch");
    }
    let _ = Command::new("wt")
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

// ── Directory picker ────────────────────────────────────────────────

fn phase_directory_picker() -> Option<DirectoryPickerResult> {
    let self_bin = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("mux"));
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
    NewRequested,
    NoWorktrees,
    Cancelled,
}

#[derive(Clone)]
pub(crate) struct DitchPlan {
    pub session: String,
    pub dir: Option<String>,
    pub body: Vec<ConfirmLine>,
    pub actions: Vec<PickerItem>,
}

// --- git worktree integration ---

#[derive(Clone)]
pub(crate) struct WtEntry {
    pub path: String,
    pub branch: Option<String>,
    pub detached: bool,
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

pub(crate) fn list_worktrees(dir: &Path) -> Vec<WtEntry> {
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

pub(crate) fn resolve_common_dir(dir: &Path) -> Option<PathBuf> {
    let out = git_output(dir, &["rev-parse", "--git-common-dir"])?;
    let raw = String::from_utf8_lossy(&out).trim().to_string();
    if raw.is_empty() {
        return None;
    }
    let p = PathBuf::from(&raw);
    Some(if p.is_absolute() { p } else { dir.join(p) })
}

pub(crate) fn repo_root_name(selected_dir: &Path, common_dir: &Path) -> String {
    if selected_dir.extension().is_some_and(|e| e == "git") && selected_dir.is_dir() {
        repo_display_name(selected_dir)
    } else {
        common_dir
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}

pub(crate) fn next_wt_suffix(
    selected_dir: &Path,
    common_dir: &Path,
    entries: &[WtEntry],
) -> String {
    let repo = repo_root_name(selected_dir, common_dir);
    let prefix = format!("{repo}.wt");
    let mut max_n = 0u32;
    let mut consider = |name: &str| {
        if let Some(rest) = name.strip_prefix(&prefix)
            && let Ok(n) = rest.parse::<u32>()
        {
            max_n = max_n.max(n);
        }
    };
    for e in entries {
        if let Some(name) = PathBuf::from(&e.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
        {
            consider(&name);
        }
    }
    let parent = worktree_parent_dir(selected_dir, common_dir);
    if let Ok(rd) = fs::read_dir(&parent) {
        for ent in rd.flatten() {
            consider(&ent.file_name().to_string_lossy());
        }
    }
    format!("wt{}", max_n + 1)
}

fn worktree_parent_dir(selected_dir: &Path, common_dir: &Path) -> PathBuf {
    if selected_dir.extension().is_some_and(|e| e == "git") && selected_dir.is_dir() {
        selected_dir
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| selected_dir.to_path_buf())
    } else {
        common_dir
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| selected_dir.to_path_buf())
    }
}

pub(crate) fn find_detached_worktree(entries: &[WtEntry]) -> Option<PathBuf> {
    entries
        .iter()
        .find(|e| e.detached)
        .map(|e| PathBuf::from(&e.path))
        .filter(|p| p.is_dir())
}

pub(crate) fn build_worktree_items(entries: &[WtEntry]) -> Vec<PickerItem> {
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

    for entry in entries {
        let name = PathBuf::from(&entry.path)
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().to_string());
        let branch = entry.branch.as_deref().unwrap_or("");

        let display = if entry.detached {
            format!("{name} (detached)")
        } else if !branch.is_empty() {
            format!("{name} ← {branch}")
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

    items
}

pub(crate) fn prompt_and_create_worktree(
    selected_dir: &Path,
    entries: &[WtEntry],
) -> Option<PathBuf> {
    let common_dir = resolve_common_dir(selected_dir)?;
    let default_suffix = next_wt_suffix(selected_dir, &common_dir, entries);
    let repo = repo_root_name(selected_dir, &common_dir);
    let prompt = if repo.is_empty() {
        "\u{f044}  Worktree".to_string()
    } else {
        format!("\u{f044}  {repo}.")
    };
    let suffix = match run_text_input(TextInputConfig {
        prompt,
        initial: default_suffix,
        placeholder: "worktree name...".to_string(),
        prefix: String::new(),
    }) {
        TextInputAction::Confirmed(s) if !s.is_empty() => s,
        _ => return None,
    };
    let result = run_with_status(&format!("Creating worktree {suffix}..."), || {
        create_new_worktree(selected_dir, &suffix)
    });
    match result {
        Ok((path, _)) => Some(path),
        Err(msg) => {
            run_with_status(&format!("Error: {msg}"), || {
                std::thread::sleep(Duration::from_secs(3));
            });
            None
        }
    }
}

pub(crate) fn create_new_worktree(
    selected_dir: &Path,
    suffix: &str,
) -> Result<(PathBuf, String), String> {
    let Some(common_dir) = resolve_common_dir(selected_dir) else {
        return Err("not a git repo".to_string());
    };
    let repo = repo_root_name(selected_dir, &common_dir);
    let wt_name = if repo.is_empty() {
        suffix.to_string()
    } else {
        format!("{repo}.{suffix}")
    };
    let wt_path = worktree_parent_dir(selected_dir, &common_dir).join(&wt_name);
    let wt_path_str = wt_path.to_string_lossy().to_string();
    let dir_arg = selected_dir.to_str().unwrap_or(".").to_string();
    let output = Command::new("git")
        .args(["-C", &dir_arg, "worktree", "add", "--detach", &wt_path_str])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if msg.is_empty() {
            format!("git exited with {}", output.status)
        } else {
            msg
        });
    }
    if !wt_path.is_dir() {
        return Err("worktree was not created".to_string());
    }
    Ok((wt_path, wt_name))
}

fn phase_worktree_picker(entries: Vec<WtEntry>) -> WorktreeResult {
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
                WorktreeResult::NewRequested
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
