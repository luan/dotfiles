use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::picker::PickerItem;
use crate::project::{
    DitchPlan, WtEntry, build_ditch_plan, build_project_items, build_worktree_items,
    create_new_worktree, create_session_at_dir, default_session_name, execute_ditch_action,
    list_worktrees, next_wt_suffix, rename_parts, rename_session, repo_root_name,
    resolve_common_dir, resolve_selected_dir_from_session, toggle_favorite, touch_lru,
    worktree_name_parts,
};
use crate::tmux::tmux;

use super::{SidebarState, handoff_to_main};

pub(super) enum SidebarOverlay {
    Rename(RenameOverlay),
    Ditch(ListOverlay),
    Project(ProjectOverlay),
    Worktree(WorktreeOverlay),
    SessionName(SessionNameOverlay),
}

pub(super) enum OverlayKeyResult {
    Unhandled,
    Keep,
    Close,
}

pub(super) struct RenameOverlay {
    pub(super) old_name: String,
    pub(super) prefix: String,
    pub(super) input: String,
    pub(super) cursor: usize,
    pub(super) error: Option<String>,
}

pub(super) struct ListOverlay {
    pub(super) items: Vec<PickerItem>,
    pub(super) selected: usize,
    pub(super) offset: usize,
    pub(super) error: Option<String>,
    pub(super) plan: Option<DitchPlan>,
}

pub(super) struct ProjectOverlay {
    pub(super) filter: String,
    pub(super) cursor: usize,
    pub(super) all_items: Vec<PickerItem>,
    pub(super) items: Vec<PickerItem>,
    pub(super) selected: usize,
    pub(super) offset: usize,
}

pub(super) enum WorktreeFlow {
    NewSession,
    NewWorktree,
}

pub(super) struct WorktreeOverlay {
    pub(super) flow: WorktreeFlow,
    pub(super) selected_dir: PathBuf,
    pub(super) entries: Vec<WtEntry>,
    pub(super) items: Vec<PickerItem>,
    pub(super) selected: usize,
    pub(super) offset: usize,
    pub(super) error: Option<String>,
}

pub(super) struct SessionNameOverlay {
    pub(super) title: String,
    pub(super) prefix: String,
    pub(super) input: String,
    pub(super) cursor: usize,
    pub(super) default_on_empty: Option<String>,
    pub(super) final_dir: PathBuf,
    pub(super) pending_worktree: Option<PathBuf>,
    pub(super) error: Option<String>,
}

pub(super) fn first_selectable_picker(items: &[PickerItem]) -> usize {
    items.iter().position(|item| item.selectable).unwrap_or(0)
}

pub(super) fn move_picker_selection(items: &[PickerItem], selected: &mut usize, dir: i32) {
    if items.is_empty() {
        return;
    }
    let mut pos = (*selected).min(items.len().saturating_sub(1));
    loop {
        if dir > 0 {
            if pos + 1 >= items.len() {
                return;
            }
            pos += 1;
        } else {
            if pos == 0 {
                return;
            }
            pos -= 1;
        }
        if items[pos].selectable {
            *selected = pos;
            return;
        }
    }
}

fn prev_char_boundary(s: &str, cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }
    s[..cursor]
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

fn next_char_boundary(s: &str, cursor: usize) -> usize {
    if cursor >= s.len() {
        return s.len();
    }
    let mut iter = s[cursor..].char_indices();
    let _ = iter.next();
    iter.next().map(|(idx, _)| cursor + idx).unwrap_or(s.len())
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '/' | '.')
}

fn prev_word_boundary(s: &str, cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }

    let chars: Vec<(usize, char)> = s[..cursor].char_indices().collect();
    let mut i = chars.len();
    while i > 0 && !is_word_char(chars[i - 1].1) {
        i -= 1;
    }
    while i > 0 && is_word_char(chars[i - 1].1) {
        i -= 1;
    }
    chars.get(i).map(|(idx, _)| *idx).unwrap_or(0)
}

fn next_word_boundary(s: &str, cursor: usize) -> usize {
    if cursor >= s.len() {
        return s.len();
    }

    let chars: Vec<(usize, char)> = s[cursor..].char_indices().collect();
    let mut i = 0usize;
    while i < chars.len() && !is_word_char(chars[i].1) {
        i += 1;
    }
    while i < chars.len() && is_word_char(chars[i].1) {
        i += 1;
    }
    chars.get(i).map(|(idx, _)| cursor + idx).unwrap_or(s.len())
}

pub(super) fn handle_readline_key(text: &mut String, cursor: &mut usize, key: KeyEvent) -> bool {
    match (key.code, key.modifiers) {
        (KeyCode::Char(c), m) if !m.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            text.insert(*cursor, c);
            *cursor += c.len_utf8();
            true
        }
        (KeyCode::Backspace, KeyModifiers::ALT) | (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
            let prev = prev_word_boundary(text, *cursor);
            text.drain(prev..*cursor);
            *cursor = prev;
            true
        }
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            if *cursor > 0 {
                let prev = prev_char_boundary(text, *cursor);
                text.drain(prev..*cursor);
                *cursor = prev;
            }
            true
        }
        (KeyCode::Delete, _) => {
            if *cursor < text.len() {
                let next = next_char_boundary(text, *cursor);
                text.drain(*cursor..next);
            }
            true
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            if *cursor < text.len() {
                let next = next_char_boundary(text, *cursor);
                text.drain(*cursor..next);
            }
            true
        }
        (KeyCode::Left, _) | (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
            *cursor = prev_char_boundary(text, *cursor);
            true
        }
        (KeyCode::Right, _) | (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
            *cursor = next_char_boundary(text, *cursor);
            true
        }
        (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
            *cursor = 0;
            true
        }
        (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
            *cursor = text.len();
            true
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            text.drain(..*cursor);
            *cursor = 0;
            true
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            text.truncate(*cursor);
            true
        }
        (KeyCode::Char('b'), KeyModifiers::ALT) => {
            *cursor = prev_word_boundary(text, *cursor);
            true
        }
        (KeyCode::Char('f'), KeyModifiers::ALT) => {
            *cursor = next_word_boundary(text, *cursor);
            true
        }
        (KeyCode::Char('d'), KeyModifiers::ALT) => {
            let next = next_word_boundary(text, *cursor);
            text.drain(*cursor..next);
            true
        }
        _ => false,
    }
}

pub(super) fn filter_picker_items(items: &[PickerItem], query: &str) -> Vec<PickerItem> {
    if query.is_empty() {
        return items.to_vec();
    }
    let matches =
        crate::filter::fuzzy_match(items, query, |item| format!("{} {}", item.display, item.id));
    matches
        .into_iter()
        .map(|(idx, _)| items[idx].clone())
        .collect()
}

fn session_exists(session_name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", &format!("={session_name}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn all_pane_dirs() -> Vec<PathBuf> {
    tmux(&["list-panes", "-a", "-F", "#{pane_current_path}"])
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn used_worktree_paths(entries: &[WtEntry]) -> HashSet<String> {
    let pane_dirs = all_pane_dirs();
    entries
        .iter()
        .filter(|entry| {
            let entry_path = Path::new(&entry.path);
            pane_dirs.iter().any(|dir| dir.starts_with(entry_path))
        })
        .map(|entry| entry.path.clone())
        .collect()
}

fn build_sidebar_worktree_items(entries: &[WtEntry]) -> (Vec<PickerItem>, usize) {
    let used = used_worktree_paths(entries);
    let mut items = build_worktree_items(entries);

    for item in items.iter_mut().skip(1) {
        if used.contains(&item.id) {
            item.right_label = "live".to_string();
        }
    }

    let selected = items
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, item)| item.selectable && !used.contains(&item.id))
        .map(|(idx, _)| idx)
        .unwrap_or_else(|| first_selectable_picker(&items));

    (items, selected)
}

// impl SidebarState continued in overlay.rs
impl SidebarState {
    pub(super) fn handle_overlay_key(&mut self, key: KeyEvent) -> bool {
        let Some(mut overlay) = self.overlay.take() else {
            return false;
        };

        let result = match &mut overlay {
            SidebarOverlay::Rename(rename) => self.handle_rename_key(rename, key),
            SidebarOverlay::Ditch(list) => self.handle_ditch_key(list, key),
            SidebarOverlay::Project(project) => self.handle_project_key(project, key),
            SidebarOverlay::Worktree(worktree) => self.handle_worktree_key(worktree, key),
            SidebarOverlay::SessionName(session) => self.handle_session_name_key(session, key),
        };

        match result {
            OverlayKeyResult::Unhandled => {
                self.overlay = Some(overlay);
                false
            }
            OverlayKeyResult::Keep => {
                if self.overlay.is_none() {
                    self.overlay = Some(overlay);
                }
                true
            }
            OverlayKeyResult::Close => true,
        }
    }

    fn handle_rename_key(&mut self, rename: &mut RenameOverlay, key: KeyEvent) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                let new_suffix = rename.input.trim();
                if new_suffix.is_empty() {
                    rename.error = Some("session name required".to_string());
                    return OverlayKeyResult::Keep;
                }
                let new_name = format!("{}{}", rename.prefix, new_suffix);
                match rename_session(&rename.old_name, &new_name) {
                    Ok(()) => OverlayKeyResult::Close,
                    Err(err) => {
                        rename.error = Some(err);
                        OverlayKeyResult::Keep
                    }
                }
            }
            _ if handle_readline_key(&mut rename.input, &mut rename.cursor, key) => {
                rename.error = None;
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
        }
    }

    fn handle_ditch_key(&mut self, list: &mut ListOverlay, key: KeyEvent) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                if let Some(item) = list.items.get(list.selected)
                    && item.selectable
                    && let Some(plan) = list.plan.as_ref()
                {
                    match execute_ditch_action(plan, &item.id) {
                        Ok(()) => {
                            handoff_to_main(self);
                            return OverlayKeyResult::Close;
                        }
                        Err(err) => list.error = Some(err),
                    }
                }
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                move_picker_selection(&list.items, &mut list.selected, 1);
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                move_picker_selection(&list.items, &mut list.selected, -1);
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
        }
    }

    fn handle_project_key(
        &mut self,
        project: &mut ProjectOverlay,
        key: KeyEvent,
    ) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                let Some(item) = project.items.get(project.selected) else {
                    return OverlayKeyResult::Keep;
                };
                let selected_dir = PathBuf::from(&item.id);
                touch_lru(&item.id);
                self.open_worktree_overlay_for_dir(selected_dir, WorktreeFlow::NewSession);
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('s'), KeyModifiers::ALT) => {
                if let Some(item) = project.items.get(project.selected) {
                    toggle_favorite(&item.id);
                    project.all_items = build_project_items("all");
                    project.items = filter_picker_items(&project.all_items, &project.filter);
                    project.selected = first_selectable_picker(&project.items);
                    project.offset = 0;
                }
                OverlayKeyResult::Keep
            }
            _ if handle_readline_key(&mut project.filter, &mut project.cursor, key) => {
                project.items = filter_picker_items(&project.all_items, &project.filter);
                project.selected = first_selectable_picker(&project.items);
                project.offset = 0;
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                move_picker_selection(&project.items, &mut project.selected, 1);
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                move_picker_selection(&project.items, &mut project.selected, -1);
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
        }
    }

    fn handle_worktree_key(
        &mut self,
        worktree: &mut WorktreeOverlay,
        key: KeyEvent,
    ) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                let Some(item) = worktree.items.get(worktree.selected) else {
                    return OverlayKeyResult::Keep;
                };

                if item.id == "__new__" {
                    let Some(common_dir) = resolve_common_dir(&worktree.selected_dir) else {
                        worktree.error = Some("not a git repo".to_string());
                        return OverlayKeyResult::Keep;
                    };
                    let repo_name = repo_root_name(&worktree.selected_dir, &common_dir);
                    let default_suffix =
                        next_wt_suffix(&worktree.selected_dir, &common_dir, &worktree.entries);
                    let (title, prefix) = if repo_name.is_empty() {
                        ("Worktree".to_string(), String::new())
                    } else {
                        (format!("{repo_name}."), format!("{repo_name}/"))
                    };
                    self.open_session_name_overlay(
                        title,
                        prefix,
                        default_suffix.clone(),
                        Some(default_suffix),
                        worktree.selected_dir.clone(),
                        Some(worktree.selected_dir.clone()),
                    );
                    return OverlayKeyResult::Keep;
                }

                let branch = worktree
                    .entries
                    .iter()
                    .find(|entry| entry.path == item.id)
                    .and_then(|entry| entry.branch.clone());
                let final_dir = PathBuf::from(&item.id);

                match worktree.flow {
                    WorktreeFlow::NewSession => {
                        let default_name = default_session_name(&worktree.selected_dir, &final_dir);
                        self.open_session_name_overlay(
                            "Session".to_string(),
                            String::new(),
                            default_name.clone(),
                            Some(default_name),
                            final_dir,
                            None,
                        );
                    }
                    WorktreeFlow::NewWorktree => {
                        let (repo_name, default_suffix) =
                            worktree_name_parts(&worktree.selected_dir, branch.as_deref());
                        let title = if repo_name.is_empty() {
                            "Session".to_string()
                        } else {
                            format!("{repo_name}/")
                        };
                        let prefix = if repo_name.is_empty() {
                            String::new()
                        } else {
                            format!("{repo_name}/")
                        };
                        self.open_session_name_overlay(
                            title,
                            prefix,
                            default_suffix,
                            None,
                            final_dir,
                            None,
                        );
                    }
                }
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                move_picker_selection(&worktree.items, &mut worktree.selected, 1);
                OverlayKeyResult::Keep
            }
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                move_picker_selection(&worktree.items, &mut worktree.selected, -1);
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
        }
    }

    fn handle_session_name_key(
        &mut self,
        session: &mut SessionNameOverlay,
        key: KeyEvent,
    ) -> OverlayKeyResult {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => OverlayKeyResult::Close,
            (KeyCode::Enter, _) => {
                let raw = session.input.trim();
                let suffix = if raw.is_empty() {
                    session.default_on_empty.clone().unwrap_or_default()
                } else {
                    raw.to_string()
                };
                if suffix.is_empty() {
                    session.error = Some("session name required".to_string());
                    return OverlayKeyResult::Keep;
                }
                if let Some(selected_dir) = session.pending_worktree.clone() {
                    match create_new_worktree(&selected_dir, &suffix) {
                        Ok((path, _)) => {
                            session.final_dir = path;
                            session.pending_worktree = None;
                        }
                        Err(err) => {
                            session.error = Some(err);
                            return OverlayKeyResult::Keep;
                        }
                    }
                }
                let session_name = format!("{}{}", session.prefix, suffix);
                if session_exists(&session_name) {
                    tmux(&["switch-client", "-t", &format!("={session_name}")]);
                } else {
                    create_session_at_dir(&session_name, &session.final_dir);
                }
                handoff_to_main(self);
                OverlayKeyResult::Close
            }
            _ if handle_readline_key(&mut session.input, &mut session.cursor, key) => {
                session.error = None;
                OverlayKeyResult::Keep
            }
            _ => OverlayKeyResult::Unhandled,
        }
    }

    pub(super) fn open_rename_overlay(&mut self) {
        let Some(old_name) = self.selected_session_id() else {
            return;
        };
        let (prefix, suffix) = rename_parts(&old_name);
        self.overlay = Some(SidebarOverlay::Rename(RenameOverlay {
            old_name,
            prefix,
            input: suffix.clone(),
            cursor: suffix.len(),
            error: None,
        }));
    }

    pub(super) fn open_ditch_overlay(&mut self) {
        let Some(session) = self.selected_session_id() else {
            return;
        };
        let Some(plan) = build_ditch_plan(&session) else {
            return;
        };
        self.overlay = Some(SidebarOverlay::Ditch(ListOverlay {
            selected: first_selectable_picker(&plan.actions),
            offset: 0,
            items: plan.actions.clone(),
            error: None,
            plan: Some(plan),
        }));
    }

    pub(super) fn open_project_overlay(&mut self) {
        let items = build_project_items("all");
        self.overlay = Some(SidebarOverlay::Project(ProjectOverlay {
            filter: String::new(),
            cursor: 0,
            selected: first_selectable_picker(&items),
            offset: 0,
            all_items: items.clone(),
            items,
        }));
    }

    pub(super) fn open_worktree_overlay(&mut self) {
        let Some(target) = self.selected_session_id() else {
            return;
        };
        let Some(selected_dir) = resolve_selected_dir_from_session(Some(&target)) else {
            return;
        };
        let entries = list_worktrees(&selected_dir);
        if entries.is_empty() {
            return;
        }
        let (items, selected) = build_sidebar_worktree_items(&entries);
        self.overlay = Some(SidebarOverlay::Worktree(WorktreeOverlay {
            flow: WorktreeFlow::NewWorktree,
            selected_dir,
            entries,
            selected,
            offset: 0,
            items,
            error: None,
        }));
    }

    fn open_worktree_overlay_for_dir(&mut self, selected_dir: PathBuf, flow: WorktreeFlow) {
        let entries = list_worktrees(&selected_dir);
        if entries.is_empty() {
            let final_dir = selected_dir.clone();
            let default_name = default_session_name(&selected_dir, &final_dir);
            self.open_session_name_overlay(
                "Session".to_string(),
                String::new(),
                default_name.clone(),
                Some(default_name),
                final_dir,
                None,
            );
            return;
        }
        let (items, selected) = build_sidebar_worktree_items(&entries);
        self.overlay = Some(SidebarOverlay::Worktree(WorktreeOverlay {
            flow,
            selected_dir,
            entries,
            selected,
            offset: 0,
            items,
            error: None,
        }));
    }

    fn open_session_name_overlay(
        &mut self,
        title: String,
        prefix: String,
        initial: String,
        default_on_empty: Option<String>,
        final_dir: PathBuf,
        pending_worktree: Option<PathBuf>,
    ) {
        self.overlay = Some(SidebarOverlay::SessionName(SessionNameOverlay {
            title,
            prefix,
            cursor: initial.len(),
            input: initial,
            default_on_empty,
            final_dir,
            pending_worktree,
            error: None,
        }));
    }
}
