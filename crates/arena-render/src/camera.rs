//! Map world pixels to cells of the fixed grid.

use arena_sim::{physics::Vec2, world::Arena};

use crate::GRID;

/// Grid columns per world pixel. At this scale the 27 px wanderer is 7 rows tall, the
/// height of its art.
pub const COLS_PER_PX: f32 = 0.62;
/// Cell height divided by cell width that the art is drawn for: kitty with
/// `cell_height 115%`. A world pixel is then square on screen.
pub const CELL_ASPECT: f32 = 2.5;
/// The grid row of the ground surface in the background art. Fighters stand on the row
/// above it.
pub const GROUND_ROW: i32 = 39;

/// A fixed view: the arena is centered in the grid and does not scroll.
///
/// The grid shows about 310 world pixels across. The background art is drawn for an arena
/// 294 world pixels wide: its floor spans columns 5 to 186.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// World x at the left edge of the grid.
    left: f32,
    ground_y: f32,
}

impl Camera {
    #[must_use]
    pub fn new(arena: &Arena) -> Camera {
        let center = f32::midpoint(arena.left_x, arena.right_x);
        Camera {
            left: center - f32::from(GRID.0) / 2.0 / COLS_PER_PX,
            ground_y: arena.ground_y,
        }
    }

    /// Return the cell (column, row) where a fighter with its feet at `p` stands.
    #[must_use]
    pub fn feet(&self, p: Vec2) -> (i32, i32) {
        let col = ((p.x - self.left) * COLS_PER_PX).floor() as i32;
        let up = ((p.y - self.ground_y) * COLS_PER_PX / CELL_ASPECT).round() as i32;
        (col, GROUND_ROW - 1 - up)
    }
}
