use std::collections::HashMap;

use crate::color::{compute_color, is_static};
use crate::group::{GroupMeta, session_group, session_suffix};

const NUM_SELECTED: &[char] = &[
    '\u{F03A4}', // 1
    '\u{F03A7}', // 2
    '\u{F03AA}', // 3
    '\u{F03AD}', // 4
    '\u{F03B1}', // 5
    '\u{F03B3}', // 6
    '\u{F03B6}', // 7
    '\u{F03B9}', // 8
    '\u{F03BC}', // 9
    '\u{F03BF}', // 9+
];
const NUM_UNSELECTED: &[char] = &[
    '\u{F03A6}', // 1
    '\u{F03A9}', // 2
    '\u{F03AC}', // 3
    '\u{F03AE}', // 4
    '\u{F03B0}', // 5
    '\u{F03B5}', // 6
    '\u{F03B8}', // 7
    '\u{F03BB}', // 8
    '\u{F03BE}', // 9
    '\u{F03C1}', // 9+
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
    width: usize,
) -> (String, Vec<(String, String)>) {
    if sessions.is_empty() {
        return (String::from("#[default]"), Vec::new());
    }

    let colors = compute_all_colors(sessions, meta);
    let color_map: Vec<(String, String)> = colors
        .iter()
        .map(|(n, c, _)| (n.clone(), c.clone()))
        .collect();

    let out = if width < 60 {
        render_narrow(sessions, cur, &colors, width)
    } else {
        let compact = width < 120;
        render_full(sessions, cur, meta, attn, &colors, compact)
    };

    (out, color_map)
}

fn compute_all_colors(sessions: &[String], meta: &GroupMeta) -> Vec<(String, String, String)> {
    let mut gpos_counter: HashMap<&str, usize> = HashMap::new();
    let mut orphan_idx = 0usize;
    let mut result = Vec::new();

    for name in sessions {
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

        result.push((name.clone(), color, dim_c));
    }

    result
}

/// Narrow (<60 cols): current session only + position count
fn render_narrow(
    sessions: &[String],
    cur: &str,
    colors: &[(String, String, String)],
    width: usize,
) -> String {
    let total = sessions.len();
    let cur_idx = sessions.iter().position(|s| s == cur).unwrap_or(0);
    let (_, ref color, _) = colors[cur_idx];

    let display = session_group(cur);
    let glyph = num_glyph(cur_idx, true);

    let mut out = format!("#[fg={color}]{glyph} #[bold]{display}#[nobold]");

    if total > 1 {
        let count_str = format!(" [{}/{}]", cur_idx + 1, total);
        let est = 3 + display.len() + count_str.len();
        if est <= width {
            out.push_str(&format!("#[fg=#585b70]{count_str}"));
        }
    }

    out.push_str("#[default]");
    out
}

/// Full rendering with optional compact mode.
/// compact=false (wide, ≥120): all sessions with names
/// compact=true (medium, 60–119): non-current-group sessions show glyph only
fn render_full(
    sessions: &[String],
    cur: &str,
    meta: &GroupMeta,
    attn: &HashMap<String, String>,
    colors: &[(String, String, String)],
    compact: bool,
) -> String {
    let cur_group_name = session_group(cur);
    let cur_dim_hex = sessions
        .iter()
        .position(|s| s == cur)
        .map_or_else(String::new, |i| colors[i].2.clone());

    let mut prev_group = String::new();
    let mut out = String::new();

    for (idx, name) in sessions.iter().enumerate() {
        let group = session_group(name);
        let gtotal = if group.is_empty() {
            0
        } else {
            *meta.counts.get(group).unwrap_or(&0)
        };

        let (_, ref color, ref dim_c) = colors[idx];
        let a = attn.get(name.as_str()).map_or("", String::as_str);
        let display = if gtotal == 1 {
            group
        } else {
            let s = session_suffix(name);
            if s.is_empty() { group } else { s }
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
        let show_name = !compact || name == cur || (!group.is_empty() && group == cur_group_name);

        if name == cur {
            out.push_str(&format!("#[fg={color}]{glyph} #[bold]{display}#[nobold]"));
        } else if show_name && a == "1" {
            out.push_str(&format!(
                "#[fg={dim_c}]{glyph} #[bold,fg={color}]●#[nobold] #[underscore,fg={dim_c}]{display}#[nounderscore]"
            ));
        } else if show_name {
            out.push_str(&format!("#[fg={dim_c}]{glyph} {display}"));
        } else if a == "1" {
            out.push_str(&format!("#[fg={dim_c}]{glyph}#[bold,fg={color}]●#[nobold]"));
        } else {
            out.push_str(&format!("#[fg={dim_c}]{glyph}"));
        }
    }

    out.push_str("#[default]");
    out
}
