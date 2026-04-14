use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use sysinfo::System;
use tracing::error;

pub(crate) fn home() -> PathBuf {
    match env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => {
            error!("HOME env var missing, falling back to /");
            PathBuf::from("/")
        }
    }
}

pub(crate) fn tmux(args: &[&str]) -> String {
    let output = match Command::new("tmux")
        .args(args)
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            error!(args = ?args, error = %e, "failed to spawn tmux");
            return String::new();
        }
    };
    if !output.status.success() {
        error!(
            args = ?args,
            exit_code = output.status.code(),
            stderr = %String::from_utf8_lossy(&output.stderr),
            "tmux command failed"
        );
    }
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub(crate) struct TmuxState {
    pub(crate) current: String,
    pub(crate) alive: HashSet<String>,
    pub(crate) attn: HashMap<String, String>,
}

pub(crate) fn query_state() -> TmuxState {
    let raw = tmux(&[
        "display-message",
        "-p",
        "#S",
        ";",
        "list-sessions",
        "-F",
        "#{session_name}\t#{@attention}",
    ]);
    let mut lines = raw.lines();
    let current = lines.next().unwrap_or("").to_string();
    let mut alive = HashSet::new();
    let mut attn = HashMap::new();
    for line in lines {
        let (name, val) = line.split_once('\t').unwrap_or((line, ""));
        if !name.is_empty() {
            alive.insert(name.to_string());
            if !val.is_empty() {
                attn.insert(name.to_string(), val.to_string());
            }
        }
    }
    TmuxState {
        current,
        alive,
        attn,
    }
}

pub(crate) struct WindowInfo {
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) active: bool,
    pub(crate) zoomed: bool,
}

pub(crate) fn query_windows() -> Vec<WindowInfo> {
    let raw = tmux(&[
        "list-windows",
        "-F",
        "#{window_index}\t#{window_name}\t#{?window_active,1,0}\t#{?window_zoomed_flag,1,0}",
    ]);
    raw.lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(4, '\t');
            let index = parts.next()?.parse().ok()?;
            let name = parts.next()?.to_string();
            let active = parts.next() == Some("1");
            let zoomed = parts.next() == Some("1");
            Some(WindowInfo {
                index,
                name,
                active,
                zoomed,
            })
        })
        .collect()
}

#[derive(Clone)]
pub(crate) struct SystemInfo {
    pub(crate) cpu_load: f32,
    pub(crate) mem_pct: u32,
    pub(crate) battery_pct: Option<u32>,
    pub(crate) battery_state: BatteryState,
    pub(crate) battery_time: String,
    pub(crate) caffeinated: bool,
    pub(crate) date: String,
    pub(crate) clock: String,
}

#[derive(Clone)]
pub(crate) enum BatteryState {
    Charging,
    Discharging,
    Charged,
    NoBattery,
}

fn shell(cmd: &str) -> String {
    Command::new("sh")
        .args(["-c", cmd])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// PIDs of caffeinate processes running without a `-t` timeout (i.e. indefinite).
/// Filters out time-limited invocations like `caffeinate -i -t 300` (Claude Code, etc.).
pub(crate) fn indefinite_caffeinate_pids() -> Vec<String> {
    let Ok(output) = Command::new("pgrep").args(["-x", "caffeinate"]).output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .filter(|pid| {
            Command::new("ps")
                .args(["-o", "args=", "-p", pid])
                .output()
                .ok()
                .map(|o| !String::from_utf8_lossy(&o.stdout).contains("-t "))
                .unwrap_or(false)
        })
        .map(String::from)
        .collect()
}

fn query_cpu_load() -> f32 {
    System::load_average().one as f32
}

fn query_memory_pct(sys: &mut System) -> u32 {
    sys.refresh_memory();
    let total = sys.total_memory();
    if total == 0 {
        return 0;
    }
    // Use available_memory (not used_memory) to match macOS memory_pressure
    // semantics: available includes inactive/cached pages eligible for reuse,
    // so (total - available) / total ≈ memory_pressure's "used percentage".
    let available = sys.available_memory();
    (((total - available) as f64 / total as f64) * 100.0).round() as u32
}

pub(crate) fn query_system_info() -> SystemInfo {
    let mut sys = System::new();
    query_system_info_with(&mut sys)
}

pub(crate) fn query_system_info_with(sys: &mut System) -> SystemInfo {
    let cpu_load = query_cpu_load();
    let mem_pct = query_memory_pct(sys);

    // Battery via pmset (sysinfo doesn't expose macOS battery)
    let batt_raw = shell("pmset -g batt");
    let (battery_pct, battery_state, battery_time) = parse_battery(&batt_raw);

    let caffeinated = !indefinite_caffeinate_pids().is_empty();

    let now = chrono::Local::now();
    let date = now.format("%a %b %-d").to_string();
    let clock = now.format("%H:%M:%S").to_string();

    SystemInfo {
        cpu_load,
        mem_pct,
        battery_pct,
        battery_state,
        battery_time,
        caffeinated,
        date,
        clock,
    }
}

fn parse_battery(raw: &str) -> (Option<u32>, BatteryState, String) {
    let batt_line = match raw.lines().find(|l| l.contains("InternalBattery")) {
        Some(l) => l,
        None => return (None, BatteryState::NoBattery, String::new()),
    };

    let pct: u32 = batt_line
        .split_whitespace()
        .find(|w| w.ends_with("%;"))
        .and_then(|w| w.trim_end_matches("%;").parse().ok())
        .unwrap_or(0);

    let state =
        if raw.contains("charged") && !raw.contains("discharging") && !raw.contains("charging;") {
            BatteryState::Charged
        } else if raw.contains("charging")
            && !raw.contains("discharging")
            && !raw.contains("not charging")
        {
            BatteryState::Charging
        } else {
            BatteryState::Discharging
        };

    // Time remaining: "3:42 remaining"
    let time = batt_line
        .split_whitespace()
        .zip(batt_line.split_whitespace().skip(1))
        .find(|(_, next)| *next == "remaining")
        .map(|(t, _)| t.to_string())
        .unwrap_or_default();

    (Some(pct), state, time)
}

pub(crate) fn git_toplevel(dir: &str) -> Option<String> {
    Command::new("git")
        .args(["-C", dir, "rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
