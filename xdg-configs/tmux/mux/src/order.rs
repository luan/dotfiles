use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::group::session_group;
use crate::tmux::home;

pub(crate) fn order_file() -> PathBuf {
    home().join(".config/tmux/session-order.json")
}

pub(crate) fn hidden_file() -> PathBuf {
    home().join(".config/tmux/session-hidden")
}

fn temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("tmp");
    path.with_file_name(format!("{file_name}.{}.tmp", std::process::id()))
}

pub(crate) fn load_lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

fn do_save(path: &Path, lines: &[String]) -> std::io::Result<()> {
    let mut seen = HashSet::new();
    let deduped: Vec<&str> = lines
        .iter()
        .filter(|l| seen.insert(l.as_str()))
        .map(String::as_str)
        .collect();
    let tmp = temp_path(path);
    let mut f = fs::File::create(&tmp)?;
    for l in &deduped {
        writeln!(f, "{l}")?;
    }
    fs::rename(tmp, path)?;
    Ok(())
}

pub(crate) fn save_lines(path: &Path, lines: &[String]) {
    if let Err(e) = do_save(path, lines) {
        error!(path = %path.display(), error = %e, "save_lines failed");
    }
}

/// A group of sessions sharing the same repo prefix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionGroup {
    pub(crate) name: String,
    pub(crate) sessions: Vec<String>,
}

/// The ordered session store. Groups and orphans interleaved in display order.
/// Invariants enforced by construction:
///   - No duplicate session names across the entire store
///   - Sessions within a group share the group's prefix
///   - Group ordering = display ordering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SessionStore {
    /// Ordered entries: each is either a named group or an orphan (group name is empty, 1 session).
    pub(crate) entries: Vec<SessionGroup>,
}

impl SessionStore {
    pub(crate) fn load() -> Self {
        let path = order_file();
        if let Ok(data) = fs::read_to_string(&path)
            && let Ok(mut store) = serde_json::from_str::<SessionStore>(&data)
        {
            store.normalize_groups();
            return store;
        }
        // Migration: read legacy flat file if JSON doesn't exist
        let legacy = home().join(".config/tmux/session-order");
        if legacy.exists() {
            let lines = load_lines(&legacy);
            let store = Self::from_flat_list(&lines);
            store.save();
            return store;
        }
        Self::default()
    }

    pub(crate) fn save(&self) {
        let path = order_file();
        // Multiple tmux hooks can refresh session order at once while a session
        // is being created or removed. A shared temp filename lets concurrent
        // writers clobber each other and corrupt the JSON store.
        let tmp = temp_path(&path);
        let json = serde_json::to_string_pretty(self).unwrap_or_default();
        if let Err(e) = fs::write(&tmp, json) {
            error!(path = %tmp.display(), error = %e, "session store write failed");
            return;
        }
        if let Err(e) = fs::rename(&tmp, &path) {
            error!(path = %path.display(), error = %e, "session store rename failed");
            return;
        }
        debug!(path = %path.display(), "session store saved");
    }

    /// Build from a flat list of session names (for migration).
    fn from_flat_list(names: &[String]) -> Self {
        let mut store = Self::default();
        let mut seen = HashSet::new();
        for name in names {
            if !seen.insert(name.clone()) {
                continue;
            }
            store.insert(name);
        }
        store
    }

    /// All session names in display order.
    pub(crate) fn ordered_names(&self) -> Vec<String> {
        self.entries
            .iter()
            .flat_map(|g| g.sessions.iter().cloned())
            .collect()
    }

    /// Insert a new session into its group (at end) or as a new group at end.
    /// If the group doesn't exist yet but a session with the group's name exists
    /// as a standalone entry, upgrade that entry into the group.
    pub(crate) fn insert(&mut self, name: &str) {
        if self.contains(name) {
            return;
        }
        let group = session_group(name);
        if let Some(entry) = self.entries.iter_mut().find(|e| e.name == group) {
            entry.sessions.push(name.to_string());
        } else if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.sessions.len() == 1 && e.sessions[0] == group)
        {
            // Upgrade standalone session into a named group
            entry.name = group.to_string();
            entry.sessions.push(name.to_string());
        } else {
            self.entries.push(SessionGroup {
                name: group.to_string(),
                sessions: vec![name.to_string()],
            });
        }
    }

    /// Merge entries that belong to the same group.
    /// Handles legacy stores where standalone sessions had empty group names.
    fn normalize_groups(&mut self) {
        let mut merged = Vec::<SessionGroup>::new();
        for entry in self.entries.drain(..) {
            for session in &entry.sessions {
                let group = session_group(session);
                if let Some(target) = merged.iter_mut().find(|e| e.name == group) {
                    if !target.sessions.contains(session) {
                        target.sessions.push(session.clone());
                    }
                } else {
                    merged.push(SessionGroup {
                        name: group.to_string(),
                        sessions: vec![session.clone()],
                    });
                }
            }
        }
        self.entries = merged;
    }

    /// Remove dead sessions from the store, dropping empty groups.
    pub(crate) fn prune(&mut self, alive: &HashSet<String>) {
        for entry in &mut self.entries {
            entry.sessions.retain(|s| alive.contains(s));
        }
        self.entries.retain(|e| !e.sessions.is_empty());
    }

    pub(crate) fn contains(&self, name: &str) -> bool {
        self.entries
            .iter()
            .any(|g| g.sessions.iter().any(|s| s == name))
    }

    /// Move a session within its group or swap group positions.
    /// Returns true if a move happened.
    pub(crate) fn move_session(&mut self, name: &str, direction: &str) -> bool {
        let group = session_group(name);

        // Find which entry contains this session
        let Some((entry_idx, session_idx)) = self.find_session(name) else {
            return false;
        };

        let entry = &self.entries[entry_idx];

        // If session is in a multi-session group, try moving within the group first
        if entry.sessions.len() > 1 {
            match direction {
                "up" if session_idx > 0 => {
                    self.entries[entry_idx]
                        .sessions
                        .swap(session_idx, session_idx - 1);
                    return true;
                }
                "down" if session_idx < entry.sessions.len() - 1 => {
                    self.entries[entry_idx]
                        .sessions
                        .swap(session_idx, session_idx + 1);
                    return true;
                }
                _ => {} // at boundary — fall through to group move
            }
        }

        // Move the entire group/orphan entry
        let n = self.entries.len();
        match direction {
            "up" if entry_idx > 0 => {
                // Skip over the previous entry (which might be a different group)
                let prev = entry_idx - 1;
                let prev_group = &self.entries[prev].name;
                // Don't let an orphan enter a group
                if group.is_empty()
                    && !prev_group.is_empty()
                    && self.entries[prev].sessions.len() > 1
                {
                    if prev == 0 {
                        return false;
                    }
                    self.entries.swap(entry_idx, prev - 1);
                } else {
                    self.entries.swap(entry_idx, prev);
                }
                true
            }
            "down" if entry_idx < n - 1 => {
                let next = entry_idx + 1;
                let next_group = &self.entries[next].name;
                if group.is_empty()
                    && !next_group.is_empty()
                    && self.entries[next].sessions.len() > 1
                {
                    if next >= n - 1 {
                        return false;
                    }
                    self.entries.swap(entry_idx, next + 1);
                } else {
                    self.entries.swap(entry_idx, next);
                }
                true
            }
            _ => false,
        }
    }

    /// Move an entire group entry up or down. Returns true if moved.
    pub(crate) fn move_group(&mut self, group_name: &str, direction: &str) -> bool {
        let Some(entry_idx) = self.entries.iter().position(|e| e.name == group_name) else {
            return false;
        };
        let n = self.entries.len();
        match direction {
            "up" if entry_idx > 0 => {
                self.entries.swap(entry_idx, entry_idx - 1);
                true
            }
            "down" if entry_idx < n - 1 => {
                self.entries.swap(entry_idx, entry_idx + 1);
                true
            }
            _ => false,
        }
    }

    fn find_session(&self, name: &str) -> Option<(usize, usize)> {
        for (ei, entry) in self.entries.iter().enumerate() {
            for (si, s) in entry.sessions.iter().enumerate() {
                if s == name {
                    return Some((ei, si));
                }
            }
        }
        None
    }

    /// Rename a session in-place (preserves position).
    pub(crate) fn rename(&mut self, old: &str, new: &str) {
        for entry in &mut self.entries {
            for s in &mut entry.sessions {
                if s == old {
                    *s = new.to_string();
                    return;
                }
            }
        }
    }
}

pub(crate) fn compute_order(alive: &HashSet<String>, include_hidden: bool) -> Vec<String> {
    let hidden: HashSet<String> = if include_hidden {
        HashSet::new()
    } else {
        load_lines(&hidden_file()).into_iter().collect()
    };

    let mut store = SessionStore::load();

    // Insert any new alive sessions not yet in the store
    let mut alive_sorted: Vec<&String> = alive.iter().collect();
    alive_sorted.sort();
    for s in alive_sorted {
        store.insert(s);
    }

    // Prune dead sessions
    store.prune(alive);

    store.save();

    store
        .ordered_names()
        .into_iter()
        .filter(|s| alive.contains(s) && !hidden.contains(s))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> String {
        v.to_string()
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|v| s(v)).collect()
    }

    fn make_store(sessions: &[&str]) -> SessionStore {
        SessionStore::from_flat_list(&names(sessions))
    }

    // ── move_session within group ──────────────────────────────

    #[test]
    fn move_session_down_within_group() {
        let mut store = make_store(&["proj/a", "proj/b", "proj/c"]);
        assert!(store.move_session("proj/a", "down"));
        assert_eq!(
            store.ordered_names(),
            names(&["proj/b", "proj/a", "proj/c"])
        );
    }

    #[test]
    fn move_session_up_within_group() {
        let mut store = make_store(&["proj/a", "proj/b", "proj/c"]);
        assert!(store.move_session("proj/c", "up"));
        assert_eq!(
            store.ordered_names(),
            names(&["proj/a", "proj/c", "proj/b"])
        );
    }

    #[test]
    fn move_session_at_top_of_group_moves_group() {
        let mut store = make_store(&["proj/a", "proj/b", "other/x"]);
        assert!(!store.move_session("proj/a", "up"));
    }

    // ── move_session across group boundary ─────────────────────

    #[test]
    fn move_session_at_boundary_swaps_groups() {
        let mut store = make_store(&["proj/a", "proj/b", "other/x"]);
        assert!(store.move_session("proj/b", "down"));
        assert_eq!(
            store.ordered_names(),
            names(&["other/x", "proj/a", "proj/b"])
        );
    }

    #[test]
    fn move_session_at_boundary_swaps_groups_up() {
        let mut store = make_store(&["proj/a", "proj/b", "other/x"]);
        assert!(store.move_session("other/x", "up"));
        assert_eq!(
            store.ordered_names(),
            names(&["other/x", "proj/a", "proj/b"])
        );
    }

    // ── normalize_groups idempotency ───────────────────────────

    #[test]
    fn normalize_groups_idempotent() {
        let mut store = make_store(&["proj/a", "proj/b", "solo", "other/x"]);
        let after_one = store.ordered_names();
        store.normalize_groups();
        assert_eq!(store.ordered_names(), after_one);
        store.normalize_groups();
        assert_eq!(store.ordered_names(), after_one);
    }

    // ── from_flat_list round-trip ──────────────────────────────

    #[test]
    fn from_flat_list_round_trip() {
        let input = names(&["proj/a", "proj/b", "solo", "other/x", "other/y"]);
        let store = SessionStore::from_flat_list(&input);
        assert_eq!(store.ordered_names(), input);
    }

    #[test]
    fn from_flat_list_deduplicates() {
        let input = names(&["a", "b", "a", "c"]);
        let store = SessionStore::from_flat_list(&input);
        assert_eq!(store.ordered_names(), names(&["a", "b", "c"]));
    }

    // ── prune ──────────────────────────────────────────────────

    #[test]
    fn prune_removes_dead_preserves_order() {
        let mut store = make_store(&["proj/a", "proj/b", "solo", "other/x"]);
        let alive: HashSet<String> = ["proj/b", "other/x"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        store.prune(&alive);
        assert_eq!(store.ordered_names(), names(&["proj/b", "other/x"]));
    }

    #[test]
    fn prune_drops_empty_groups() {
        let mut store = make_store(&["proj/a", "proj/b", "solo"]);
        let alive: HashSet<String> = ["solo"].iter().map(|s| s.to_string()).collect();
        store.prune(&alive);
        assert_eq!(store.entries.len(), 1);
        assert_eq!(store.ordered_names(), names(&["solo"]));
    }

    // ── load_lines / save_lines round-trip ─────────────────────

    #[test]
    fn save_load_lines_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-lines");
        let lines = names(&["alpha", "beta", "gamma"]);
        save_lines(&path, &lines);
        let loaded = load_lines(&path);
        assert_eq!(loaded, lines);
    }

    #[test]
    fn save_lines_deduplicates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-dedup");
        let lines = names(&["a", "b", "a", "c"]);
        save_lines(&path, &lines);
        let loaded = load_lines(&path);
        assert_eq!(loaded, names(&["a", "b", "c"]));
    }

    #[test]
    fn move_session_not_found_returns_false() {
        let mut store = make_store(&["a", "b"]);
        assert!(!store.move_session("missing", "up"));
    }

    // Bug: standalone "proj" exists; inserting "proj/feature" should upgrade
    // it into a grouped entry, not create a duplicate.
    #[test]
    fn insert_upgrades_standalone_into_group() {
        let mut store = make_store(&["proj", "other"]);
        store.insert("proj/feature");
        assert_eq!(
            store.ordered_names(),
            names(&["proj", "proj/feature", "other"])
        );
        // Must be a single group entry, not two
        assert_eq!(store.entries.iter().filter(|e| e.name == "proj").count(), 1);
    }

    // Bug: orphan session jumps past a grouped block — distinct code path from
    // the group-boundary swap already tested above.
    #[test]
    fn move_orphan_across_group_boundary() {
        let mut store = make_store(&["solo", "proj/a", "proj/b"]);
        // "solo" is orphan at entry 0; "proj" group at entry 1
        assert!(store.move_session("solo", "down"));
        assert_eq!(store.ordered_names(), names(&["proj/a", "proj/b", "solo"]));
    }
}
