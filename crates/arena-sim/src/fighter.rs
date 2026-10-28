pub mod moth;
pub mod moves;
pub mod wanderer;

use crate::{
    action::{Action, Facing},
    physics::{GRAVITY, Vec2},
    world::Arena,
};

pub enum Kind {
    Moth,
    Wanderer,
}

/// Numbers that do not change during a fight.
///
/// Speeds are in pixels per tick. Durations are in ticks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub max_health: u32,
    pub run_speed: f32,
    pub jump_speed: f32,
    pub dash_speed: f32,
    pub dash_duration: u32,
    /// Counted from the start of a dash.
    pub dash_cooldown: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fighter {
    pub stats: Stats,
    pub position: Vec2,
    pub velocity: Vec2,
    pub facing: Facing,
    pub on_ground: bool,
    /// Ticks left in the current dash. Zero means no dash.
    pub dash_left: u32,
    /// Ticks until the next dash can start.
    pub dash_cooldown_left: u32,
    /// The input of the previous tick, for press and release checks.
    pub last_action: Action,
}

impl Fighter {
    #[must_use]
    pub fn new(stats: Stats, position: Vec2, facing: Facing, on_ground: bool) -> Self {
        Self {
            stats,
            position,
            velocity: Vec2::default(),
            facing,
            on_ground,
            dash_left: 0,
            dash_cooldown_left: 0,
            last_action: Action::default(),
        }
    }

    /// Advance the fighter by one tick.
    pub fn update(&mut self, action: Action, arena: Arena) {
        if let Some(direction) = action.direction {
            self.facing = direction;
        }

        let jump_pressed = action.jump && !self.last_action.jump;
        let dash_pressed = action.dash && !self.last_action.dash;

        if jump_pressed && self.on_ground {
            self.velocity.y = self.stats.jump_speed;
            self.on_ground = false;
        }

        if dash_pressed && self.dash_cooldown_left == 0 {
            self.dash_left = self.stats.dash_duration;
            self.dash_cooldown_left = self.stats.dash_cooldown;
        }

        if self.dash_left > 0 {
            // A dash goes straight forward and ignores gravity.
            self.velocity.x = self.facing.sign() * self.stats.dash_speed;
            self.velocity.y = 0.0;
        } else {
            self.velocity.x = action.direction.map_or(0.0, Facing::sign) * self.stats.run_speed;
            self.velocity.y -= GRAVITY;
        }

        self.position += self.velocity;

        if self.position.y <= arena.ground_y {
            self.position.y = arena.ground_y;
            self.velocity.y = 0.0;
            self.on_ground = true;
        }
        if self.position.x <= arena.left_x {
            self.position.x = arena.left_x;
            self.velocity.x = 0.0;
            self.dash_left = 0;
        }
        if self.position.x >= arena.right_x {
            self.position.x = arena.right_x;
            self.velocity.x = 0.0;
            self.dash_left = 0;
        }

        self.dash_left = self.dash_left.saturating_sub(1);
        self.dash_cooldown_left = self.dash_cooldown_left.saturating_sub(1);
        self.last_action = action;
    }
}
