use crate::{
    action::Action,
    fighter::{
        Stats,
        moves::{Move, Projectile},
    },
    physics::{Rect, Vec2},
};

const BODY: Rect = Rect::new(Vec2::new(-5.0, 0.0), Vec2::new(5.0, 22.0));

pub const STATS: Stats = Stats {
    max_health: 5,
    run_speed: 1.5,
    jump_speed: 4.0,
    dash_speed: 4.0,
    dash_duration: 8,
    dash_cooldown: 30,
    body: BODY,
};

// Indices into `MOVES`.
pub const ATTACK_ID: usize = 0;
pub const CAST_ID: usize = 1;

pub const ATTACK: Move = Move {
    startup: 10,
    active: 6,
    recovery: 6,
    damage: 1,
    hitbox: Some(Rect::new(Vec2::new(5.0, 6.0), Vec2::new(20.0, 18.0))),
    hitstun: 12,
    ..Move::BASE
};

pub const ORB: Projectile = Projectile {
    spawn: Vec2::new(10.0, 12.0),
    velocity: Vec2::new(5.0, 0.0),
    hitbox: Rect::new(Vec2::new(-3.0, -3.0), Vec2::new(3.0, 3.0)),
    damage: 1,
    hitstun: 12,
    ..Projectile::BASE
};

pub const CAST: Move = Move {
    startup: 8,
    active: 6,
    recovery: 16,
    projectile: Some(ORB),
    ..Move::BASE
};

pub const MOVES: [Move; 2] = {
    let mut m = [Move::BASE; 2];
    m[ATTACK_ID] = ATTACK;
    m[CAST_ID] = CAST;
    m
};

/// Return the id of the move that this input starts.
#[must_use]
pub fn pick(action: Action, _on_ground: bool) -> Option<usize> {
    if action.attack {
        Some(ATTACK_ID)
    } else if action.cast {
        Some(CAST_ID)
    } else {
        None
    }
}
