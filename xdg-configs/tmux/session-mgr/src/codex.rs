use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const POLL_INTERVAL: Duration = Duration::from_secs(120);

fn log_path() -> PathBuf {
    env::temp_dir().join("codex-usage-log.tsv")
}

fn auth_path() -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".codex/auth.json"))
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn access_token() -> Option<String> {
    let raw = fs::read_to_string(auth_path()?).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    json.get("tokens")?
        .get("access_token")?
        .as_str()
        .map(String::from)
}

fn fetch_usage(token: &str) -> Option<serde_json::Value> {
    let out = Command::new("curl")
        .args([
            "-sS",
            "--max-time",
            "10",
            "-H",
            &format!("Authorization: Bearer {token}"),
            "https://chatgpt.com/backend-api/wham/usage",
        ])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

fn poll_once() -> Option<()> {
    let token = access_token()?;
    let json = fetch_usage(&token)?;
    let rate = json.get("rate_limit")?;
    let prim = rate.get("primary_window")?;
    let sec = rate.get("secondary_window")?;
    let prim_pct = prim.get("used_percent")?.as_f64()?;
    let prim_reset = prim.get("reset_at")?.as_i64()?;
    let sec_pct = sec.get("used_percent")?.as_f64()?;
    let sec_reset = sec.get("reset_at")?.as_i64()?;

    let line = format!(
        "{}\t{}\t{}\t{}\t{}\n",
        now_ts(),
        prim_pct,
        prim_reset,
        sec_pct,
        sec_reset
    );
    let path = log_path();
    // Append so history builds up over time; usage_bars reads monotonic max.
    let existing = fs::read_to_string(&path).unwrap_or_default();
    fs::write(&path, format!("{existing}{line}")).ok()
}

pub fn start_poller() {
    thread::spawn(|| {
        loop {
            let _ = poll_once();
            thread::sleep(POLL_INTERVAL);
        }
    });
}
