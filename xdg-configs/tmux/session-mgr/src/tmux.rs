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

pub fn git_toplevel(dir: &str) -> Option<String> {
    Command::new("git")
        .args(["-C", dir, "rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
