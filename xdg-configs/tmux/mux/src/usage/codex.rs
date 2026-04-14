use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::{UsagePoller, UsageSnapshot, now_ts};

pub(super) struct CodexPoller;

impl UsagePoller for CodexPoller {
    fn provider_name(&self) -> &'static str {
        "codex"
    }

    fn log_path(&self) -> PathBuf {
        env::temp_dir().join("codex-usage-log.tsv")
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_secs(120)
    }

    fn token(&self) -> Option<String> {
        let home = env::var("HOME").ok()?;
        let raw = fs::read_to_string(PathBuf::from(home).join(".codex/auth.json")).ok()?;
        let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
        json.get("tokens")?
            .get("access_token")?
            .as_str()
            .map(String::from)
    }

    fn endpoint(&self) -> &'static str {
        "https://chatgpt.com/backend-api/wham/usage"
    }

    fn parse_response(&self, body: &str) -> Option<UsageSnapshot> {
        let json: serde_json::Value = serde_json::from_str(body).ok()?;
        let rate = json.get("rate_limit")?;
        let prim = rate.get("primary_window")?;
        let sec = rate.get("secondary_window")?;
        let prim_pct = prim.get("used_percent")?.as_f64()?;
        let prim_reset = prim.get("reset_at")?.as_i64()?;
        let sec_pct = sec.get("used_percent")?.as_f64()?;
        let sec_reset = sec.get("reset_at")?.as_i64()?;

        Some(UsageSnapshot {
            lines: format!(
                "{}\t{}\t{}\t{}\t{}\n",
                now_ts(),
                prim_pct,
                prim_reset,
                sec_pct,
                sec_reset,
            ),
            append: true,
        })
    }
}
