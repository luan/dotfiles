use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::group::session_group;
use crate::tmux::home;

pub fn order_file() -> PathBuf {
    home().join(".config/tmux/session-order.json")
}

pub fn hidden_file() -> PathBuf {
    home().join(".config/tmux/session-hidden")
}

fn temp_path(path: &PathBuf) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("tmp");
    path.with_file_name(format!("{file_name}.{}.tmp", std::process::id()))
}

pub fn load_lines(path: &PathBuf) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

pub fn save_lines(path: &PathBuf, lines: &[String]) {
    let mut seen = HashSet::new();
    let deduped: Vec<&str> = lines
        .iter()
        .filter(|l| seen.insert(l.as_str()))
        .map(String::as_str)
        .collect();
    let tmp = temp_path(path);
    let mut f = fs::File::create(&tmp).unwrap();
    for l in &deduped {
        writeln!(f, "{l}").unwrap();
    }
    fs::rename(tmp, path).unwrap();
}

/// A group of sessions sharing the same repo prefix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionGroup {
    pub name: String,
    pub sessions: Vec<String>,
}

/// The ordered session store. Groups and orphans interleaved in display order.
/// Invariants enforced by construction:
///   - No duplicate session names across the entire store
///   - Sessions within a group share the group's prefix
///   - Group ordering = display ordering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionStore {
    /// Ordered entries: each is either a named group or an orphan (group name is empty, 1 session).
    pub entries: Vec<SessionGroup>,
}

impl SessionStore {
    pub fn load() -> Self {
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

    pub fn save(&self) {
        let path = order_file();
        // Multiple tmux hooks can refresh session order at once while a session
        // is being created or removed. A shared temp filename lets concurrent
        // writers clobber each other and corrupt the JSON store.
        let tmp = temp_path(&path);
        let json = serde_json::to_string_pretty(self).unwrap_or_default();
        fs::write(&tmp, json).ok();
        fs::rename(tmp, path).ok();
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
    pub fn ordered_names(&self) -> Vec<String> {
        self.entries
            .iter()
            .flat_map(|g| g.sessions.iter().cloned())
            .collect()
    }

    /// Insert a new session into its group (at end) or as a new group at end.
    /// If the group doesn't exist yet but a session with the group's name exists
    /// as a standalone entry, upgrade that entry into the group.
    pub fn insert(&mut self, name: &str) {
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
    pub fn prune(&mut self, alive: &HashSet<String>) {
        for entry in &mut self.entries {
            entry.sessions.retain(|s| alive.contains(s));
        }
        self.entries.retain(|e| !e.sessions.is_empty());
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries
            .iter()
            .any(|g| g.sessions.iter().any(|s| s == name))
    }

    /// Move a session within its group or swap group positions.
    /// Returns true if a move happened.
    pub fn move_session(&mut self, name: &str, direction: &str) -> bool {
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
    pub fn move_group(&mut self, group_name: &str, direction: &str) -> bool {
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
    pub fn rename(&mut self, old: &str, new: &str) {
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

pub fn compute_order(alive: &HashSet<String>, include_hidden: bool) -> Vec<String> {
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
