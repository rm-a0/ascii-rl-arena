use crate::{
    action::Action,
    fighter::{Stats, moves::Move},
    physics::{Rect, Vec2},
};

const BODY: Rect = Rect::new(Vec2::new(-4.0, 0.0), Vec2::new(5.0, 34.0));

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
pub const STAB_ID: usize = 0;
pub const LUNGE_ID: usize = 1;
pub const GUARD_ID: usize = 2;
pub const RIPOSTE_ID: usize = 3;

pub const STAB: Move = Move {
    startup: 8,
    active: 4,
    recovery: 12,
    damage: 1,
    hitbox: Some(Rect::new(Vec2::new(6.0, 12.0), Vec2::new(28.0, 18.0))),
    hitstun: 12,
    ..Move::BASE
};

pub const LUNGE: Move = Move {
    startup: 18,
    active: 10,
    recovery: 22,
    damage: 2,
    hitbox: Some(Rect::new(Vec2::new(6.0, 12.0), Vec2::new(28.0, 18.0))),
    knockback: Vec2::new(1.0, 0.0),
    lunge: Vec2::new(5.0, 0.0),
    hitstun: 20,
    ..Move::BASE
};

pub const GUARD: Move = Move {
    startup: 2,
    active: 24,
    recovery: 8,
    guard: true,
    on_block: Some(RIPOSTE_ID),
    ..Move::BASE
};

pub const RIPOSTE: Move = Move {
    startup: 2,
    active: 4,
    recovery: 14,
    damage: 1,
    hitbox: Some(Rect::new(Vec2::new(6.0, 2.0), Vec2::new(22.0, 22.0))),
    knockback: Vec2::new(1.0, 0.0),
    hitstun: 20,
    ..Move::BASE
};

pub const MOVES: [Move; 4] = {
    let mut m = [Move::BASE; 4];
    m[STAB_ID] = STAB;
    m[LUNGE_ID] = LUNGE;
    m[GUARD_ID] = GUARD;
    m[RIPOSTE_ID] = RIPOSTE;
    m
};

/// Return the id of the move that this input starts. Guard wins over attack.
#[must_use]
pub fn pick(pressed: Action, _held: Action, _on_ground: bool) -> Option<usize> {
    if pressed.guard {
        Some(GUARD_ID)
    } else if pressed.attack {
        Some(STAB_ID)
    } else {
        None
    }
}
