use std::collections::HashMap;
use std::time::Duration;

use ratatui::prelude::*;

use crate::group::{GroupMeta, session_group, session_suffix};
use crate::palette::{group_glyph, hex_to_color, num_glyph};

use super::meta::{DiffStat, ProcessTreeInfo, PullRequestCheck, PullRequestMeta, SessionMeta};

pub(super) enum ItemKind {
    Session {
        diff: Option<DiffStat>,
        cpu_pct: f32,
        mem_bytes: u64,
    },
    Process(ProcessTreeInfo),
    Group,
    Branch {
        pr: Option<PullRequestMeta>,
    },
    PullRequestUnresolved {
        count: u32,
        pr: PullRequestMeta,
    },
    PullRequestCheck {
        check: PullRequestCheck,
        pr: PullRequestMeta,
    },
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
                diff: sm.diff,
                cpu_pct: sm.cpu_pct,
                mem_bytes: sm.mem_bytes,
            },
        });

        // Detail rows (all indented to align after number glyph)
        for (pi, process) in sm.processes.iter().enumerate() {
            items.push(Item {
                id: format!("__process__{name}__{pi}"),
                display: process.name.clone(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Process(process.clone()),
            });
        }
        for (ai, agent) in sm.agents.iter().enumerate() {
            items.push(Item {
                id: format!("__agent__{name}__{ai}"),
                display: agent.name.clone(),
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
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
                indent: detail_indent,
                tree: detail_tree,
                color,
                dim_color,
                selectable: false,
                session_id: Some(name.clone()),
                kind: ItemKind::Branch { pr: sm.pr.clone() },
            });
        }
        if let Some(pr) = &sm.pr {
            if pr.unresolved_comments > 0 {
                items.push(Item {
                    id: format!("__pr_unresolved__{name}"),
                    display: format!("{} unresolved", pr.unresolved_comments),
                    indent: detail_indent,
                    tree: detail_tree,
                    color,
                    dim_color,
                    selectable: false,
                    session_id: Some(name.clone()),
                    kind: ItemKind::PullRequestUnresolved {
                        count: pr.unresolved_comments,
                        pr: pr.clone(),
                    },
                });
            }
            if name == cur {
                for (ci, check) in pr.checks.iter().enumerate() {
                    items.push(Item {
                        id: format!("__pr_check__{name}__{ci}"),
                        display: check.name.clone(),
                        indent: detail_indent,
                        tree: detail_tree,
                        color,
                        dim_color,
                        selectable: false,
                        session_id: Some(name.clone()),
                        kind: ItemKind::PullRequestCheck {
                            check: check.clone(),
                            pr: pr.clone(),
                        },
                    });
                }
            }
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
