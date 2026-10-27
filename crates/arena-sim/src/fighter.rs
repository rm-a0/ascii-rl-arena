use crate::{action::Facing, mechanic::meter::Meter};

pub struct Stats {
    pub health: Meter,
}

pub struct Fighter {
    pub stats: Stats,
    pub facing: Facing,
}
