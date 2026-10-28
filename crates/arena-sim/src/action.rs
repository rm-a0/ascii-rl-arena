/// A horizontal direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    Right,
    Left,
}

impl Facing {
    /// Return -1 for left and +1 for right.
    #[must_use]
    pub fn sign(self) -> f32 {
        match self {
            Facing::Left => -1.0,
            Facing::Right => 1.0,
        }
    }
}

/// The input of one tick. A button is `true` while it is held.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent buttons that combine"
)]
pub struct Action {
    pub jump: bool,
    pub dash: bool,
    pub attack: bool,
    pub guard: bool,
    pub direction: Option<Facing>,
}

impl Action {
    #[must_use]
    pub const fn pressed(self, last: Action) -> Action {
        Action {
            jump: self.jump && !last.jump,
            dash: self.dash && !last.dash,
            attack: self.attack && !last.attack,
            guard: self.guard && !last.guard,
            direction: self.direction,
        }
    }
}
