//! ASCII renderer for the arena.
//!
//! The picture is a fixed grid of `GRID` cells, so all art is made for one size. Each frame
//! draws the background art, then each fighter's art with its feet at the fighter's place.
//! All art is hand-made text in `art/`; `art.rs` has the format.
//!
//! The renderer only reads the world.

// Cell positions convert between floats and integers.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

pub mod art;
pub mod camera;
pub mod color;
pub mod frame;
pub mod term;

use arena_sim::world::World;

use crate::{art::Art, camera::Camera, color::Rgb, frame::Frame};

/// The size of the picture in cells, columns × rows.
pub const GRID: (u16, u16) = (192, 44);
/// Background color of every cell. Set on purpose, so the terminal theme does not show.
pub const BACKDROP: Rgb = Rgb::hex(0x05_0807);

/// Draws a world into a frame of glyphs.
pub struct Renderer {
    background: Art,
    /// The player first, then the boss, as in the simulation.
    fighters: [Art; 2],
}

impl Default for Renderer {
    fn default() -> Renderer {
        let read =
            |text: &str, name: &str| Art::parse(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        Renderer {
            background: read(include_str!("../art/arena.txt"), "art/arena.txt"),
            fighters: [
                read(include_str!("../art/wanderer.txt"), "art/wanderer.txt"),
                read(include_str!("../art/moth.txt"), "art/moth.txt"),
            ],
        }
    }
}

impl Renderer {
    /// Draw `world` into `frame`. `t` is the animation clock in seconds.
    pub fn render(&self, world: &World, t: f32, frame: &mut Frame) {
        frame.reset(GRID.0, GRID.1, BACKDROP);
        self.background.frame(t, 1.0).draw(frame, 0, 0);
        let cam = Camera::new(&world.arena);
        for (i, (art, f)) in self.fighters.iter().zip(&world.fighters).enumerate() {
            let (col, row) = cam.feet(f.position);
            // Offset the clocks, so the two idle loops do not step together.
            let phase = i as f32 * 0.37;
            art.frame(t + phase, f.facing.sign()).stamp(frame, col, row);
        }
    }
}
