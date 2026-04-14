use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tracing::{debug, error};

mod codex;
mod copilot;

use codex::CodexPoller;
use copilot::CopilotPoller;

trait UsagePoller: Send + 'static {
    fn provider_name(&self) -> &'static str;
    fn log_path(&self) -> PathBuf;
    fn poll_interval(&self) -> Duration;
    fn token(&self) -> Option<String>;
    fn endpoint(&self) -> &'static str;
    fn extra_headers(&self) -> Vec<String> {
        Vec::new()
    }
    fn parse_response(&self, body: &str) -> Option<UsageSnapshot>;

    fn run_loop(&self) {
        loop {
            match self.poll_once() {
                Some(()) => debug!(provider = self.provider_name(), "poll ok"),
                None => error!(provider = self.provider_name(), "poll failed"),
            }
            thread::sleep(self.poll_interval());
        }
    }

    fn poll_once(&self) -> Option<()> {
        let token = self.token()?;
        let body = fetch(self.endpoint(), &token, &self.extra_headers())?;
        let snap = self.parse_response(&body)?;
        write_snapshot(&self.log_path(), &snap)
    }
}

struct UsageSnapshot {
    lines: String,
    /// When true, append to the log file (history builds over time).
    /// When false, overwrite with the latest sample only.
    append: bool,
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn fetch(endpoint: &str, token: &str, extra_headers: &[String]) -> Option<String> {
    let mut args = vec![
        "-sS".to_string(),
        "--max-time".to_string(),
        "10".to_string(),
        "-H".to_string(),
        format!("Authorization: Bearer {token}"),
    ];
    for h in extra_headers {
        args.push("-H".to_string());
        args.push(h.clone());
    }
    args.push(endpoint.to_string());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = Command::new("curl")
        .args(&refs)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

fn write_snapshot(path: &Path, snap: &UsageSnapshot) -> Option<()> {
    if snap.append {
        let existing = fs::read_to_string(path).unwrap_or_default();
        fs::write(path, format!("{existing}{}", snap.lines)).ok()
    } else {
        fs::write(path, &snap.lines).ok()
    }
}

pub(super) fn start_all() {
    thread::spawn(|| CodexPoller.run_loop());
    thread::spawn(|| CopilotPoller.run_loop());
}
