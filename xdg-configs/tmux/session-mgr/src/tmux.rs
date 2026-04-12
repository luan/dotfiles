use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn home() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap())
}

pub fn tmux(args: &[&str]) -> String {
    String::from_utf8_lossy(
        &Command::new("tmux")
            .args(args)
            .stderr(Stdio::null())
            .output()
            .expect("failed to run tmux")
            .stdout,
    )
    .trim()
    .to_string()
}

pub struct TmuxState {
    pub current: String,
    pub alive: HashSet<String>,
    pub attn: HashMap<String, String>,
}

pub fn query_state() -> TmuxState {
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

pub struct WindowInfo {
    pub index: usize,
    pub name: String,
    pub active: bool,
    pub zoomed: bool,
}

pub fn query_windows() -> Vec<WindowInfo> {
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

pub struct SystemInfo {
    pub cpu_load: f32,
    pub mem_pct: u32,
    pub battery_pct: Option<u32>,
    pub battery_state: BatteryState,
    pub battery_time: String,
    pub caffeinated: bool,
    pub clock: String,
}

pub enum BatteryState {
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

pub fn query_system_info() -> SystemInfo {
    // CPU: 1-minute load average
    let cpu_load: f32 = shell("sysctl -n vm.loadavg")
        .trim_start_matches('{')
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);

    // Memory: parse vm_stat
    let mem_pct = parse_memory();

    // Battery via pmset
    let batt_raw = shell("pmset -g batt");
    let (battery_pct, battery_state, battery_time) = parse_battery(&batt_raw);

    // Caffeinate: check if running
    let caffeinated = Command::new("pgrep")
        .args(["-x", "caffeinate"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());

    // Clock
    let clock = chrono::Local::now().format("%H:%M").to_string();

    SystemInfo {
        cpu_load,
        mem_pct,
        battery_pct,
        battery_state,
        battery_time,
        caffeinated,
        clock,
    }
}

fn parse_memory() -> u32 {
    // macOS memory pressure: "System-wide memory free percentage: 69%"
    let raw = shell("memory_pressure -Q");
    let free_pct: u32 = raw
        .lines()
        .find(|l| l.contains("free percentage"))
        .and_then(|l| {
            l.split_whitespace()
                .find(|w| w.ends_with('%'))
                .and_then(|w| w.trim_end_matches('%').parse().ok())
        })
        .unwrap_or(100);
    100u32.saturating_sub(free_pct)
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

pub fn git_toplevel(dir: &str) -> Option<String> {
    Command::new("git")
        .args(["-C", dir, "rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
