use std::collections::HashSet;
use std::env;
use std::process::{Command, Stdio};

mod chooser;
mod color;
mod group;
mod order;
mod picker;
mod project;
mod status;
mod tmux;

use color::compute_color;
use group::GroupMeta;
use order::{SessionStore, compute_order};
use picker::{TextInputAction, TextInputConfig, run_text_input};
use status::render_status;
use tmux::{query_state, tmux as tmux_cmd};

fn cmd_order(args: &[String]) {
    let include_all = args.iter().any(|a| a == "--all");
    let alive: HashSet<String> = tmux_cmd(&["list-sessions", "-F", "#S"])
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    for s in compute_order(&alive, include_all) {
        println!("{s}");
    }
}

fn cmd_update_with_args(args: &[String]) {
    let st = query_state();
    // Use explicit session name if provided (from hook), otherwise fall back to query
    let current = args
        .first()
        .filter(|s| !s.is_empty())
        .map_or(&st.current, |s| s);
    let client_width: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200);
    // CPU+battery modules hide below 100 cols, so reduce the right-side budget
    let right_budget = if client_width < 100 { 4 } else { 40 };
    let available_width = client_width.saturating_sub(right_budget).max(20);

    let sessions = compute_order(&st.alive, false);
    let meta = GroupMeta::new(&sessions);
    let (status, colors) = render_status(&sessions, current, &meta, &st.attn, available_width);
    let cur_color = colors
        .iter()
        .find(|(n, _)| n == current)
        .map_or("#FFFFFF", |(_, c)| c.as_str());

    let mut tmux_args: Vec<String> = vec![
        "set-option".into(),
        "-t".into(),
        current.clone(),
        "-u".into(),
        "@attention".into(),
    ];
    for (name, color) in &colors {
        tmux_args.extend([
            ";".into(),
            "set-option".into(),
            "-t".into(),
            name.clone(),
            "@session_color".into(),
            color.clone(),
        ]);
    }
    tmux_args.extend([
        ";".into(),
        "set".into(),
        "-t".into(),
        current.clone(),
        "@session_color".into(),
        cur_color.into(),
    ]);
    tmux_args.extend([
        ";".into(),
        "set".into(),
        "-g".into(),
        "status-left-length".into(),
        available_width.to_string(),
    ]);
    tmux_args.extend([
        ";".into(),
        "set".into(),
        "-g".into(),
        "status-left".into(),
        format!(" {status} "),
    ]);
    tmux_args.extend([";".into(), "refresh-client".into(), "-S".into()]);

    let refs: Vec<&str> = tmux_args.iter().map(String::as_str).collect();
    tmux_cmd(&refs);

    let _ = Command::new("grrr")
        .args(["clear", &format!("claude-{current}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn cmd_list() {
    let st = query_state();
    let sessions = compute_order(&st.alive, false);
    let meta = GroupMeta::new(&sessions);
    let (status, colors) = render_status(&sessions, &st.current, &meta, &st.attn, 200);
    print!("{status}");

    if !colors.is_empty() {
        let mut args: Vec<String> = Vec::new();
        for (i, (name, color)) in colors.iter().enumerate() {
            if i > 0 {
                args.push(";".into());
            }
            args.extend([
                "set-option".into(),
                "-t".into(),
                name.clone(),
                "@session_color".into(),
                color.clone(),
            ]);
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        tmux_cmd(&refs);
    }
}

fn cmd_color(args: &[String]) {
    let mut mode = "color";
    let (mut pos, mut total, mut gpos, mut gtotal) = (0, 0, 0, 0);
    let mut i = 0;
    while i < args.len().saturating_sub(1) {
        match args[i].as_str() {
            "--dim" => {
                mode = "dim";
                i += 1;
            }
            "--both" => {
                mode = "both";
                i += 1;
            }
            "--pos" => {
                pos = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            "--total" => {
                total = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            "--group-pos" => {
                gpos = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            "--group-total" => {
                gtotal = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            _ => break,
        }
    }
    let name = args.last().map_or("", String::as_str);
    let (c, d) = compute_color(name, pos, total, gpos, gtotal);
    match mode {
        "dim" => println!("{d}"),
        "both" => println!("{c}\t{d}"),
        _ => println!("{c}"),
    }
}

fn cmd_switch(args: &[String]) {
    let direction = args.first().map_or("next", String::as_str);
    let st = query_state();
    let sessions = compute_order(&st.alive, false);
    if sessions.is_empty() {
        return;
    }
    let idx = sessions.iter().position(|s| s == &st.current);
    let target = match (idx, direction) {
        (Some(i), "prev") => &sessions[(i + sessions.len() - 1) % sessions.len()],
        (Some(i), _) => &sessions[(i + 1) % sessions.len()],
        (None, "prev") => sessions.last().unwrap(),
        (None, _) => &sessions[0],
    };
    tmux_cmd(&["switch-client", "-t", target]);
}

fn cmd_move(args: &[String]) {
    let direction = args.first().map_or("", String::as_str);
    let st = query_state();
    let current = if args.len() > 1 {
        args[1].clone()
    } else {
        st.current.clone()
    };

    let mut store = order::SessionStore::load();
    store.prune(&st.alive);
    if store.move_session(&current, direction) {
        store.save();
    }
    // Fork the status-bar refresh into background so tmux unblocks immediately
    // (allows rapid repeated moves without losing keypresses to serialization)
    let exe = env::current_exe().unwrap_or_else(|_| "tmux-session".into());
    let _ = Command::new(exe)
        .args(["update"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn cmd_rename() {
    let old_name = tmux_cmd(&["display-message", "-p", "#S"]);

    // Split into fixed prefix (repo/) and editable suffix
    let (prefix, suffix) = if let Some(slash) = old_name.find('/') {
        (
            format!("{}/", &old_name[..slash]),
            old_name[slash + 1..].to_string(),
        )
    } else {
        (String::new(), old_name.clone())
    };

    let new_suffix = match run_text_input(TextInputConfig {
        prompt: "\u{f044}  Rename".to_string(),
        initial: suffix.clone(),
        placeholder: "session name...".to_string(),
        prefix: prefix.clone(),
    }) {
        TextInputAction::Confirmed(s) => s.trim().to_string(),
        TextInputAction::Cancelled => return,
    };

    if new_suffix.is_empty() {
        return;
    }

    let new_name = format!("{prefix}{new_suffix}");
    if new_name == old_name {
        return;
    }

    // Check for collision
    if Command::new("tmux")
        .args(["has-session", "-t", &format!("={new_name}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
    {
        eprintln!("Session '{new_name}' already exists");
        return;
    }

    tmux_cmd(&["rename-session", "-t", &format!("={old_name}"), &new_name]);

    // Update order store
    let mut store = SessionStore::load();
    store.rename(&old_name, &new_name);
    store.save();
}

fn cmd_select(args: &[String]) {
    let index: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    if index == 0 {
        return;
    }
    let st = query_state();
    let sessions = compute_order(&st.alive, false);
    if let Some(target) = sessions.get(index - 1) {
        tmux_cmd(&["switch-client", "-t", target]);
    }
}

fn cmd_attention() {
    let st = query_state();
    if let Some(target) = st
        .attn
        .iter()
        .find(|(_, v)| *v == "1")
        .map(|(k, _)| k.as_str())
    {
        tmux_cmd(&["switch-client", "-t", target]);
    }
}

fn cmd_hide_toggle(args: &[String]) {
    let session = match args.first() {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return,
    };
    let path = order::hidden_file();
    let mut lines = order::load_lines(&path);
    if let Some(pos) = lines.iter().position(|l| l == &session) {
        lines.remove(pos);
    } else {
        lines.push(session);
    }
    order::save_lines(&path, &lines);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map_or("list", String::as_str);
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    match cmd {
        "order" => cmd_order(&rest),
        "list" => cmd_list(),
        "color" => cmd_color(&rest),
        "switch" => cmd_switch(&rest),
        "move" => cmd_move(&rest),
        "chooser-list" => chooser::cmd_chooser_list(),
        "chooser" => chooser::cmd_chooser(),
        "project-list" => project::cmd_project_list(&rest),
        "toggle-favorite" => project::cmd_toggle_favorite(&rest),
        "new-session" => project::cmd_new_session(),
        "new-worktree" => project::cmd_new_worktree(),
        "ditch" => project::cmd_ditch(),
        "rename" => cmd_rename(),
        "select" => cmd_select(&rest),
        "attention" => cmd_attention(),
        "hide-toggle" => cmd_hide_toggle(&rest),
        "update" => cmd_update_with_args(&rest),
        _ => {
            eprintln!("Unknown: {cmd}");
            std::process::exit(1);
        }
    }
}
