use std::env;
use std::io;
use std::process::{Command, Stdio};

/// Spawn a background child and reap it when it exits.
///
/// Dropping `std::process::Child` does not wait for the child on Unix, so any
/// short-lived helper spawned by a long-lived `mux sidebar` process becomes a
/// zombie until the sidebar exits. Keep ownership of the child in a tiny
/// reaper thread so background helpers can still run asynchronously without
/// leaving `<defunct>` children behind.
pub(crate) fn spawn_reaped(mut command: Command) -> io::Result<()> {
    let mut child = command.spawn()?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Spawn `mux update` in the background, detached from the caller's stdio.
/// Used after state changes (move, sidebar close) so tmux refreshes without
/// blocking the initiating keypress.
pub(crate) fn spawn_detached_update() {
    let exe = env::current_exe().unwrap_or_else(|_| "mux".into());
    let mut command = Command::new(exe);
    command
        .args(["update"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _ = spawn_reaped(command);
}
