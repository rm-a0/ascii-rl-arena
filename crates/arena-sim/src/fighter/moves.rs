use crate::physics::{Rect, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub startup: u32,
    pub active: u32,
    pub recovery: u32,
    pub hitbox: Option<Rect>,
    pub damage: u32,
    pub knockback: Vec2,
    pub windup: Vec2,
    pub lunge: Vec2,
    pub guard: bool,
    pub on_block: Option<usize>,
}

impl Move {
    pub const BASE: Move = Move {
        startup: 0,
        active: 0,
        recovery: 0,
        damage: 0,
        hitbox: None,
        knockback: Vec2::ZERO,
        windup: Vec2::ZERO,
        lunge: Vec2::ZERO,
        guard: false,
        on_block: None,
    };
}
