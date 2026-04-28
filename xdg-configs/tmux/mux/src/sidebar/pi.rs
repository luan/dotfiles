use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::tmux::home;

use super::claude::AgentCtx;
use super::meta::{AgentInstance, PaneInfo};

const STATE_STALE: Duration = Duration::from_secs(8);

#[derive(Debug, Deserialize)]
struct PiAgentState {
    version: u32,
    agent: String,
    #[serde(rename = "paneId")]
    pane_id: String,
    pid: u32,
    #[serde(rename = "updatedAtMs")]
    updated_at_ms: u64,
    #[serde(rename = "lastActivityAtMs")]
    last_activity_at_ms: u64,
    activity: Option<String>,
    asking: bool,
    ctx: Option<PiAgentCtx>,
}

#[derive(Debug, Deserialize)]
struct PiAgentCtx {
    pct: u8,
    tokens: String,
}

/// Pi's mux-sidebar extension writes one JSON heartbeat per tmux pane. Prefer
/// that native state over pane scraping so mux can display Pi reliably even when
/// the CLI process is a node shim instead of a literal `pi` process.
pub(super) fn query_pi_agents(all_panes: &[PaneInfo]) -> HashMap<(String, String), AgentInstance> {
    let pane_sessions: HashMap<&str, &str> = all_panes
        .iter()
        .map(|pane| (pane.pane_id.as_str(), pane.session.as_str()))
        .collect();
    if pane_sessions.is_empty() {
        return HashMap::new();
    }

    let mut seen_pids = HashSet::new();
    let mut result = HashMap::new();
    let Ok(entries) = fs::read_dir(state_dir()) else {
        return result;
    };

    let now = now_ms();
    for entry in entries.flatten() {
        let Ok(contents) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(state) = serde_json::from_str::<PiAgentState>(&contents) else {
            continue;
        };
        if state.version != 1 || state.agent != "pi" {
            continue;
        }
        if now.saturating_sub(state.updated_at_ms) > STATE_STALE.as_millis() as u64 {
            continue;
        }
        let Some(&session) = pane_sessions.get(state.pane_id.as_str()) else {
            continue;
        };
        if !seen_pids.insert(state.pid) || !pid_alive(state.pid) {
            continue;
        }

        let age = if state.last_activity_at_ms == 0 {
            None
        } else {
            Some(Duration::from_millis(
                now.saturating_sub(state.last_activity_at_ms),
            ))
        };
        result.insert(
            (session.to_string(), state.pane_id.clone()),
            AgentInstance {
                name: "pi".to_string(),
                pane_id: state.pane_id,
                gerund: state.activity,
                ctx: state.ctx.map(|ctx| AgentCtx {
                    pct: ctx.pct.min(100),
                    tokens: ctx.tokens,
                }),
                age,
                asking: state.asking,
            },
        );
    }

    result
}

fn pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn state_dir() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("mux/pi-agents")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pi_state() {
        let state: PiAgentState = serde_json::from_str(
            r#"{
                "version": 1,
                "agent": "pi",
                "paneId": "%12",
                "pid": 123,
                "updatedAtMs": 10,
                "lastActivityAtMs": 5,
                "activity": "Thinking…",
                "asking": false,
                "ctx": { "pct": 42, "tokens": "84k/200k" }
            }"#,
        )
        .unwrap();

        assert_eq!(state.pane_id, "%12");
        assert_eq!(state.activity.as_deref(), Some("Thinking…"));
        assert_eq!(state.ctx.unwrap().tokens, "84k/200k");
    }
}
