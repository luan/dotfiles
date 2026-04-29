use ratatui::prelude::*;

// Catppuccin Mocha
pub(crate) const MANTLE: Color = Color::Rgb(0x11, 0x11, 0x1b);
pub(crate) const BASE: Color = Color::Rgb(0x15, 0x15, 0x20);
pub(crate) const SURFACE0: Color = Color::Rgb(0x1c, 0x1c, 0x29);
pub(crate) const SURFACE1: Color = Color::Rgb(0x58, 0x5b, 0x70);
pub(crate) const OVERLAY0: Color = Color::Rgb(0x7f, 0x84, 0x9c);
pub(crate) const SUBTEXT0: Color = Color::Rgb(0xba, 0xc2, 0xde);
pub(crate) const TEXT: Color = Color::Rgb(0xcd, 0xd6, 0xf4);
pub(crate) const PEACH: Color = Color::Rgb(0xfa, 0xb3, 0x87);
pub(crate) const BLUE: Color = Color::Rgb(0x89, 0xb4, 0xfa);
pub(crate) const MAUVE: Color = Color::Rgb(0xcb, 0xa6, 0xf7);

const NUM_SELECTED: &[char] = &[
    '\u{F03A4}',
    '\u{F03A7}',
    '\u{F03AA}',
    '\u{F03AD}',
    '\u{F03B1}',
    '\u{F03B3}',
    '\u{F03B6}',
    '\u{F03B9}',
    '\u{F03BC}',
    '\u{F03BF}',
];
const NUM_UNSELECTED: &[char] = &[
    '\u{F03A6}',
    '\u{F03A9}',
    '\u{F03AC}',
    '\u{F03AE}',
    '\u{F03B0}',
    '\u{F03B5}',
    '\u{F03B8}',
    '\u{F03BB}',
    '\u{F03BE}',
    '\u{F03C1}',
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

pub(crate) fn num_glyph(idx: usize, selected: bool) -> char {
    let table = if selected {
        NUM_SELECTED
    } else {
        NUM_UNSELECTED
    };
    *table.get(idx).unwrap_or(&table[table.len() - 1])
}

pub(crate) fn group_glyph(count: usize, selected: bool) -> char {
    let table = if selected {
        GROUP_SELECTED
    } else {
        GROUP_UNSELECTED
    };
    let idx = count.clamp(1, table.len()) - 1;
    table[idx]
}

pub(crate) fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color::Rgb(r, g, b)
    } else {
        Color::Rgb(0x89, 0xb4, 0xfa)
    }
}

// ── Display primitives (moved from sidebar/claude.rs) ────────────

use std::time::Duration;

pub(crate) fn ctx_label_color(pct: u8) -> Color {
    if pct >= 80 {
        Color::Rgb(0xff, 0x5f, 0x5f) // red — nearly full
    } else if pct >= 50 {
        Color::Rgb(0xff, 0xaf, 0x5f) // orange — watch it
    } else {
        Color::Rgb(0x87, 0xd7, 0x87) // green — plenty of room
    }
}

pub(crate) fn dim_color(c: Color) -> Color {
    let Color::Rgb(r, g, b) = c else { return c };
    let scale = |v: u8| ((v as u32 * 40) / 100) as u8;
    Color::Rgb(scale(r), scale(g), scale(b))
}

pub(crate) fn format_age(d: Duration) -> String {
    let s = d.as_secs();
    if s < 60 {
        "<1m".to_string()
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else {
        ">1h".to_string()
    }
}

pub(crate) fn age_color(d: Duration) -> Color {
    let s = d.as_secs();
    if s < 300 {
        Color::Rgb(0xa6, 0xe3, 0xa1)
    } else if s < 3600 {
        Color::Rgb(0x89, 0xb4, 0xfa)
    } else {
        Color::Rgb(0xf3, 0x8b, 0xa8)
    }
}
