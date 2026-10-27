use crate::{
    action::Action,
    fighter::{self, Fighter},
};

/// The edges of the fight area in world pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arena {
    pub ground_y: f32,
    pub left_x: f32,
    pub right_x: f32,
}

/// The full state of one fight.
#[derive(Clone, Debug, PartialEq)]
pub struct World {
    pub arena: Arena,
    /// Index 0 is the player. Index 1 is the boss.
    pub fighters: [Fighter; 2],
    /// Ticks since the start of the fight.
    pub tick: u32,
}

impl World {
    #[must_use]
    pub fn new(arena: Arena, fighters: [Fighter; 2]) -> Self {
        Self {
            arena,
            fighters,
            tick: 0,
        }
    }

    /// Advance the fight by one tick. `actions[i]` controls `fighters[i]`.
    pub fn step(&mut self, actions: [Action; 2]) {
        self.tick += 1;
        for (fighter, action) in self.fighters.iter_mut().zip(actions.iter()) {
            fighter.update(*action, self.arena);
        }
    }
}
