/// A horizontal direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    Right,
    Left,
}

/// The input of one tick. A button is `true` while it is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Action {
    pub jump: bool,
    pub dash: bool,
    pub direction: Option<Facing>,
}
