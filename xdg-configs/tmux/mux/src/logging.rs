use std::fs;
use std::io::Write;
use std::process::Command;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::tmux::home;

fn state_dir() -> Option<std::path::PathBuf> {
    let dir = home().join(".local/state/mux");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn prune_old_logs(dir: &std::path::Path) {
    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(7 * 24 * 60 * 60);
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        if !name_str.starts_with("mux.log") {
            continue;
        }
        if let Ok(meta) = entry.metadata()
            && let Ok(modified) = meta.modified()
            && modified < cutoff
        {
            let _ = fs::remove_file(entry.path());
        }
    }
}

pub(crate) fn init_tracing() -> Option<WorkerGuard> {
    let dir = state_dir()?;
    prune_old_logs(&dir);

    let appender = tracing_appender::rolling::daily(&dir, "mux.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(appender);

    let filter = EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::WARN.into())
        .with_env_var("MUX_LOG")
        .from_env_lossy();

    let layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_level(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .try_init()
        .ok()?;

    Some(guard)
}

fn latest_log(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut logs: Vec<std::path::PathBuf> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("mux.log"))
        })
        .map(|e| e.path())
        .collect();
    logs.sort();
    logs.pop()
}

pub(crate) fn cmd_log(args: &[String]) {
    let log_dir = match state_dir() {
        Some(d) => d,
        None => {
            eprintln!("cannot determine state directory");
            std::process::exit(1);
        }
    };
    let path = match latest_log(&log_dir) {
        Some(p) => p,
        None => {
            eprintln!("no log file in {}", log_dir.display());
            std::process::exit(1);
        }
    };
    let path_str = path.to_string_lossy();

    if args.iter().any(|a| a == "--clear") {
        eprint!("truncate {}? [y/N] ", path.display());
        let _ = std::io::stderr().flush();
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_ok()
            && answer.trim().eq_ignore_ascii_case("y")
        {
            let _ = fs::File::create(&path);
        }
        return;
    }

    if args.iter().any(|a| a == "-f" || a == "--follow") {
        let _ = Command::new("tail").args(["-f", &path_str]).status();
        return;
    }

    // Default: open in less, fall back to tail, fall back to cat
    let less = Command::new("less").args(["+G", &path_str]).status();
    if less.is_err() {
        let tail = Command::new("tail").args(["-n", "200", &path_str]).status();
        if tail.is_err() {
            let _ = Command::new("cat").arg(&*path_str).status();
        }
    }
}
