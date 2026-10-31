pub mod moth_bot;

use arena_sim::{action::Action, world::World};

pub trait Controller {
    fn act(&mut self, world: &World, me: usize) -> Action;
}
