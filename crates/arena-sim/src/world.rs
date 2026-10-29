use crate::{
    action::Action,
    event::Event,
    fighter::{Fighter, Outcome},
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

    /// Advance the fight by one tick and return what happened. `actions[i]` controls `fighters[i]`.
    pub fn step(&mut self, actions: [Action; 2]) -> Vec<Event> {
        self.tick += 1;
        for (fighter, action) in self.fighters.iter_mut().zip(actions) {
            fighter.update(action, self.arena);
        }

        // Find all strikes before any lands, so both fighters see the same tick.
        let mut strikes = Vec::new();
        for (attacker, target) in [(0, 1), (1, 0)] {
            let Some((hitbox, strike)) = self.fighters[attacker].strike() else {
                continue;
            };
            let Some(hurtbox) = self.fighters[target].hurtbox() else {
                continue;
            };
            if hitbox.intersects(&hurtbox) {
                strikes.push((attacker, target, strike));
            }
        }

        let mut events = Vec::new();
        for (attacker, target, strike) in strikes {
            self.fighters[attacker].landed();
            let hit = Event::Hit {
                attacker,
                target,
                damage: strike.damage,
            };
            match self.fighters[target].struck(&strike) {
                Outcome::Hit => events.push(hit),
                Outcome::Blocked => events.push(Event::Blocked { attacker, target }),
                Outcome::Died => events.extend([hit, Event::Died { fighter: target }]),
            }
        }
        events
    }
}
