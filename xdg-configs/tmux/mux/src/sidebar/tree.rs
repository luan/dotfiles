use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use ratatui::prelude::*;

use crate::group::{GroupMeta, session_group, session_suffix};
use crate::palette::{group_glyph, hex_to_color, num_glyph};

use super::meta::{DiffStat, ProcessTreeInfo, SessionMeta};

pub(super) enum ItemKind {
    Session {
        diff: Option<DiffStat>,
        cpu_pct: f32,
        mem_bytes: u64,
    },
    Process(ProcessTreeInfo),
    Group,
    Branch,
    Agent {
        name: String,
        age: Option<Duration>,
        /// When Some, the agent is actively working — the gerund (e.g. "Churning…")
        /// drives the percolation text animation on this row.
        gerund: Option<String>,
        /// Context window: (pct, tokens) shown at row end.
        ctx: Option<(u8, String)>,
        /// Agent is waiting for user input.
        asking: bool,
    },
    Status,
    Progress(u8),
}

#[derive(Clone, Copy)]
pub(super) enum Tree {
    None,
    Middle, // ├ (session, not last in group)
    Last,   // └ (session, last in group)
    Pipe,   // │ (detail under non-last session)
    Blank,  // spaces (detail under last session)
}

pub(super) struct Item {
    pub(super) id: String,
    pub(super) display: String,
    pub(super) search_text: String,
    pub(super) indent: u16,
    pub(super) tree: Tree,
    pub(super) color: Color,
    pub(super) dim_color: Color,
    pub(super) selectable: bool,
    pub(super) session_id: Option<Rc<str>>,
    pub(super) kind: ItemKind,
}

pub(super) fn build_items(
    sessions: &[String],
    cur: &str,
    meta: &HashMap<String, SessionMeta>,
) -> Vec<Item> {
    let group_meta = GroupMeta::new(sessions);

    let color_list = crate::color::compute_session_colors(sessions, &group_meta);
    let session_colors: Vec<(Color, Color)> = color_list
        .iter()
        .map(|(_, c, d)| (hex_to_color(c), hex_to_color(d)))
        .collect();

    let empty_meta = SessionMeta::default();
    let mut items = Vec::with_capacity(estimate_item_capacity(sessions, meta, &group_meta));
    let session_refs: Vec<Rc<str>> = sessions
        .iter()
        .map(|session| Rc::<str>::from(session.as_str()))
        .collect();
    let mut idx = 0usize;
    let mut last_group = "";

    for (i, name) in sessions.iter().enumerate() {
        let session_id = session_refs[i].clone();
        let group = session_group(name);
        let gtotal = if group.is_empty() {
            0
        } else {
            *group_meta.counts.get(group).unwrap_or(&0)
        };
        let (color, dim_color) = session_colors[i];
        let sm = meta.get(name).unwrap_or(&empty_meta);

        let is_grouped = !group.is_empty() && gtotal > 1;
        let is_last_in_group =
            is_grouped && sessions.get(i + 1).map(|n| session_group(n)) != Some(group);
        let session_tree = if !is_grouped {
            Tree::None
        } else if is_last_in_group {
            Tree::Last
        } else {
            Tree::Middle
        };
        let detail_tree = if !is_grouped {
            Tree::None
        } else if is_last_in_group {
            Tree::Blank
        } else {
            Tree::Pipe
        };

        // Grouped session
        let (session_display, session_indent, detail_indent) = if is_grouped {
            if group != last_group {
                let gg = group_glyph(gtotal, false);
                items.push(Item {
                    id: format!("__group__{group}"),
                    display: format!("{gg} {group}"),
                    search_text: String::new(),
                    indent: 0,
                    tree: Tree::None,
                    color,
                    dim_color,
                    selectable: false,
                    session_id: None,
                    kind: ItemKind::Group,
                });
            }
            let suffix = {
                let s = session_suffix(name);
                if s.is_empty() {
                    group.to_string()
                } else {
                    s.to_string()
                }
            };
            let glyph = num_glyph(idx, name == cur);
            idx += 1;
            (format!("{glyph} {suffix}"), 2u16, 4u16)
        } else {
            let flat = if !group.is_empty() {
                group
            } else {
                name.as_str()
            };
            let glyph = num_glyph(idx, name == cur);
            idx += 1;
            (format!("{glyph} {flat}"), 0u16, 2u16)
        };

        items.push(Item {
            id: name.clone(),
            search_text: format!("{session_display} {name}"),
            display: session_display,
            indent: session_indent,
            tree: session_tree,
            color,
            dim_color,
            selectable: true,
            session_id: Some(session_id.clone()),
            kind: ItemKind::Session {
                diff: sm.diff,
                cpu_pct: sm.cpu_pct,
                mem_bytes: sm.mem_bytes,
            },
        });

        // Detail rows (all indented to align after number glyph)
        for (pi, process) in sm.processes.iter().enumerate() {
            items.push(Item {
                id: format!("__process__{name}__{pi}"),
                display: String::new(),
                search_text: String::new(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(session_id.clone()),
                kind: ItemKind::Process(process.clone()),
            });
        }
        for (ai, agent) in sm.agents.iter().enumerate() {
            items.push(Item {
                id: format!("__agent__{name}__{ai}"),
                display: String::new(),
                search_text: String::new(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(session_id.clone()),
                kind: ItemKind::Agent {
                    name: agent.name.clone(),
                    age: agent.age,
                    gerund: agent.gerund.clone(),
                    ctx: agent.ctx.as_ref().map(|c| (c.pct, c.tokens.clone())),
                    asking: agent.asking,
                },
            });
        }
        if !sm.branch.is_empty() {
            items.push(Item {
                id: format!("__branch__{name}"),
                display: sm.branch.clone(),
                search_text: String::new(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(session_id.clone()),
                kind: ItemKind::Branch,
            });
        }
        if !sm.status.is_empty() {
            items.push(Item {
                id: format!("__status__{name}"),
                display: sm.status.clone(),
                search_text: String::new(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(session_id.clone()),
                kind: ItemKind::Status,
            });
        }
        if let Some(pct) = sm.progress {
            items.push(Item {
                id: format!("__progress__{name}"),
                display: String::new(),
                search_text: String::new(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(session_id),
                kind: ItemKind::Progress(pct),
            });
        }

        last_group = group;
    }

    items
}

fn estimate_item_capacity(
    sessions: &[String],
    meta: &HashMap<String, SessionMeta>,
    group_meta: &GroupMeta,
) -> usize {
    let empty_meta = SessionMeta::default();
    let mut total = 0usize;
    let mut last_group = "";

    for name in sessions {
        let group = session_group(name);
        let group_total = if group.is_empty() {
            0
        } else {
            *group_meta.counts.get(group).unwrap_or(&0)
        };
        let is_grouped = !group.is_empty() && group_total > 1;
        if is_grouped && group != last_group {
            total += 1;
        }

        let sm = meta.get(name).unwrap_or(&empty_meta);
        total += 1; // session
        total += sm.processes.len();
        total += sm.agents.len();
        total += usize::from(!sm.branch.is_empty());
        total += usize::from(!sm.status.is_empty());
        total += usize::from(sm.progress.is_some());

        last_group = group;
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidebar::meta::{AgentInstance, ProcessTreeInfo};

    #[test]
    fn build_items_preserves_group_session_and_detail_order() {
        let sessions = vec![
            "work/api".to_string(),
            "work/ui".to_string(),
            "ops".to_string(),
        ];
        let mut meta = HashMap::new();
        meta.insert(
            "work/api".to_string(),
            SessionMeta {
                processes: vec![ProcessTreeInfo {
                    name: "cargo".to_string(),
                    cpu_pct: 1.0,
                    mem_bytes: 1024,
                }],
                agents: vec![AgentInstance {
                    name: "claude".to_string(),
                    pane_id: "%1".to_string(),
                    gerund: Some("Testing…".to_string()),
                    ctx: None,
                    age: None,
                    asking: false,
                }],
                status: "review needed".to_string(),
                progress: Some(42),
                ..SessionMeta::default()
            },
        );

        let items = build_items(&sessions, "work/api", &meta);

        assert!(matches!(items[0].kind, ItemKind::Group));
        assert_eq!(items[1].id, "work/api");
        assert!(matches!(items[1].kind, ItemKind::Session { .. }));
        assert_eq!(items[1].session_id.as_deref(), Some("work/api"));
        assert!(matches!(items[2].kind, ItemKind::Process(_)));
        assert_eq!(items[2].session_id.as_deref(), Some("work/api"));
        assert!(matches!(items[3].kind, ItemKind::Agent { .. }));
        assert_eq!(items[3].session_id.as_deref(), Some("work/api"));
        assert!(
            items
                .iter()
                .any(|item| matches!(item.kind, ItemKind::Status))
        );
        assert!(
            items
                .iter()
                .any(|item| matches!(item.kind, ItemKind::Progress(42)))
        );
    }

    #[test]
    fn build_items_uses_exact_capacity_estimate() {
        let sessions = vec!["work/api".to_string(), "work/ui".to_string()];
        let mut meta = HashMap::new();
        meta.insert(
            "work/api".to_string(),
            SessionMeta {
                agents: vec![AgentInstance {
                    name: "codex".to_string(),
                    pane_id: "%2".to_string(),
                    gerund: None,
                    ctx: None,
                    age: None,
                    asking: false,
                }],
                progress: Some(7),
                ..SessionMeta::default()
            },
        );

        let items = build_items(&sessions, "work/api", &meta);

        assert_eq!(items.len(), items.capacity());
    }
}
