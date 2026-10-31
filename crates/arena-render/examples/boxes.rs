//! Draw every move's hitbox and projectile over the fighter's idle art, at game scale.
//!
//! Run: `cargo run -p arena-render --example boxes`. It prints one picture and exits, so
//! change a number in the move tables and run it again. Panels wrap to the terminal width.
//! Add a new move to `SHEET` to see it.

// Cell positions convert between floats and integers.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use arena_render::{
    BACKDROP,
    art::Art,
    camera::{CELL_ASPECT, COLS_PER_PX},
    color::Rgb,
    frame::Frame,
};
use arena_sim::{
    fighter::{Kind, moth, moves::Move, wanderer},
    physics::{Rect, Vec2},
};

/// The moves to show, in order.
const SHEET: [(Kind, &str, Move); 6] = [
    (Kind::Moth, "STAB", moth::STAB),
    (Kind::Moth, "LUNGE", moth::LUNGE),
    (Kind::Moth, "GUARD", moth::GUARD),
    (Kind::Moth, "RIPOSTE", moth::RIPOSTE),
    (Kind::Wanderer, "ATTACK", wanderer::ATTACK),
    (Kind::Wanderer, "CAST", wanderer::CAST),
];

const PANEL_W: i32 = 64;
/// Title, 11 rows for the tallest art, ruler, ruler labels, gap.
const PANEL_H: i32 = 15;
/// Ticks between two drawn places of a projectile.
const PATH_STEP: u32 = 10;

const BODY: Rgb = Rgb::hex(0x40_b060);
const HIT: Rgb = Rgb::hex(0xd0_3040);
const ORB: Rgb = Rgb::hex(0xe0_b040);
const TITLE: Rgb = Rgb::hex(0xe6_e1d2);
const NOTE: Rgb = Rgb::hex(0x6f_9a86);
const RULER: Rgb = Rgb::hex(0x3a_5048);

fn main() {
    let arts = [
        (Kind::Wanderer, art(include_str!("../art/wanderer.txt"))),
        (Kind::Moth, art(include_str!("../art/moth.txt"))),
    ];
    let term_w = ratatui::crossterm::terminal::size().map_or(192, |(w, _)| i32::from(w));
    let per_row = (term_w / PANEL_W).max(1);
    let rows = (SHEET.len() as i32 + per_row - 1) / per_row;

    let mut frame = Frame::default();
    let width = (per_row * PANEL_W) as u16;
    frame.reset(width, (rows * PANEL_H + 3) as u16, BACKDROP);

    for (i, (kind, name, m)) in SHEET.iter().enumerate() {
        let left = i as i32 % per_row * PANEL_W;
        let top = i as i32 / per_row * PANEL_H;
        let art = &arts
            .iter()
            .find(|(k, _)| k == kind)
            .expect("art for kind")
            .1;
        panel(&mut frame, left, top, *kind, name, m, art);
    }
    legend(&mut frame, rows * PANEL_H);
    print!("{}", frame.to_ansi());
}

fn art(text: &str) -> Art {
    Art::parse(text).unwrap_or_else(|e| panic!("art: {e}"))
}

/// Draw one move: title, art, boxes and a ruler in world pixels from the feet.
fn panel(frame: &mut Frame, left: i32, top: i32, kind: Kind, name: &str, m: &Move, art: &Art) {
    let sprite = art.frame(0.0, 1.0);
    let feet = (left + 2 + sprite.anchor as i32, top + 11);

    let who = match kind {
        Kind::Moth => "moth",
        Kind::Wanderer => "wanderer",
    };
    let title = format!("{who} {name}");
    text(frame, left + 1, top, &title, TITLE);
    let mut data = format!(
        "{}/{}/{}  dmg {}",
        m.startup, m.active, m.recovery, m.damage
    );
    if m.guard {
        data.push_str("  guard");
    }
    text(frame, left + 3 + title.len() as i32, top, &data, NOTE);

    tint(frame, feet, kind.stats().body, Vec2::ZERO, BODY, 0.35);

    // The body moves during startup and active ticks, and the hitbox moves with it.
    let first = Vec2::new(m.windup.x * m.startup as f32 + m.lunge.x, 0.0);
    let last = Vec2::new(first.x + m.lunge.x * m.active.saturating_sub(1) as f32, 0.0);
    if let Some(hit) = m.hitbox {
        if last != first {
            tint(frame, feet, hit, last, HIT, 0.25);
        }
        tint(frame, feet, hit, first, HIT, 0.6);
    }
    if let Some(p) = m.projectile {
        let spawn = first + p.spawn;
        let right_edge = left + PANEL_W - 2;
        for step in 1.. {
            let t = (step * PATH_STEP) as f32;
            let at = spawn + p.velocity * t;
            if p.velocity == Vec2::ZERO || col_of(feet, at.x + p.hitbox.min.x) > right_edge {
                break;
            }
            tint(frame, feet, p.hitbox, at, ORB, 0.22);
            let (c, r) = (col_of(feet, at.x), row_of(feet, at.y + p.hitbox.max.y) - 1);
            text(frame, c, r, &format!("{}t", step * PATH_STEP), NOTE);
        }
        tint(frame, feet, p.hitbox, spawn, ORB, 0.6);
    }
    sprite.stamp(frame, feet.0, feet.1);
    ruler(frame, feet, left + PANEL_W - 2);
}

/// Mark every 10 px on the ground row and label every 20 px.
fn ruler(frame: &mut Frame, feet: (i32, i32), right_edge: i32) {
    let row = feet.1 + 1;
    for col in feet.0 - 2..=right_edge {
        frame.put(col, row, '·', RULER);
    }
    for px in (0..).step_by(10) {
        let col = col_of(feet, px as f32);
        if col > right_edge {
            break;
        }
        frame.put(col, row, '|', NOTE);
        if px % 20 == 0 {
            text(frame, col, row + 1, &px.to_string(), NOTE);
        }
    }
}

fn legend(frame: &mut Frame, top: i32) {
    let lines = [
        "red: hitbox on the first active tick · dim red: on the last active tick, after the lunge · yellow: projectile on spawn, then every 10 ticks",
        "x/y/z = startup/active/recovery ticks · ruler: px from the feet, facing right · 1 col = 1.6 px, 1 row = 4 px · green: body (hurtbox) at the start",
    ];
    for (i, line) in lines.iter().enumerate() {
        text(frame, 1, top + i as i32, line, NOTE);
    }
}

/// Return the column of world x `px`, measured from the feet.
fn col_of(feet: (i32, i32), px: f32) -> i32 {
    feet.0 + (px * COLS_PER_PX).floor() as i32
}

/// Return the row of world y `px`, measured up from the feet.
fn row_of(feet: (i32, i32), px: f32) -> i32 {
    feet.1 - (px * COLS_PER_PX / CELL_ASPECT).floor() as i32
}

/// Tint the background of every cell that `rect`, moved by `at`, covers.
fn tint(frame: &mut Frame, feet: (i32, i32), rect: Rect, at: Vec2, color: Rgb, t: f32) {
    let (min, max) = (rect.min + at, rect.max + at);
    // `max` is exclusive: a box that ends on a cell edge does not cover the next cell.
    let cols = col_of(feet, min.x)..=col_of(feet, max.x - 0.01);
    let rows = row_of(feet, max.y - 0.01)..=row_of(feet, min.y);
    for row in rows {
        for col in cols.clone() {
            frame.set_bg(col, row, BACKDROP.mix(color, t));
        }
    }
}

fn text(frame: &mut Frame, col: i32, row: i32, s: &str, color: Rgb) {
    for (i, ch) in s.chars().enumerate() {
        frame.put(col + i as i32, row, ch, color);
    }
}
