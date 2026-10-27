use crate::{
    action::{Action, Facing},
    physics::Vec2,
};

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
    pub dash_cooldown: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fighter {
    pub stats: Stats,
    pub position: Vec2,
    pub velocity: Vec2,
    pub facing: Facing,
    pub on_ground: bool,
    /// The input of the previous tick, for press and release checks.
    pub last_action: Action,
}
