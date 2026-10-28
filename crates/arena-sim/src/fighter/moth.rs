use crate::{
    fighter::{Fighter, moves::Move},
    physics::{Rect, Vec2},
};

pub const RIPOSTE_ID: usize = 3;

pub const STAB: Move = Move {
    startup: 8,
    active: 4,
    recovery: 12,
    damage: 1,
    hitbox: Some(Rect::new(Vec2::new(6.0, 2.0), Vec2::new(22.0, 10.0))),
    ..Move::BASE
};

pub const LUNGE: Move = Move {
    startup: 18,
    active: 10,
    recovery: 22,
    damage: 2,
    hitbox: Some(Rect::new(Vec2::new(6.0, 2.0), Vec2::new(22.0, 10.0))),
    knockback: Vec2::new(1.0, 0.0),
    lunge: Vec2::new(5.0, 0.0),
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
    hitbox: Some(Rect::new(Vec2::new(6.0, 2.0), Vec2::new(22.0, 10.0))),
    knockback: Vec2::new(1.0, 0.0),
    ..Move::BASE
};

pub const MOVES: [Move; 4] = {
    let mut m = [Move::BASE; 4];
    m[0] = STAB;
    m[1] = LUNGE;
    m[2] = GUARD;
    m[RIPOSTE_ID] = RIPOSTE;
    m
};

pub struct Moth {
    pub fighter: Fighter,
}
