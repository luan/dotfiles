use std::collections::HashMap;

use crate::color::{compute_color, is_static};
use crate::group::{GroupMeta, session_group, session_suffix};
use crate::tmux::{SystemInfo, WindowInfo};

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

// Catppuccin Mocha palette
const SURFACE0: &str = "#313244";
const SURFACE1: &str = "#45475a";
const OVERLAY2: &str = "#9399b2";
const SUBTEXT0: &str = "#a6adc8";
const TEXT: &str = "#cdd6f4";
const CRUST: &str = "#11111b";
const GREEN: &str = "#a6e3a1";
const YELLOW: &str = "#f9e2af";
const RED: &str = "#f38ba8";
// Powerline rounded separators
const PL_LEFT: char = '\u{E0B6}'; //
const PL_RIGHT: char = '\u{E0B4}'; //

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

pub struct BarOutput {
    pub left: String,
    pub colors: Vec<(String, String)>,
}

pub fn render_bar(
    sessions: &[String],
    cur_session: &str,
    meta: &GroupMeta,
    attn: &HashMap<String, String>,
    width: usize,
) -> BarOutput {
    if sessions.is_empty() {
        return BarOutput {
            left: String::from("#[default]"),
            colors: Vec::new(),
        };
    }

    let colors = compute_all_colors(sessions, meta);
    let color_map: Vec<(String, String)> = colors
        .iter()
        .map(|(n, c, _)| (n.clone(), c.clone()))
        .collect();

    let session_str = if width < 60 {
        render_sessions_narrow(sessions, cur_session, &colors, width)
    } else {
        let compact = width < 120;
        render_sessions_full(sessions, cur_session, meta, attn, &colors, compact)
    };

    BarOutput {
        left: format!(" {session_str} "),
        colors: color_map,
    }
}

const ARROW_RIGHT: char = '\u{E0B0}'; //

pub fn render_windows(windows: &[WindowInfo], cur_color: &str) -> String {
    if windows.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    let last = windows.len() - 1;

    for (i, w) in windows.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == last;

        let display = if w.zoomed {
            format!("{}()", w.name)
        } else {
            w.name.clone()
        };

        // Left cap: first window gets bg=default, others merge with previous SURFACE1
        let left_bg = if is_first { "default" } else { SURFACE1 };

        // Number section colors
        let (num_fg, num_bg) = if w.active {
            (CRUST, cur_color)
        } else {
            (OVERLAY2, SURFACE0)
        };

        out.push_str(&format!(
            "#[range=user|w:{}]#[fg={num_bg},bg={left_bg}]{PL_LEFT}#[fg={num_fg},bg={num_bg}] {} #[fg={num_bg},bg={SURFACE1}]{ARROW_RIGHT}#[fg={TEXT},bg={SURFACE1}] {display} ",
            w.index, w.index,
        ));

        // Right edge: last window gets PL_RIGHT, others flat
        if is_last {
            out.push_str(&format!("#[fg={SURFACE1},bg=default]{PL_RIGHT}"));
        }

        out.push_str("#[norange]");
    }

    out.push_str("#[default]");
    out
}

// ── Sessions ──────────────────────────────────────────────────

pub fn compute_all_colors(sessions: &[String], meta: &GroupMeta) -> Vec<(String, String, String)> {
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

fn render_sessions_narrow(
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

    let mut out =
        format!("#[range=user|s:{cur}]#[fg={color}]{glyph} #[bold]{display}#[nobold]#[norange]");

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

fn render_sessions_full(
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

        // Wrap each session in a range for click handling
        out.push_str(&format!("#[range=user|s:{name}]"));

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

        out.push_str("#[norange]");
    }

    out.push_str("#[default]");
    out
}

// ── System Info ───────────────────────────────────────────────

const SEP: &str = "#[fg=#585b70]│#[fg=default]";

fn colored(text: &str, color: &str) -> String {
    format!("#[fg={color}]{text}")
}

pub fn render_system_info(info: &SystemInfo) -> String {
    use crate::tmux::BatteryState;

    let mut sections: Vec<String> = Vec::new();

    // Caffeine (on = yellow filled cup, off = gray outline)
    let (caf_icon, caf_color) = if info.caffeinated {
        ("\u{F0176}", YELLOW) // 󰅶 md-coffee (on)
    } else {
        ("\u{F06CA}", OVERLAY2) // 󰛊 md-coffee-outline (off)
    };
    sections.push(format!(
        "#[range=user|caffeine]{}#[norange]",
        colored(&format!(" {caf_icon} "), caf_color)
    ));

    // CPU
    let cpu_color = if info.cpu_load > 10.0 {
        RED
    } else if info.cpu_load > 4.0 {
        YELLOW
    } else {
        SUBTEXT0
    };
    sections.push(colored(
        &format!(" \u{F0EE0} {:.1} ", info.cpu_load),
        cpu_color,
    ));

    // Memory
    let mem_color = if info.mem_pct > 90 {
        RED
    } else if info.mem_pct > 70 {
        YELLOW
    } else {
        SUBTEXT0
    };
    sections.push(colored(
        &format!(" \u{F035B} {}% ", info.mem_pct),
        mem_color,
    ));

    // Battery
    if let Some(pct) = info.battery_pct {
        match info.battery_state {
            BatteryState::Charged => {
                sections.push(colored(" \u{F0E7} ", GREEN));
            }
            _ => {
                let (icon, color) = match info.battery_state {
                    BatteryState::Charging => ('\u{F0084}', GREEN),
                    _ if pct > 75 => ('\u{F0079}', SUBTEXT0),
                    _ if pct > 50 => ('\u{F007E}', SUBTEXT0),
                    _ if pct > 25 => ('\u{F007A}', YELLOW),
                    _ => ('\u{F008E}', RED),
                };
                let time_str =
                    if !info.battery_time.is_empty() && !info.battery_time.starts_with('(') {
                        format!(" {}", info.battery_time)
                    } else {
                        String::new()
                    };
                sections.push(format!(
                    "{}{}",
                    colored(&format!(" {icon} {pct}%"), color),
                    colored(&time_str, OVERLAY2),
                ));
            }
        }
    }

    // Clock
    sections.push(colored(&format!(" \u{F0954} {} ", info.clock), TEXT));

    // Wrap all sections in one continuous pill with thin separators
    let inner = sections.join(SEP);
    format!(
        "#[fg={SURFACE1}]{PL_LEFT}#[bg={SURFACE1}]{inner}#[none,fg={SURFACE1}]{PL_RIGHT}#[default]"
    )
}
