use arena_sim::{
    action::{Action, Facing},
    world::World,
};

use crate::Controller;

/// A human's brain: the keys and mouse buttons that are down now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Keyboard {
    held: Action,
    left: bool,
    right: bool,
}

impl Keyboard {
    #[must_use]
    pub const fn new() -> Self {
        Keyboard {
            held: Action { ..Action::BASE },
            left: false,
            right: false,
        }
    }

    /// Record that key `c` went down or up.
    pub fn key(&mut self, c: char, down: bool) {
        match c {
            'a' => self.left = down,
            'd' => self.right = down,
            ' ' => self.held.jump = down,
            _ => {}
        }
    }

    /// Record that the left or the right mouse button went down or up.
    pub fn mouse(&mut self, left: bool, right: bool, down: bool) {
        if left {
            self.held.attack = down;
        }
        if right {
            self.held.cast = down;
        }
    }

    pub fn shift(&mut self, down: bool) {
        self.held.dash = down;
    }
}

impl Controller for Keyboard {
    fn act(&mut self, _world: &World, _me: usize) -> Action {
        // Both keys down, or neither: stand still.
        let direction = match (self.left, self.right) {
            (true, false) => Some(Facing::Left),
            (false, true) => Some(Facing::Right),
            _ => None,
        };
        Action {
            direction,
            ..self.held
        }
    }
}
