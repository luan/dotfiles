use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::NaiveDate;

const POLL_INTERVAL: Duration = Duration::from_secs(300);

fn log_path() -> PathBuf {
    env::temp_dir().join("copilot-usage-log.tsv")
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn gh_token() -> Option<String> {
    let out = Command::new("gh")
        .args(["auth", "token"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn fetch_json(token: &str) -> Option<serde_json::Value> {
    let out = Command::new("curl")
        .args([
            "-sS",
            "--max-time",
            "10",
            "-H",
            &format!("Authorization: Bearer {token}"),
            "-H",
            "Editor-Version: vscode/1.85",
            "-H",
            "User-Agent: GithubCopilot/1.155",
            "https://api.github.com/copilot_internal/user",
        ])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

fn parse_reset_date(s: &str) -> Option<i64> {
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?;
    Some(d.and_hms_opt(0, 0, 0)?.and_utc().timestamp())
}

fn poll_once() -> Option<()> {
    let token = gh_token()?;
    let json = fetch_json(&token)?;
    let prem = json.get("quota_snapshots")?.get("premium_interactions")?;
    let pct_remaining = prem.get("percent_remaining")?.as_f64()?;
    let used = 100.0 - pct_remaining;
    let reset_date = json.get("quota_reset_date")?.as_str()?;
    let reset_ts = parse_reset_date(reset_date)?;
    let line = format!("{}\t{}\t{}\n", now_ts(), used, reset_ts);
    fs::write(log_path(), line).ok()
}

/// Spawn a background thread that refreshes the Copilot usage TSV every
/// 5 minutes. Call once at sidebar startup.
pub fn start_poller() {
    thread::spawn(|| {
        loop {
            let _ = poll_once();
            thread::sleep(POLL_INTERVAL);
        }
    });
}
