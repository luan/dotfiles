//! Claude Code hook integration for mux sidebar state.
//!
//! Pane scraping is best-effort: it depends on the visible terminal buffer and
//! can miss permission prompts or stale active lines. Claude hooks give us a
//! small structured heartbeat keyed by the tmux pane that launched Claude.

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::tmux::home;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) enum HookEvent {
    UserPromptSubmit,
    PreToolUse,
    PermissionRequest,
    PostToolUse,
    Stop,
    SubagentStop,
    Notification,
    SessionStart,
    SessionEnd,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct HookRecord {
    pub(super) pane_id: String,
    pub(super) session_id: String,
    pub(super) event: HookEvent,
    pub(super) timestamp_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HookPayload {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    hook_event_name: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct HookSignal {
    pub(super) gerund: Option<String>,
    pub(super) asking: bool,
    pub(super) age: Option<Duration>,
    pub(super) idle: bool,
}

pub(super) fn install(cwd: &str) {
    if cwd.is_empty() {
        return;
    }
    if let Err(err) = install_inner(Path::new(cwd)) {
        tracing::debug!(cwd, error = %err, "install claude hooks");
    }
}

fn install_inner(cwd: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    let command = format!("{} hook", shell_escape(&exe.to_string_lossy()));

    let settings_dir = cwd.join(".claude");
    fs::create_dir_all(&settings_dir)?;
    let settings_path = settings_dir.join("settings.local.json");

    let mut root: serde_json::Value = if settings_path.exists() {
        let raw = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    if !root.is_object() {
        root = serde_json::json!({});
    }

    let obj = root.as_object_mut().expect("root is object");
    let hooks_entry = obj
        .entry("hooks".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !hooks_entry.is_object() {
        *hooks_entry = serde_json::json!({});
    }
    let hooks = hooks_entry.as_object_mut().expect("hooks is object");

    for event in [
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "Stop",
        "SubagentStop",
        "Notification",
    ] {
        ensure_hook_command(hooks, event, &command);
    }

    let pretty = serde_json::to_string_pretty(&root)?;
    let unchanged = fs::read_to_string(&settings_path)
        .map(|existing| existing == pretty)
        .unwrap_or(false);
    if !unchanged {
        fs::write(settings_path, pretty)?;
    }
    Ok(())
}

fn ensure_hook_command(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    command: &str,
) {
    let entry = hooks
        .entry(event.to_string())
        .or_insert_with(|| serde_json::json!([]));
    if !entry.is_array() {
        *entry = serde_json::json!([]);
    }
    let arr = entry.as_array_mut().expect("hook entry is array");

    let already_present = arr.iter().any(|group| {
        group
            .get("hooks")
            .and_then(|h| h.as_array())
            .map(|hs| {
                hs.iter()
                    .any(|h| h.get("command").and_then(|c| c.as_str()) == Some(command))
            })
            .unwrap_or(false)
    });
    if already_present {
        return;
    }

    arr.push(serde_json::json!({
        "hooks": [
            { "type": "command", "command": command }
        ]
    }));
}

pub(super) fn read_signal(pane_id: &str) -> Option<HookSignal> {
    let rec = read_record(pane_id)?;
    let age = Duration::from_millis(now_ms().saturating_sub(rec.timestamp_ms));
    // Avoid making a days-old hook record look like live work after a tmux
    // server restart or a stale pane id reuse.
    if age > Duration::from_secs(60 * 60) {
        return None;
    }
    Some(match rec.event {
        HookEvent::UserPromptSubmit => HookSignal {
            gerund: Some("Thinking…".to_string()),
            asking: false,
            age: Some(age),
            idle: false,
        },
        HookEvent::PreToolUse | HookEvent::PostToolUse => HookSignal {
            gerund: Some(
                rec.tool_name
                    .as_deref()
                    .map(tool_gerund)
                    .unwrap_or_else(|| "Working…".to_string()),
            ),
            asking: false,
            age: Some(age),
            idle: false,
        },
        HookEvent::PermissionRequest | HookEvent::Notification => HookSignal {
            gerund: None,
            asking: true,
            age: Some(age),
            idle: true,
        },
        HookEvent::Stop | HookEvent::SubagentStop | HookEvent::SessionEnd => HookSignal {
            gerund: None,
            asking: false,
            age: Some(age),
            idle: true,
        },
        HookEvent::SessionStart | HookEvent::Other => HookSignal {
            gerund: None,
            asking: false,
            age: Some(age),
            idle: false,
        },
    })
}

fn read_record(pane_id: &str) -> Option<HookRecord> {
    let path = record_path(pane_id);
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub(crate) fn ingest_stdin() {
    let mut raw = String::new();
    if io::stdin().read_to_string(&mut raw).is_err() || raw.trim().is_empty() {
        return;
    }
    let Ok(payload) = serde_json::from_str::<HookPayload>(&raw) else {
        return;
    };
    let pane_id = std::env::var("TMUX_PANE").unwrap_or_default();
    if pane_id.is_empty() {
        return;
    }
    let event = payload
        .hook_event_name
        .as_deref()
        .and_then(parse_event)
        .unwrap_or(HookEvent::Other);
    let record = HookRecord {
        pane_id: pane_id.clone(),
        session_id: payload.session_id.unwrap_or_default(),
        event,
        timestamp_ms: now_ms(),
        tool_name: payload.tool_name,
        message: payload.message,
    };
    let _ = write_atomic(&record_path(&pane_id), &record);
}

fn parse_event(name: &str) -> Option<HookEvent> {
    serde_json::from_str::<HookEvent>(&format!("\"{name}\"")).ok()
}

fn tool_gerund(tool: &str) -> String {
    match tool {
        "Read" => "Reading…",
        "Write" | "Edit" | "MultiEdit" => "Editing…",
        "Bash" => "Running…",
        "Grep" | "Glob" | "LS" => "Searching…",
        "Task" => "Delegating…",
        _ => "Working…",
    }
    .to_string()
}

fn record_path(pane_id: &str) -> PathBuf {
    state_dir().join(format!("{}.json", sanitize(pane_id)))
}

fn state_dir() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("mux/claude-hooks")
}

fn write_atomic(target: &Path, record: &HookRecord) -> io::Result<()> {
    let Some(dir) = target.parent() else {
        return Ok(());
    };
    fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(
        ".{}.tmp",
        target
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("hook")
    ));
    let bytes = serde_json::to_vec(record).map_err(io::Error::other)?;
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, target)?;
    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn shell_escape(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "/_-.".contains(c))
    {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}
