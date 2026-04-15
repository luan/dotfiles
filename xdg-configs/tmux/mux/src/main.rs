use std::collections::HashSet;
use std::env;
use std::process::{Command, Stdio};

use serde::Deserialize;
use tracing::{debug, error};

mod chooser;
mod color;
mod filter;
mod group;
mod logging;
mod mru;
mod order;
mod palette;
mod picker;
mod process;
mod project;
mod sidebar;
mod status;
mod tmux;
mod usage;
mod usage_bars;

use color::compute_color;
use group::GroupMeta;
use order::compute_order;
use picker::{TextInputAction, TextInputConfig, run_text_input};
use project::{rename_parts, rename_session};
use status::{render_bar, render_windows};
use tmux::{
    query_state, query_system_info, query_system_info_with, query_windows, tmux as tmux_cmd,
};

#[derive(Deserialize)]
struct WeztermPane {
    window_id: u64,
    tab_id: u64,
    pane_id: u64,
    title: String,
    is_active: bool,
    left_col: usize,
}

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
    let current = args
        .first()
        .filter(|s| !s.is_empty())
        .map_or(&st.current, |s| s);
    let client_width: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200);

    let sessions = compute_order(&st.alive, false);
    let meta = GroupMeta::new(&sessions);

    let pre_colors: Vec<(String, String)> = {
        let c = status::compute_all_colors(&sessions, &meta);
        c.iter()
            .map(|(n, col, _)| (n.clone(), col.clone()))
            .collect()
    };
    let cur_color = pre_colors
        .iter()
        .find(|(n, _)| n == current)
        .map_or("#FFFFFF", |(_, c)| c.as_str());

    let bar = render_bar(&sessions, current, &meta, &st.attn, client_width);
    let windows = query_windows(current);
    let win_str = render_windows(&windows, cur_color);

    // If the sidebar is open, hide the session list in the status bar
    let sidebar_open = tmux_cmd(&["show-option", "-gv", "@sidebar_open"]) == "1";
    let left = if sidebar_open { "" } else { bar.left.as_str() };

    // When the terminal is fullscreened on a notched MacBook display, paint
    // the status-bar background solid black so the notch area reads as an
    // intentional gap instead of broken chrome. The two-row stacking (blank
    // row above real content) is separately gated by `@two_row_status` so
    // users can keep the notched black-bg single-row layout.
    let notched = tmux_cmd(&["show-option", "-gv", "@notched"]) == "1";
    let two_row = notched && tmux_cmd(&["show-option", "-gv", "@two_row_status"]) != "0";
    let status_style = if notched { "bg=#000000" } else { "bg=default" };

    // Build status-format[0]: normally sessions=left, windows=centre,
    // system-info=right. When notched, the centre is behind the display cutout
    // so windows shift to the left segment (after sessions).
    let status_fmt = if notched {
        format!(
            "#[bg=#000000]#[align=left]{left}{win}#[align=right]#{{@sysinfo}}",
            left = left,
            win = win_str,
        )
    } else {
        format!(
            "#[align=left]{left}#[align=centre]{win}#[align=right]#{{@sysinfo}}",
            left = left,
            win = win_str,
        )
    };

    let mut tmux_args: Vec<String> = vec![
        "set-option".into(),
        "-t".into(),
        current.clone(),
        "-u".into(),
        "@attention".into(),
    ];
    for (name, color) in &bar.colors {
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
        "status-style".into(),
        status_style.into(),
    ]);
    // On notched displays, stack a blank black row above the real content so
    // the notch sits in dedicated empty space instead of eating chrome.
    if two_row {
        tmux_args.extend([
            ";".into(),
            "set".into(),
            "-g".into(),
            "status".into(),
            "2".into(),
        ]);
        tmux_args.extend([
            ";".into(),
            "set".into(),
            "-g".into(),
            "status-format[0]".into(),
            String::new(),
        ]);
        tmux_args.extend([
            ";".into(),
            "set".into(),
            "-g".into(),
            "status-format[1]".into(),
            status_fmt,
        ]);
    } else {
        tmux_args.extend([
            ";".into(),
            "set".into(),
            "-g".into(),
            "status".into(),
            "on".into(),
        ]);
        tmux_args.extend([
            ";".into(),
            "set".into(),
            "-gu".into(),
            "status-format[1]".into(),
        ]);
        tmux_args.extend([
            ";".into(),
            "set".into(),
            "-g".into(),
            "status-format[0]".into(),
            status_fmt,
        ]);
    }
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
    let bar = render_bar(&sessions, &st.current, &meta, &st.attn, 200);
    print!("{}", bar.left);

    if !bar.colors.is_empty() {
        let mut args: Vec<String> = Vec::new();
        for (i, (name, color)) in bar.colors.iter().enumerate() {
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

fn cmd_click(args: &[String]) {
    let range = args.first().map_or("", String::as_str);
    if let Some(session) = range.strip_prefix("s:") {
        tmux_cmd(&["switch-client", "-t", session]);
    } else if let Some(window) = range.strip_prefix("w:") {
        tmux_cmd(&["select-window", "-t", &format!(":{window}")]);
    } else if range == "caffeine" {
        toggle_caffeine();
    }
}

fn toggle_caffeine() {
    let pids = tmux::indefinite_caffeinate_pids();
    if !pids.is_empty() {
        for pid in pids {
            let _ = Command::new("kill").arg(&pid).status();
        }
    } else {
        let _ = Command::new("caffeinate")
            .args(["-di"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
    // Re-render @sysinfo immediately so the icon flips without waiting for
    // the 5s sysinfo-daemon tick. A short delay gives the spawned caffeinate
    // process time to register with pgrep on the kill→spawn path.
    std::thread::sleep(std::time::Duration::from_millis(50));
    let info = query_system_info();
    let rendered = status::render_system_info(&info);
    tmux_cmd(&[
        "set-option",
        "-g",
        "@sysinfo",
        &rendered,
        ";",
        "refresh-client",
        "-S",
    ]);
}

fn cmd_system_info() {
    let system = query_system_info();
    print!("{}", status::render_system_info(&system));
}

fn cmd_sysinfo_daemon() {
    let pid_file = std::env::temp_dir().join("tmux-sysinfo.pid");

    // Deduplication: bail if another instance is already running
    if let Ok(contents) = std::fs::read_to_string(&pid_file)
        && let Ok(pid) = contents.trim().parse::<u32>()
    {
        let alive = Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if alive {
            return;
        }
    }
    let _ = std::fs::write(&pid_file, std::process::id().to_string());

    // Reuse one System instance across all ticks — avoids per-tick allocation
    let mut sys = sysinfo::System::new();

    // Initial render before entering the loop
    let mut cached = query_system_info_with(&mut sys);
    let rendered = status::render_system_info(&cached);
    tmux_cmd(&[
        "set-option",
        "-g",
        "@sysinfo",
        &rendered,
        ";",
        "refresh-client",
        "-S",
    ]);

    let mut tick = 0u64;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        tick += 1;

        let now = chrono::Local::now();
        cached.date = now.format("%a %b %-d").to_string();
        cached.clock = now.format("%H:%M:%S").to_string();

        // Refresh expensive fields every 5 seconds
        if tick.is_multiple_of(5) {
            let fresh = query_system_info_with(&mut sys);
            cached.cpu_load = fresh.cpu_load;
            cached.mem_pct = fresh.mem_pct;
            cached.battery_pct = fresh.battery_pct;
            cached.battery_state = fresh.battery_state;
            cached.battery_time = fresh.battery_time;
            cached.caffeinated = fresh.caffeinated;
        }

        let rendered = status::render_system_info(&cached);
        tmux_cmd(&[
            "set-option",
            "-g",
            "@sysinfo",
            &rendered,
            ";",
            "refresh-client",
            "-S",
        ]);
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
        (None, "prev") => sessions.last().expect("non-empty after early return"),
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
    process::spawn_detached_update();
}

fn cmd_rename(args: &[String]) {
    let old_name = args
        .first()
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_else(|| tmux_cmd(&["display-message", "-p", "#S"]));

    let (prefix, suffix) = rename_parts(&old_name);

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
    let _ = rename_session(&old_name, &new_name);
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

fn cmd_picker(args: &[String]) {
    let Some(action) = args.first().map(String::as_str) else {
        error!("missing picker action");
        std::process::exit(1);
    };

    if tmux_cmd(&["show-option", "-gv", "@sidebar_open"]) == "1" && dispatch_sidebar_picker(action)
    {
        return;
    }

    open_picker_popup(action);
}

fn dispatch_sidebar_picker(action: &str) -> bool {
    let key = match action {
        "rename" => "r",
        "chooser" => "/",
        "new-session" => "n",
        "new-worktree" => "w",
        "ditch" => "x",
        _ => return false,
    };

    let Ok(output) = Command::new("wezterm")
        .args(["cli", "list", "--format", "json"])
        .stderr(Stdio::null())
        .output()
    else {
        return false;
    };

    if !output.status.success() {
        return false;
    }

    let Ok(panes) = serde_json::from_slice::<Vec<WeztermPane>>(&output.stdout) else {
        return false;
    };

    let Some(active) = panes.iter().find(|pane| pane.is_active) else {
        return false;
    };

    let Some(sidebar) = panes
        .iter()
        .filter(|pane| {
            pane.window_id == active.window_id
                && pane.tab_id == active.tab_id
                && pane.pane_id != active.pane_id
                && pane.left_col < active.left_col
                && pane.title == "mux"
        })
        .min_by_key(|pane| pane.left_col)
    else {
        return false;
    };

    let pane_id = sidebar.pane_id.to_string();

    if !Command::new("wezterm")
        .args(["cli", "activate-pane", "--pane-id", &pane_id])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
    {
        return false;
    }

    Command::new("wezterm")
        .args(["cli", "send-text", "--no-paste", "--pane-id", &pane_id, key])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn open_picker_popup(action: &str) {
    let (width, height, allow_failure) = match action {
        "rename" => ("60%", "6", true),
        "chooser" => ("80%", "40%", false),
        "new-session" => ("80%", "60%", true),
        "new-worktree" => ("80%", "60%", true),
        "ditch" => ("80%", "50%", true),
        _ => return,
    };

    let mut command = format!("~/.config/tmux/scripts/mux {action}");
    if allow_failure {
        command.push_str(" || true");
    }

    let _ = Command::new("tmux")
        .args([
            "display-popup",
            "-E",
            "-B",
            "-w",
            width,
            "-h",
            height,
            "-x",
            "C",
            "-y",
            "C",
            "-s",
            "bg=#{@popup_bg}",
            &command,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn cmd_bench(args: &[String]) {
    let iterations: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(20);

    // Bench usage_bars::collect()
    let mut collect_times = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let t0 = std::time::Instant::now();
        let _ = usage_bars::collect();
        collect_times.push(t0.elapsed());
    }

    // Bench a full sidebar refresh cycle (the core of what refresh() does)
    let alive: std::collections::HashSet<String> = tmux_cmd(&["list-sessions", "-F", "#S"])
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    let sessions = compute_order(&alive, false);

    let mut refresh_times = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let t0 = std::time::Instant::now();
        // Simulate the refresh hot path: batch tmux query + meta + usage_bars
        let _batch = tmux_cmd(&[
            "show-option",
            "-gv",
            "@notched",
            ";",
            "display-message",
            "-p",
            "#S",
            ";",
            "list-sessions",
            "-F",
            "#S",
        ]);
        sidebar::bench_query_session_meta(&sessions);
        let _ = usage_bars::collect();
        refresh_times.push(t0.elapsed());
    }

    let stats = |times: &[std::time::Duration]| -> (u128, u128, u128) {
        let min = times.iter().map(|d| d.as_micros()).min().unwrap_or(0);
        let max = times.iter().map(|d| d.as_micros()).max().unwrap_or(0);
        let avg = times.iter().map(|d| d.as_micros()).sum::<u128>() / times.len().max(1) as u128;
        (min, avg, max)
    };

    // Bench sysinfo daemon tick (query_system_info_with reusing a System instance)
    let mut sys = sysinfo::System::new();
    // Warm up the System instance (first call populates internal caches)
    let _ = query_system_info_with(&mut sys);
    let mut sysinfo_times = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let t0 = std::time::Instant::now();
        let _ = query_system_info_with(&mut sys);
        sysinfo_times.push(t0.elapsed());
    }

    let (cmin, cavg, cmax) = stats(&collect_times);
    let (rmin, ravg, rmax) = stats(&refresh_times);
    let (smin, savg, smax) = stats(&sysinfo_times);

    println!("usage_bars::collect()  n={iterations}  min={cmin}µs  avg={cavg}µs  max={cmax}µs");
    println!("refresh() simulation   n={iterations}  min={rmin}µs  avg={ravg}µs  max={rmax}µs");
    println!("sysinfo daemon tick    n={iterations}  min={smin}µs  avg={savg}µs  max={smax}µs");
}

fn main() {
    let _guard = logging::init_tracing();
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map_or("list", String::as_str);
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    debug!(subcommand = %cmd, args = ?rest, "dispatch");
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
        "new-worktree" => project::cmd_new_worktree(&rest),
        "ditch" => project::cmd_ditch(&rest),
        "rename" => cmd_rename(&rest),
        "select" => cmd_select(&rest),
        "attention" => cmd_attention(),
        "hide-toggle" => cmd_hide_toggle(&rest),
        "picker" => cmd_picker(&rest),
        "update" => cmd_update_with_args(&rest),
        "click" => cmd_click(&rest),
        "sidebar" => sidebar::cmd_sidebar(),
        "mru-cycle" => mru::cmd_mru_cycle(&rest),
        "mru-overlay" => mru::cmd_mru_overlay(),
        "system-info" => cmd_system_info(),
        "sysinfo-daemon" => cmd_sysinfo_daemon(),
        "log" => logging::cmd_log(&rest),
        "bench" => cmd_bench(&rest),
        _ => {
            error!(cmd = %cmd, "unknown subcommand");
            std::process::exit(1);
        }
    }
}
