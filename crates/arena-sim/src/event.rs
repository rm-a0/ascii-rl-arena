use crate::fighter::Fighter;

pub enum Event {
    Hit {
        attacker: Fighter,
        target: Fighter,
        damage: u32,
    },
}
