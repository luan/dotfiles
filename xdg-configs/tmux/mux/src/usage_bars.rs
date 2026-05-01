use std::process::{Command, Stdio};

use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(crate) struct Snapshot {
    pub(crate) lines: Vec<String>,
}

pub(crate) fn collect(width: u16) -> Snapshot {
    let output = Command::new(ct_bin())
        .args([
            "tui",
            "usage-bars",
            "--sidebar",
            "--width",
            &width.to_string(),
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_default();

    Snapshot {
        lines: output
            .lines()
            .map(str::to_string)
            .filter(|line| !line.trim().is_empty())
            .collect(),
    }
}

fn ct_bin() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("CT_BIN") {
        return path.into();
    }

    let cargo_ct = crate::tmux::home().join(".cargo/bin/ct");
    if cargo_ct.exists() {
        return cargo_ct;
    }

    "ct".into()
}

pub(crate) fn height(lines: &[String]) -> u16 {
    lines.len() as u16
}

pub(crate) fn draw(f: &mut Frame, area: Rect, bg: Color, lines: &[String]) {
    if lines.is_empty() || area.height == 0 {
        return;
    }

    for (i, line) in lines.iter().take(area.height as usize).enumerate() {
        let row = Rect {
            x: area.x,
            y: area.y + i as u16,
            width: area.width,
            height: 1,
        };
        f.render_widget(
            Paragraph::new(parse_ansi_line(line, bg)).style(Style::default().bg(bg)),
            row,
        );
    }
}

fn parse_ansi_line(input: &str, bg: Color) -> Line<'static> {
    let mut spans = Vec::new();
    let mut buf = String::new();
    let mut fg: Option<Color> = None;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && matches!(chars.peek(), Some('[')) {
            chars.next();
            let mut code = String::new();
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    if c == 'm' {
                        flush_span(&mut spans, &mut buf, fg, bg);
                        apply_sgr(&code, &mut fg);
                    }
                    break;
                }
                code.push(c);
            }
        } else {
            buf.push(ch);
        }
    }

    flush_span(&mut spans, &mut buf, fg, bg);
    Line::from(spans)
}

fn flush_span(spans: &mut Vec<Span<'static>>, buf: &mut String, fg: Option<Color>, bg: Color) {
    if buf.is_empty() {
        return;
    }
    let mut style = Style::default().bg(bg);
    if let Some(fg) = fg {
        style = style.fg(fg);
    }
    spans.push(Span::styled(std::mem::take(buf), style));
}

fn apply_sgr(code: &str, fg: &mut Option<Color>) {
    if code == "0" || code == "39" {
        *fg = None;
        return;
    }

    let parts: Vec<&str> = code.split(';').collect();
    if parts.len() == 5
        && parts[0] == "38"
        && parts[1] == "2"
        && let (Ok(r), Ok(g), Ok(b)) = (
            parts[2].parse::<u8>(),
            parts[3].parse::<u8>(),
            parts[4].parse::<u8>(),
        )
    {
        *fg = Some(Color::Rgb(r, g, b));
    }
}
