use std::ops::Deref;

use crate::fighter::Fighter;

pub enum MothState {
    Idle,
    Chase,
    Attack,
    ChargeAttack,
    Shield,
}

pub struct Moth {
    pub fighter: Fighter,
    pub state: MothState,
}

impl Deref for Moth {
    type Target = Fighter;
    fn deref(&self) -> &Self::Target {
        &self.fighter
    }
}
