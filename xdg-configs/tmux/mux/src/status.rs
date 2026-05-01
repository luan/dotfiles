use crate::group::{GroupMeta, session_group, session_suffix};
use crate::palette::{group_glyph, num_glyph};
use crate::tmux::{SystemInfo, WindowInfo};

// Catppuccin Mocha palette (hex strings for tmux format strings)
const SURFACE0: &str = "#141421";
const SURFACE1: &str = "#272738";
const PILL_A: &str = "#272738";
const PILL_B: &str = "#141421";
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

pub(crate) struct BarOutput {
    pub(crate) left: String,
    pub(crate) colors: Vec<(String, String)>,
}

pub(crate) fn render_bar(
    sessions: &[String],
    cur_session: &str,
    meta: &GroupMeta,
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
        render_sessions_full(sessions, cur_session, meta, &colors, compact)
    };

    BarOutput {
        left: format!(" {session_str} "),
        colors: color_map,
    }
}

const ARROW_RIGHT: char = '\u{E0B0}'; //

/// Blend a `#rrggbb` hex toward CRUST so active-window session colors don't
/// flashbang behind the window number.
fn dim_hex(hex: &str) -> String {
    let bytes = hex.trim_start_matches('#');
    if bytes.len() != 6 {
        return hex.to_string();
    }
    let Ok(r) = u8::from_str_radix(&bytes[0..2], 16) else {
        return hex.to_string();
    };
    let Ok(g) = u8::from_str_radix(&bytes[2..4], 16) else {
        return hex.to_string();
    };
    let Ok(b) = u8::from_str_radix(&bytes[4..6], 16) else {
        return hex.to_string();
    };
    // 60% session color + 40% CRUST
    let mix = |v: u8, t: u8| ((v as u32 * 60 + t as u32 * 40) / 100) as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        mix(r, 0x11),
        mix(g, 0x11),
        mix(b, 0x1b)
    )
}

pub(crate) fn render_windows(windows: &[WindowInfo], cur_color: &str) -> String {
    if windows.is_empty() {
        return String::new();
    }

    let cur_bg = dim_hex(cur_color);
    let text_bg_for = |active: bool| if active { SURFACE1 } else { SURFACE0 };

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

        // Left cap merges with previous window's text pill.
        let left_bg: &str = if is_first {
            "default"
        } else {
            text_bg_for(windows[i - 1].active)
        };

        let text_bg = text_bg_for(w.active);
        let (num_fg, num_bg): (&str, &str) = if w.active {
            (CRUST, cur_bg.as_str())
        } else {
            (OVERLAY2, SURFACE0)
        };

        out.push_str(&format!(
            "#[range=user|w:{}]#[fg={num_bg},bg={left_bg}]{PL_LEFT}#[fg={num_fg},bg={num_bg}] {} #[fg={num_bg},bg={text_bg}]{ARROW_RIGHT}#[fg={TEXT},bg={text_bg}] {display} ",
            w.index, w.index,
        ));

        // Right edge: last window gets PL_RIGHT, others flat
        if is_last {
            out.push_str(&format!("#[fg={text_bg},bg=default]{PL_RIGHT}"));
        }

        out.push_str("#[norange]");
    }

    out.push_str("#[default]");
    out
}

pub(crate) fn render_windows_centered_in_main(
    rendered_windows: &str,
    client_width: usize,
    sidebar_offset: usize,
) -> String {
    if rendered_windows.is_empty() {
        return String::new();
    }

    let main_width = client_width.saturating_sub(sidebar_offset);
    if main_width == 0 {
        return rendered_windows.to_string();
    }

    let win_width = tmux_visible_width(rendered_windows);
    let pad = sidebar_offset + main_width.saturating_sub(win_width) / 2;
    format!("{}{}", " ".repeat(pad), rendered_windows)
}

pub(crate) fn render_windows_left_of_notch(
    rendered_windows: &str,
    client_width: usize,
    sidebar_offset: usize,
    notch_width: usize,
) -> String {
    if rendered_windows.is_empty() {
        return String::new();
    }

    let win_width = tmux_visible_width(rendered_windows);
    let notch_left = client_width.saturating_sub(notch_width) / 2;
    let safe_end = notch_left.saturating_sub(2);

    let pad = if safe_end <= sidebar_offset || win_width >= safe_end.saturating_sub(sidebar_offset)
    {
        sidebar_offset
    } else {
        safe_end - win_width
    };

    format!("{}{}", " ".repeat(pad), rendered_windows)
}

fn tmux_visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '#' && chars.peek() == Some(&'[') {
            chars.next();
            for inner in chars.by_ref() {
                if inner == ']' {
                    break;
                }
            }
            continue;
        }

        width += 1;
    }

    width
}

// ── Sessions ──────────────────────────────────────────────────

pub(crate) fn compute_all_colors(
    sessions: &[String],
    meta: &GroupMeta,
) -> Vec<(String, String, String)> {
    crate::color::compute_session_colors(sessions, meta)
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
        } else if show_name {
            out.push_str(&format!("#[fg={dim_c}]{glyph}   {display}"));
        } else {
            out.push_str(&format!("#[fg={dim_c}]{glyph} "));
        }

        out.push_str("#[norange]");
    }

    out.push_str("#[default]");
    out
}

// ── System Info ───────────────────────────────────────────────

// Diagonal powerline-extra glyph between alternating-bg sections.
const DIAG: char = '\u{E0B8}';

fn colored(text: &str, color: &str) -> String {
    format!("#[fg={color}]{text}")
}

pub(crate) fn render_system_info(info: &SystemInfo) -> String {
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
        &format!(" \u{F4BC} {:.1} ", info.cpu_load),
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
    sections.push(colored(&format!(" \u{EFC5} {}% ", info.mem_pct), mem_color));

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
                        format!(" {} ", info.battery_time)
                    } else {
                        " ".to_string()
                    };
                sections.push(format!(
                    "{}{}",
                    colored(&format!(" {icon} {pct}%"), color),
                    colored(&time_str, OVERLAY2),
                ));
            }
        }
    }

    // Date + Clock
    sections.push(format!(
        "#[fg={OVERLAY2},italics] {} #[noitalics]#[fg={TEXT}]\u{F0954} {} ",
        info.date, info.clock
    ));

    // Assemble sections with alternating pill backgrounds and a diagonal
    // separator between each pair — replaces the drawn vertical divider with
    // a shape that carries the grouping information on its own. Anchor on the
    // last section so the rightmost pill is always PILL_B; battery presence
    // otherwise flips the colour pattern across the whole bar.
    let n = sections.len();
    let bg_for = |i: usize| {
        if (n - 1 - i).is_multiple_of(2) {
            PILL_B
        } else {
            PILL_A
        }
    };
    let first_bg = bg_for(0);

    let mut out = format!("#[fg={first_bg},bg=default]{PL_LEFT}");
    for (i, section) in sections.iter().enumerate() {
        let bg = bg_for(i);
        out.push_str(&format!("#[bg={bg}]{section}"));
        if i + 1 < sections.len() {
            let next_bg = bg_for(i + 1);
            out.push_str(&format!("#[fg={bg},bg={next_bg}]{DIAG}"));
        }
    }
    // No right cap: the rounded glyph leaves visible default-bg space in its
    // unfilled half, which reads as an awkward gap between the last pill and
    // the terminal edge. Let the pill end flush with its trailing padding.
    out.push_str("#[default]");
    out
}
