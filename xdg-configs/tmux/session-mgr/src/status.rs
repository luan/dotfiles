use std::collections::HashMap;

use crate::color::{compute_color, is_static};
use crate::group::{GroupMeta, session_group, session_suffix};

const NUM_SELECTED: &[char] = &[
    '\u{F0CA0}',
    '\u{F0CA2}',
    '\u{F0CA4}',
    '\u{F0CA6}',
    '\u{F0CA8}',
    '\u{F0CAA}',
    '\u{F0CAC}',
    '\u{F0CAE}',
    '\u{F0CB0}',
    '\u{F0FEC}',
];
const NUM_UNSELECTED: &[char] = &[
    '\u{F0CA1}',
    '\u{F0CA3}',
    '\u{F0CA5}',
    '\u{F0CA7}',
    '\u{F0CA9}',
    '\u{F0CAB}',
    '\u{F0CAD}',
    '\u{F0CAF}',
    '\u{F0CB1}',
    '\u{F0FED}',
];

const GROUP_SELECTED: &[char] = &[
    '\u{F0F0F}',
    '\u{F0F10}',
    '\u{F0F11}',
    '\u{F0F12}',
    '\u{F0F13}',
    '\u{F0F14}',
    '\u{F0F15}',
    '\u{F0F16}',
    '\u{F0F17}',
    '\u{F0FEA}',
];
const GROUP_UNSELECTED: &[char] = &[
    '\u{F03A5}',
    '\u{F03A8}',
    '\u{F03AB}',
    '\u{F03B2}',
    '\u{F03AF}',
    '\u{F03B4}',
    '\u{F03B7}',
    '\u{F03BA}',
    '\u{F03BD}',
    '\u{F03C0}',
];

fn num_glyph(idx: usize, selected: bool) -> char {
    let table = if selected {
        NUM_SELECTED
    } else {
        NUM_UNSELECTED
    };
    if idx < table.len() {
        table[idx]
    } else {
        table[table.len() - 1]
    }
}

fn group_glyph(count: usize, selected: bool) -> char {
    let table = if selected {
        GROUP_SELECTED
    } else {
        GROUP_UNSELECTED
    };
    let idx = count.clamp(1, table.len()) - 1;
    table[idx]
}

pub fn render_status(
    sessions: &[String],
    cur: &str,
    meta: &GroupMeta,
    attn: &HashMap<String, String>,
) -> (String, Vec<(String, String)>) {
    let mut gpos_counter: HashMap<&str, usize> = HashMap::new();
    let mut orphan_idx = 0usize;
    let mut prev_group = String::new();
    let mut out = String::new();
    let mut colors = Vec::new();

    // Pre-check: does the current session belong to any group?
    let cur_group_name = session_group(cur);

    // Pre-compute current session's dim color for group icon
    let cur_dim_hex = {
        let mut oi = 0usize;
        let mut gpc: HashMap<&str, usize> = HashMap::new();
        let mut found = String::new();
        for s in sessions {
            let g = session_group(s);
            if s == cur {
                let (_, d) = if is_static(s) {
                    compute_color(s, 0, 0, 0, 0)
                } else if !g.is_empty() {
                    let gp = *gpc.get(g).unwrap_or(&0);
                    let gi = *meta.group_idx.get(g).unwrap_or(&0);
                    let gt = *meta.counts.get(g).unwrap_or(&0);
                    compute_color(s, gi, meta.dynamic_total, gp, gt)
                } else {
                    compute_color(s, meta.dynamic_groups + oi, meta.dynamic_total, 0, 0)
                };
                found = d;
                break;
            }
            if !g.is_empty() {
                *gpc.entry(g).or_default() += 1;
            } else if !is_static(s) {
                oi += 1;
            }
        }
        found
    };

    for (idx, name) in sessions.iter().enumerate() {
        let group = session_group(name);
        let gtotal = if group.is_empty() {
            0
        } else {
            *meta.counts.get(group).unwrap_or(&0)
        };

        let (color, dim_c) = if is_static(name) {
            compute_color(name, 0, 0, 0, 0)
        } else if !group.is_empty() {
            let gpos = *gpos_counter.get(group).unwrap_or(&0);
            let gidx = *meta.group_idx.get(group).unwrap_or(&0);
            let r = compute_color(name, gidx, meta.dynamic_total, gpos, gtotal);
            *gpos_counter.entry(group).or_default() += 1;
            r
        } else {
            let r = compute_color(
                name,
                meta.dynamic_groups + orphan_idx,
                meta.dynamic_total,
                0,
                0,
            );
            orphan_idx += 1;
            r
        };

        colors.push((name.clone(), color.clone()));
        let a = attn.get(name.as_str()).map_or("", String::as_str);
        let display = if !group.is_empty() && gtotal == 1 {
            group
        } else if !group.is_empty() {
            session_suffix(name)
        } else {
            name.as_str()
        };
        let cur_group_key = if group.is_empty() {
            format!("__orphan__{name}")
        } else {
            group.to_string()
        };

        // Separator between groups
        if !prev_group.is_empty() && cur_group_key != prev_group {
            out.push_str(" #[fg=#585b70]│ ");
        } else if idx > 0 {
            out.push(' ');
        }

        // Group icon on first session of a multi-session group
        if !group.is_empty() && gtotal > 1 && cur_group_key != prev_group {
            let group_selected = cur_group_name == group;
            let gg = group_glyph(gtotal, group_selected);
            if group_selected {
                out.push_str(&format!("#[fg={cur_dim_hex}]{gg}#[fg=default] "));
            } else {
                out.push_str(&format!("#[fg=#585b70]{gg}#[fg=default] "));
            }
        }
        prev_group = cur_group_key;

        let glyph = num_glyph(idx, name == cur);

        if name == cur {
            out.push_str(&format!("#[fg={color}]{glyph} #[bold]{display}#[nobold]"));
        } else if a == "1" {
            out.push_str(&format!(
                "#[fg={dim_c}]{glyph} #[bold,fg={color}]●#[nobold] #[underscore,fg={dim_c}]{display}#[nounderscore]"
            ));
        } else {
            out.push_str(&format!("#[fg={dim_c}]{glyph} {display}"));
        }
    }

    out.push_str("#[default]");
    (out, colors)
}
