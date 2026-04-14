use ratatui::prelude::*;

use crate::palette::{OVERLAY0, TEXT};

use super::super::{KEY_CMD, KEY_CTRL, KEY_OPT, KEY_SHIFT, KEY_TAB};

pub(in crate::sidebar) fn hint_key(k: impl Into<std::borrow::Cow<'static, str>>) -> Span<'static> {
    Span::styled(k, Style::default().fg(TEXT).bold().italic())
}

pub(in crate::sidebar) fn hint_lbl(s: impl Into<String>) -> Span<'static> {
    Span::styled(s.into(), Style::default().fg(OVERLAY0).italic())
}

pub(in crate::sidebar) fn hint_sep() -> Span<'static> {
    Span::styled("  ", Style::default())
}

pub(in crate::sidebar) fn footer_hints(width: usize, show_hidden: bool) -> Vec<Line<'static>> {
    let opt_jk = format!("{KEY_OPT} jk");
    let h_short = if show_hidden { " sho" } else { " hid" };
    let h_long = if show_hidden { " show" } else { " hide" };

    if width >= 34 {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key("n"),
                hint_lbl(" new"),
                hint_sep(),
                hint_key("w"),
                hint_lbl(" work"),
                hint_sep(),
                hint_key("r"),
                hint_lbl(" ren"),
            ]),
            Line::from(vec![
                hint_lbl(" "),
                hint_key("x"),
                hint_lbl(" del"),
                hint_sep(),
                hint_key("h"),
                hint_lbl(h_short.to_string()),
                hint_sep(),
                hint_key(opt_jk),
                hint_lbl(" mv"),
                hint_sep(),
                hint_key("q"),
                hint_lbl(" close"),
            ]),
        ]
    } else if width >= 22 {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key("n"),
                hint_lbl(" new"),
                hint_sep(),
                hint_key("w"),
                hint_lbl(" wt"),
                hint_sep(),
                hint_key("r"),
                hint_lbl(" ren"),
            ]),
            Line::from(vec![
                hint_lbl(" "),
                hint_key("x"),
                hint_lbl(" del"),
                hint_sep(),
                hint_key("h"),
                hint_lbl(h_long.to_string()),
                hint_sep(),
                hint_key("q"),
                hint_lbl(" close"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key("n"),
                hint_lbl(" "),
                hint_key("w"),
                hint_lbl(" "),
                hint_key("r"),
                hint_lbl(" "),
                hint_key("x"),
                hint_lbl(" "),
                hint_key("h"),
            ]),
            Line::from(vec![hint_lbl(" "), hint_key("q"), hint_lbl(" close")]),
        ]
    }
}

pub(in crate::sidebar) fn chooser_footer_hints(width: usize, show_hidden: bool) -> Vec<Line<'static>> {
    let opt_h = format!("{KEY_OPT} h");
    let opt_jk = format!("{KEY_OPT} jk");
    let h_long = if show_hidden { " show" } else { " hide" };

    if width >= 34 {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key("↵"),
                hint_lbl(" jump"),
                hint_sep(),
                hint_key("/text"),
                hint_lbl(" search"),
                hint_sep(),
                hint_key("esc"),
                hint_lbl(" done"),
            ]),
            Line::from(vec![
                hint_lbl(" "),
                hint_key(opt_h),
                hint_lbl(h_long.to_string()),
                hint_sep(),
                hint_key(opt_jk),
                hint_lbl(" move"),
                hint_sep(),
                hint_key("q"),
                hint_lbl(" close"),
            ]),
        ]
    } else if width >= 22 {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key("↵"),
                hint_lbl(" jump"),
                hint_sep(),
                hint_key("esc"),
                hint_lbl(" done"),
            ]),
            Line::from(vec![
                hint_lbl(" "),
                hint_key(opt_h),
                hint_lbl(h_long.to_string()),
                hint_sep(),
                hint_key("q"),
                hint_lbl(" close"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key("↵"),
                hint_lbl(" "),
                hint_key("esc"),
            ]),
            Line::from(vec![hint_lbl(" "), hint_key("q"), hint_lbl(" close")]),
        ]
    }
}

pub(in crate::sidebar) fn overlay_footer_hints(width: usize) -> Vec<Line<'static>> {
    if width >= 34 {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key("↵"),
                hint_lbl(" use"),
                hint_sep(),
                hint_key("esc"),
                hint_lbl(" back"),
            ]),
            Line::from(vec![hint_lbl(" ")]),
        ]
    } else {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key("↵"),
                hint_lbl(" "),
                hint_key("esc"),
            ]),
            Line::from(vec![hint_lbl(" ")]),
        ]
    }
}

pub(in crate::sidebar) fn unfocused_footer_hints(width: usize) -> Vec<Line<'static>> {
    let ctrl_tab = format!("{KEY_CTRL} {KEY_TAB}");
    let cmd_o = format!("{KEY_CMD} O");
    let cmd_p = format!("{KEY_CMD} P");
    let cmd_n = format!("{KEY_CMD} N");
    let cmd_semi = format!("{KEY_CMD} ;");
    let cmd_shift_np = format!("{KEY_CMD} {KEY_SHIFT} NP");

    if width >= 34 {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key(cmd_o.clone()),
                hint_lbl(" focus"),
                hint_sep(),
                hint_key(ctrl_tab),
                hint_lbl(" last"),
            ]),
            Line::from(vec![
                hint_lbl(" "),
                hint_key(cmd_p),
                hint_lbl(" pick"),
                hint_sep(),
                hint_key(cmd_n),
                hint_lbl(" new"),
                hint_sep(),
                hint_key(cmd_semi),
                hint_lbl(" attn"),
            ]),
            Line::from(vec![
                hint_lbl(" "),
                hint_key(cmd_shift_np),
                hint_lbl(" next/prev"),
            ]),
        ]
    } else if width >= 22 {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key(cmd_o.clone()),
                hint_lbl(" focus"),
                hint_sep(),
                hint_key(ctrl_tab),
                hint_lbl(" last"),
            ]),
            Line::from(vec![
                hint_lbl(" "),
                hint_key(cmd_p),
                hint_lbl(" pick"),
                hint_sep(),
                hint_key(cmd_n),
                hint_lbl(" new"),
            ]),
            Line::from(vec![hint_lbl(" "), hint_key(cmd_semi), hint_lbl(" attn")]),
        ]
    } else {
        vec![
            Line::from(vec![
                hint_lbl(" "),
                hint_key(cmd_o.clone()),
                hint_lbl(" "),
                hint_key(ctrl_tab),
            ]),
            Line::from(vec![
                hint_lbl(" "),
                hint_key(cmd_p),
                hint_lbl(" "),
                hint_key(cmd_n),
            ]),
            Line::from(vec![hint_lbl(" "), hint_key(cmd_semi)]),
        ]
    }
}
