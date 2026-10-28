//! Terminal output through ratatui.

use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use crate::frame::Frame;

/// Return the `cols` × `rows` box in the middle of `area`, or `None` when it does not fit.
#[must_use]
pub fn centered(area: Rect, cols: u16, rows: u16) -> Option<Rect> {
    (area.width >= cols && area.height >= rows).then(|| Rect {
        x: area.x + (area.width - cols) / 2,
        y: area.y + (area.height - rows) / 2,
        width: cols,
        height: rows,
    })
}

impl Widget for &Frame {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for row in 0..self.rows.min(area.height) {
            for col in 0..self.cols.min(area.width) {
                let c = self.cells[usize::from(row) * usize::from(self.cols) + usize::from(col)];
                let [fr, fg, fb] = c.fg;
                let [br, bg, bb] = c.bg;
                buf[(area.x + col, area.y + row)]
                    .set_char(c.ch)
                    .set_fg(Color::Rgb(fr, fg, fb))
                    .set_bg(Color::Rgb(br, bg, bb));
            }
        }
    }
}
