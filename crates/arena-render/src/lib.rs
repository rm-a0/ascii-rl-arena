//! ASCII renderer for the arena.
//!
//! The picture is a fixed grid of `GRID` cells, so all art is made for one size. Each frame
//! draws the background art, then each fighter's art with its feet at the fighter's place,
//! then each shot, then the boss's health bar under the floor.
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

use arena_sim::{
    fighter::{Fighter, Kind, State, moth, moves::Phase, wanderer},
    world::World,
};

use crate::{art::Art, camera::Camera, color::Rgb, frame::Frame};

/// The size of the picture in cells, columns × rows.
pub const GRID: (u16, u16) = (192, 44);
/// Background color of every cell. Set on purpose, so the terminal theme does not show.
pub const BACKDROP: Rgb = Rgb::hex(0x05_0807);
/// The boss's health bar: columns at full health, its row under the floor, and colors.
const BAR_COLS: i32 = 80;
const BAR_ROW: i32 = GRID.1 as i32 - 2;
const BAR_FULL: Rgb = Rgb::hex(0xc0_3048);
const BAR_EMPTY: Rgb = Rgb::hex(0x2a_0e14);

/// The hand-made arenas. Each is drawn for an arena 294 world pixels wide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// A mossy hollow under a dark canopy, with a ruined arch.
    Hollow,
    /// A moonlit lake with a stone bridge.
    Lake,
}

impl Stage {
    /// Return the next stage, and the first after the last.
    #[must_use]
    pub fn next(self) -> Stage {
        match self {
            Stage::Hollow => Stage::Lake,
            Stage::Lake => Stage::Hollow,
        }
    }
}

/// Draws a world into a frame of glyphs.
pub struct Renderer {
    pub stage: Stage,
    /// The background art of each stage, in the order of `Stage`.
    backgrounds: [Art; 2],
    /// The player first, then the boss, as in the simulation.
    fighters: [Art; 2],
    /// The wanderer's cast. It is the only shot so far.
    orb: Art,
}

impl Default for Renderer {
    fn default() -> Renderer {
        let read =
            |text: &str, name: &str| Art::parse(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        Renderer {
            stage: Stage::Hollow,
            backgrounds: [
                read(include_str!("../art/hollow.txt"), "art/hollow.txt"),
                read(include_str!("../art/lake.txt"), "art/lake.txt"),
            ],
            fighters: [
                read(include_str!("../art/wanderer.txt"), "art/wanderer.txt"),
                read(include_str!("../art/moth.txt"), "art/moth.txt"),
            ],
            orb: read(include_str!("../art/orb.txt"), "art/orb.txt"),
        }
    }
}

impl Renderer {
    /// Draw `world` into `frame`. `t` is the animation clock in seconds.
    pub fn render(&self, world: &World, t: f32, frame: &mut Frame) {
        frame.reset(GRID.0, GRID.1, BACKDROP);
        self.backgrounds[self.stage as usize]
            .frame(t, 1.0)
            .draw(frame, 0, 0);
        let cam = Camera::new(&world.arena);
        for (i, (art, f)) in self.fighters.iter().zip(&world.fighters).enumerate() {
            let (col, row) = cam.feet(f.position);
            // Offset the clocks, so the two idle loops do not step together.
            let offset = i as f32 * 0.37;
            art.pose(&pose_name(f), t + offset, f.facing.sign())
                .stamp(frame, col, row);
        }
        for shot in &world.shots {
            let (col, row) = cam.feet(shot.position);
            self.orb.frame(t, shot.facing.sign()).stamp(frame, col, row);
        }
        boss_bar(&world.fighters[1], frame);
    }
}

/// Draw the boss's health as a red bar, centered on its row. Clear the floor art under it.
fn boss_bar(boss: &Fighter, frame: &mut Frame) {
    let left = (i32::from(GRID.0) - BAR_COLS) / 2;
    let share = boss.health as f32 / boss.stats().max_health as f32;
    let full = (BAR_COLS as f32 * share).round() as i32;
    for col in left - 1..=left + BAR_COLS {
        let bg = match col - left {
            i if !(0..BAR_COLS).contains(&i) => BACKDROP,
            i if i < full => BAR_FULL,
            _ => BAR_EMPTY,
        };
        frame.put(col, BAR_ROW, ' ', BACKDROP);
        frame.set_bg(col, BAR_ROW, bg);
    }
}

/// Return the name of the pose that shows what a fighter does, such as `stab.active`.
/// The art files name their poses the same way.
#[must_use]
pub fn pose_name(f: &Fighter) -> String {
    match f.state {
        State::Free => "idle".to_string(),
        State::Dash { .. } => "dash".to_string(),
        State::Hurt { .. } => "hurt".to_string(),
        State::Dead => "dead".to_string(),
        State::Attack { id, tick, .. } => {
            let phase = match f.moves()[id].phase(tick) {
                Some(Phase::Startup) => "startup",
                Some(Phase::Hold) => "hold",
                Some(Phase::Active) => "active",
                Some(Phase::Recovery) | None => "recovery",
            };
            format!("{}.{phase}", move_name(f.kind, id))
        }
    }
}

/// Return the name of move `id` of a fighter kind, as the art files use it.
#[must_use]
pub fn move_name(kind: Kind, id: usize) -> &'static str {
    match (kind, id) {
        (Kind::Moth, moth::STAB_ID) => "stab",
        (Kind::Moth, moth::LUNGE_ID) => "lunge",
        (Kind::Moth, moth::DIVE_ID) => "dive",
        (Kind::Moth, moth::GUST_ID) => "gust",
        (Kind::Wanderer, wanderer::ATTACK_ID) => "attack",
        (Kind::Wanderer, wanderer::CAST_ID) => "cast",
        _ => "move",
    }
}
