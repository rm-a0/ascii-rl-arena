use crate::{
    action::Facing,
    physics::{Rect, Vec2},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Startup,
    Hold,
    Active,
    Recovery,
}

/// One hit as it reaches its target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    pub damage: u32,
    /// In world direction, already flipped by the attacker's facing.
    pub knockback: Vec2,
    pub hitstun: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projectile {
    pub spawn: Vec2,
    pub velocity: Vec2,
    pub hitbox: Rect,
    pub damage: u32,
    pub knockback: Vec2,
    pub hitstun: u32,
}

impl Projectile {
    pub const BASE: Projectile = Projectile {
        spawn: Vec2::ZERO,
        velocity: Vec2::ZERO,
        hitbox: Rect::ZERO,
        damage: 0,
        knockback: Vec2::ZERO,
        hitstun: 0,
    };
}

/// A projectile in flight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    /// Index of the fighter that fired it. It can hit only the other fighter.
    pub owner: usize,
    pub position: Vec2,
    pub facing: Facing,
    pub projectile: Projectile,
}

impl Shot {
    #[must_use]
    pub fn hitbox(&self) -> Rect {
        self.projectile.hitbox.placed(self.position, self.facing)
    }

    /// Move by one tick of velocity.
    pub fn fly(&mut self) {
        let v = self.projectile.velocity;
        self.position += Vec2::new(v.x * self.facing.sign(), v.y);
    }

    #[must_use]
    pub fn strike(&self) -> Strike {
        let p = &self.projectile;
        Strike {
            damage: p.damage,
            knockback: Vec2::new(p.knockback.x * self.facing.sign(), p.knockback.y),
            hitstun: p.hitstun,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub startup: u32,
    /// Ticks after startup in which the fighter hangs still in the air and turns to face
    /// the foe.
    pub hold: u32,
    pub active: u32,
    pub recovery: u32,
    pub hitbox: Option<Rect>,
    pub damage: u32,
    pub knockback: Vec2,
    pub hitstun: u32,
    /// Velocity in px/tick in startup ticks, facing right. A zero y leaves the fall to gravity.
    /// A zero x in the air keeps the drift.
    pub windup: Vec2,
    /// Velocity in px/tick in active ticks, as `windup`.
    pub lunge: Vec2,
    /// The active ticks fly straight to the foe's feet as they are on the first active
    /// tick, never upward, and arrive on the last. Gravity does not pull. `lunge` is unused.
    pub aim: bool,
    pub projectile: Option<Projectile>,
    /// Ticks from the start of the move until it can start again.
    pub cooldown: u32,
}

impl Move {
    pub const BASE: Move = Move {
        startup: 0,
        hold: 0,
        active: 0,
        recovery: 0,
        damage: 0,
        hitbox: None,
        knockback: Vec2::ZERO,
        hitstun: 0,
        windup: Vec2::ZERO,
        lunge: Vec2::ZERO,
        aim: false,
        projectile: None,
        cooldown: 0,
    };

    /// Return the phase at `tick`, counted from 0. Return `None` after the last tick.
    #[must_use]
    pub const fn phase(&self, tick: u32) -> Option<Phase> {
        if tick < self.startup {
            Some(Phase::Startup)
        } else if tick < self.startup + self.hold {
            Some(Phase::Hold)
        } else if tick < self.first_active() + self.active {
            Some(Phase::Active)
        } else if tick < self.first_active() + self.active + self.recovery {
            Some(Phase::Recovery)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn first_active(&self) -> u32 {
        self.startup + self.hold
    }

    #[must_use]
    pub fn is_active(&self, tick: u32) -> bool {
        self.phase(tick) == Some(Phase::Active)
    }
}
