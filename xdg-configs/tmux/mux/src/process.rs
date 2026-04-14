use std::env;
use std::process::{Command, Stdio};

/// Spawn `mux update` in the background, detached from the caller's stdio.
/// Used after state changes (move, sidebar close) so tmux refreshes without
/// blocking the initiating keypress.
pub(crate) fn spawn_detached_update() {
    let exe = env::current_exe().unwrap_or_else(|_| "mux".into());
    let _ = Command::new(exe)
        .args(["update"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}
