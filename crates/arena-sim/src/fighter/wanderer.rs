use crate::{
    action::Action,
    fighter::{Stats, moves::Move},
};

pub const STATS: Stats = Stats {
    max_health: 5,
    run_speed: 1.5,
    jump_speed: 4.0,
    dash_speed: 4.0,
    dash_duration: 8,
    dash_cooldown: 30,
};

pub const MOVES: [Move; 0] = [];

/// Return the id of the move that this input starts.
#[must_use]
pub fn pick(_pressed: Action, _held: Action, _on_ground: bool) -> Option<usize> {
    None
}
