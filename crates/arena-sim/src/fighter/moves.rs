use crate::physics::{Rect, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Startup,
    Active,
    Recovery,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projectile {
    pub spawn: Vec2,
    pub velocity: Vec2,
    pub hitbox: Rect,
    pub damage: u32,
    pub knockback: Vec2,
}

impl Projectile {
    pub const BASE: Projectile = Projectile {
        spawn: Vec2::ZERO,
        velocity: Vec2::ZERO,
        hitbox: Rect::ZERO,
        damage: 0,
        knockback: Vec2::ZERO,
    };
}

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
    pub projectile: Option<Projectile>,
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
        projectile: None,
    };

    /// Return the phase at `tick`, counted from 0. Return `None` after the last tick.
    #[must_use]
    pub const fn phase(&self, tick: u32) -> Option<Phase> {
        if tick < self.startup {
            Some(Phase::Startup)
        } else if tick < self.startup + self.active {
            Some(Phase::Active)
        } else if tick < self.startup + self.active + self.recovery {
            Some(Phase::Recovery)
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_active(&self, tick: u32) -> bool {
        self.phase(tick) == Some(Phase::Active)
    }
}
