use std::fmt::Write;

use crate::color::Rgb;

/// One terminal cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
}

/// A finished picture: one glyph and two colors per terminal cell.
///
/// A frame does not know where it is shown. The terminal output and file dumps read it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub cols: u16,
    pub rows: u16,
    pub cells: Vec<Cell>,
}

impl Frame {
    /// Set the size and fill every cell with a blank on `bg`.
    pub fn reset(&mut self, cols: u16, rows: u16, bg: Rgb) {
        self.cols = cols;
        self.rows = rows;
        let blank = Cell {
            ch: ' ',
            fg: [0; 3],
            bg: bg.to_u8(),
        };
        self.cells.clear();
        self.cells
            .resize(usize::from(cols) * usize::from(rows), blank);
    }

    fn index(&self, col: i32, row: i32) -> Option<usize> {
        let inside =
            (0..i32::from(self.cols)).contains(&col) && (0..i32::from(self.rows)).contains(&row);
        inside.then(|| row as usize * usize::from(self.cols) + col as usize)
    }

    #[must_use]
    pub fn get(&self, col: i32, row: i32) -> Option<&Cell> {
        self.index(col, row).map(|i| &self.cells[i])
    }

    /// Set the glyph and its color. Cells outside the frame are ignored.
    pub fn put(&mut self, col: i32, row: i32, ch: char, fg: Rgb) {
        if let Some(i) = self.index(col, row) {
            self.cells[i].ch = ch;
            self.cells[i].fg = fg.to_u8();
        }
    }

    /// Set the background color of a cell. Cells outside the frame are ignored.
    pub fn set_bg(&mut self, col: i32, row: i32, bg: Rgb) {
        if let Some(i) = self.index(col, row) {
            self.cells[i].bg = bg.to_u8();
        }
    }

    /// Return the frame as text with 24-bit color escape codes.
    #[must_use]
    pub fn to_ansi(&self) -> String {
        let mut out = String::new();
        for row in self.cells.chunks(usize::from(self.cols).max(1)) {
            let mut last = None;
            for c in row {
                if last != Some((c.fg, c.bg)) {
                    let ([fr, fg, fb], [br, bg, bb]) = (c.fg, c.bg);
                    let _ = write!(out, "\x1b[38;2;{fr};{fg};{fb};48;2;{br};{bg};{bb}m");
                    last = Some((c.fg, c.bg));
                }
                out.push(c.ch);
            }
            out.push_str("\x1b[0m\n");
        }
        out
    }
}
