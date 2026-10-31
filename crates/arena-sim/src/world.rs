use crate::{
    action::Action,
    event::Event,
    fighter::{Fighter, Outcome, moves::Shot},
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
    pub shots: Vec<Shot>,
    pub tick: u32,
    pub max_ticks: u32,
}

impl World {
    #[must_use]
    pub fn new(arena: Arena, fighters: [Fighter; 2], max_ticks: u32) -> Self {
        Self {
            arena,
            fighters,
            shots: Vec::new(),
            tick: 0,
            max_ticks,
        }
    }

    #[must_use]
    pub fn is_over(&self) -> bool {
        self.fighters.iter().any(Fighter::is_dead) || self.tick >= self.max_ticks
    }

    /// Advance the fight by one tick and return what happened. `actions[i]` controls `fighters[i]`.
    pub fn step(&mut self, actions: [Action; 2]) -> Vec<Event> {
        self.tick += 1;
        // Each fighter sees where the other stood before this tick.
        let feet = [self.fighters[0].position, self.fighters[1].position];
        for (i, (fighter, action)) in self.fighters.iter_mut().zip(actions).enumerate() {
            fighter.update(action, self.arena, feet[1 - i]);
        }
        for (owner, fighter) in self.fighters.iter().enumerate() {
            self.shots.extend(fighter.shot(owner));
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
        // A move hits once.
        for &(attacker, _, _) in &strikes {
            self.fighters[attacker].landed();
        }

        // A shot hits once and is gone. It is also gone when it leaves the arena.
        for shot in &mut self.shots {
            shot.fly();
        }
        let (arena, fighters) = (self.arena, &self.fighters);
        self.shots.retain(|shot| {
            let target = 1 - shot.owner;
            let hits = fighters[target]
                .hurtbox()
                .is_some_and(|body| shot.hitbox().intersects(&body));
            if hits {
                strikes.push((shot.owner, target, shot.strike()));
            }
            let inside = (arena.left_x..=arena.right_x).contains(&shot.position.x);
            !hits && inside
        });

        let mut events = Vec::new();
        for (attacker, target, strike) in strikes {
            let hit = Event::Hit {
                attacker,
                target,
                damage: strike.damage,
            };
            match self.fighters[target].struck(&strike) {
                Outcome::Hit => events.push(hit),
                Outcome::Died => events.extend([hit, Event::Died { fighter: target }]),
            }
        }
        // A death on the last tick is a win or a loss, not a timeout.
        if self.max_ticks <= self.tick && !self.fighters.iter().any(Fighter::is_dead) {
            events.push(Event::Timeout);
        }
        events
    }
}
