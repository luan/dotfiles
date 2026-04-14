use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

mod frame;
mod hints;
mod items;
mod overlays;

pub(super) use frame::draw;

fn render_at(f: &mut Frame, x: u16, y: u16, w: u16, line: Line<'_>, bg: Color) {
    let area = Rect {
        x,
        y,
        width: w,
        height: 1,
    };
    f.render_widget(Paragraph::new(line).style(Style::default().bg(bg)), area);
}
