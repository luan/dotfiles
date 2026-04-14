use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use chrono::NaiveDate;

use super::{UsagePoller, UsageSnapshot, now_ts};

pub(super) struct CopilotPoller;

impl UsagePoller for CopilotPoller {
    fn provider_name(&self) -> &'static str {
        "copilot"
    }

    fn log_path(&self) -> PathBuf {
        env::temp_dir().join("copilot-usage-log.tsv")
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_secs(300)
    }

    fn token(&self) -> Option<String> {
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

    fn endpoint(&self) -> &'static str {
        "https://api.github.com/copilot_internal/user"
    }

    fn extra_headers(&self) -> Vec<String> {
        vec![
            "Editor-Version: vscode/1.85".to_string(),
            "User-Agent: GithubCopilot/1.155".to_string(),
        ]
    }

    fn parse_response(&self, body: &str) -> Option<UsageSnapshot> {
        let json: serde_json::Value = serde_json::from_str(body).ok()?;
        let prem = json.get("quota_snapshots")?.get("premium_interactions")?;
        let pct_remaining = prem.get("percent_remaining")?.as_f64()?;
        let used = 100.0 - pct_remaining;
        let reset_date = json.get("quota_reset_date")?.as_str()?;
        let reset_ts = parse_reset_date(reset_date)?;
        Some(UsageSnapshot {
            lines: format!("{}\t{}\t{}\n", now_ts(), used, reset_ts),
            append: true,
        })
    }
}

fn parse_reset_date(s: &str) -> Option<i64> {
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?;
    Some(d.and_hms_opt(0, 0, 0)?.and_utc().timestamp())
}
