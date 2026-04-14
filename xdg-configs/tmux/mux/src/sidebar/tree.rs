use std::collections::HashMap;
use std::time::Duration;

use ratatui::prelude::*;

use crate::group::{GroupMeta, session_group, session_suffix};
use crate::palette::{group_glyph, hex_to_color, num_glyph};

use super::meta::SessionMeta;

pub(super) enum ItemKind {
    Session { attention: bool },
    Group,
    Branch,
    Agent { name: String, age: Option<Duration> },
    Activity(String),
    ContextBar { pct: u8, tokens: String },
    Ports(Vec<u16>),
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
    pub(super) indent: u16,
    pub(super) tree: Tree,
    pub(super) color: Color,
    pub(super) dim_color: Color,
    pub(super) selectable: bool,
    pub(super) session_id: Option<String>,
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
    let mut items = Vec::new();
    let mut idx = 0usize;
    let mut last_group = String::new();

    for (i, name) in sessions.iter().enumerate() {
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
            display: session_display,
            indent: session_indent,
            tree: session_tree,
            color,
            dim_color,
            selectable: true,
            session_id: Some(name.clone()),
            kind: ItemKind::Session {
                attention: sm.attention,
            },
        });

        // Detail rows (all indented to align after number glyph)
        if !sm.agent.is_empty() {
            items.push(Item {
                id: format!("__agent__{name}"),
                display: sm.agent.clone(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Agent {
                    name: sm.agent.clone(),
                    age: sm.claude_age,
                },
            });

            if let Some(act) = &sm.claude_activity {
                items.push(Item {
                    id: format!("__activity__{name}"),
                    display: act.clone(),
                    indent: detail_indent,
                    tree: detail_tree,
                    color,
                    dim_color,
                    selectable: false,
                    session_id: Some(name.clone()),
                    kind: ItemKind::Activity(act.clone()),
                });
            }

            if let Some(ctx) = &sm.claude_ctx {
                items.push(Item {
                    id: format!("__ctx__{name}"),
                    display: String::new(),
                    indent: detail_indent,
                    tree: detail_tree,
                    color,
                    dim_color,
                    selectable: false,
                    session_id: Some(name.clone()),
                    kind: ItemKind::ContextBar {
                        pct: ctx.pct,
                        tokens: ctx.tokens.clone(),
                    },
                });
            }
        }
        if !sm.branch.is_empty() {
            items.push(Item {
                id: format!("__branch__{name}"),
                display: sm.branch.clone(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Branch,
            });
        }
        if !sm.ports.is_empty() {
            items.push(Item {
                id: format!("__ports__{name}"),
                display: String::new(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Ports(sm.ports.clone()),
            });
        }
        if !sm.status.is_empty() {
            items.push(Item {
                id: format!("__status__{name}"),
                display: sm.status.clone(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Status,
            });
        }
        if let Some(pct) = sm.progress {
            items.push(Item {
                id: format!("__progress__{name}"),
                display: String::new(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Progress(pct),
            });
        }

        last_group = group.to_string();
    }

    items
}
