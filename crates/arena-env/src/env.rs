use arena_control::{Controller, moth_bot::MothBot};
use arena_sim::{
    action::Facing,
    fighter::{Fighter, Kind},
    physics::Vec2,
    world::{Arena, World},
};

use crate::{
    decode::{Choice, decode},
    observe::observe,
    reward::Weights,
};

/// Ticks in one fight: 90 seconds.
const MAX_TICKS: u32 = 90 * 60;

#[derive(Clone, Debug)]
pub struct Env {
    world: World,
    bot: MothBot,
    weights: Weights,
    frame_skip: u32,
}

impl Env {
    #[must_use]
    pub fn new(weights: Weights, frame_skip: u32) -> Self {
        Env {
            world: fresh_world(),
            bot: MothBot::new(0),
            weights,
            frame_skip,
        }
    }

    pub fn reset(&mut self, seed: u64, out: &mut [f32]) {
        self.world = fresh_world();
        self.bot = MothBot::new(seed);
        observe(&self.world, 0, out);
    }

    pub fn step(&mut self, choice: Choice, out: &mut [f32]) -> (f32, bool) {
        let mut reward = 0.0;
        let mut done = false;
        let action = decode(choice, &self.world, 0);

        for _ in 0..self.frame_skip {
            let events = self.world.step([action, self.bot.act(&self.world, 1)]);
            reward += self.weights.reward(&events, 0);
            done = self.world.is_over();
            if done {
                break;
            }
        }
        observe(&self.world, 0, out);
        (reward, done)
    }
}

/// Return the fight at its start: the wanderer (index 0) left, the moth (index 1) right.
///
/// Keep it the same as `new_world` in the preview, so that a replay starts from the same world.
fn fresh_world() -> World {
    let arena = Arena {
        ground_y: 0.0,
        left_x: 0.0,
        right_x: 294.0,
    };
    let wanderer = Fighter::new(Kind::Wanderer, Vec2::new(112.0, 0.0), Facing::Right);
    let moth = Fighter::new(Kind::Moth, Vec2::new(182.0, 0.0), Facing::Left);
    World::new(arena, [wanderer, moth], MAX_TICKS)
}
