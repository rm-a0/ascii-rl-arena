use std::ops::Range;

use arena_sim::{
    action::{Action, Facing},
    fighter::{Fighter, State, moth, moves::Move},
    physics::{Rect, Vec2},
    world::World,
};
use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;

use crate::Controller;

/// Feet closer than this, in pixels, are too close to fight. The bot backs off.
const TOO_CLOSE: f32 = 8.0;
/// Feet closer than this, in pixels, crowd the bot. After a hit it gusts or dashes away.
const CROWDED: f32 = 36.0;
/// Largest gap between the feet, in pixels, from which the stab catches a foe that runs.
const STAB_GAP: f32 = 50.0;
/// Gap between the feet, in pixels, where the bot can dive. The dive aims itself.
const DIVE_RANGE: Range<f32> = 30.0..150.0;
/// Chance to dive on a choice in range, when the dive is ready.
const DIVE_CHANCE: f64 = 0.15;
/// Ticks of rest after the bot's own move. This gives the foe a window to attack.
const REST: Range<u32> = 15..40;
/// Ticks that one plan lasts.
const PLAN: Range<u32> = 8..24;

/// The moth's scripted brain. It acts only while its fighter is `Free`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MothBot {
    rng: Pcg32,
    plan: Plan,
    /// Ticks until the next choice. Counts down only while the fighter is `Free`.
    left: u32,
    /// The fighter was `Hurt` on the last tick.
    was_hurt: bool,
}

/// What the legs do while nothing needs an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plan {
    Idle,
    Approach,
    BackOff,
}

impl Controller for MothBot {
    /// Rules in priority order. The first rule that gives an action wins.
    fn act(&mut self, world: &World, me: usize) -> Action {
        let foe = &world.fighters[1 - me];
        let me = &world.fighters[me];
        let recovered = self.was_hurt && me.state == State::Free;
        self.was_hurt = matches!(me.state, State::Hurt { .. });

        // The sim runs a started move. Release all buttons, or a held one starts it again.
        if me.state != State::Free {
            return Action::BASE;
        }
        if recovered && let Some(action) = self.escape(me, foe) {
            return action;
        }
        if let Some(action) = self.punish(me, foe) {
            return action;
        }
        if self.left > 0 {
            return self.follow(me, foe);
        }
        self.choose(me, foe)
    }
}

impl MothBot {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Pcg32::seed_from_u64(seed),
            plan: Plan::Idle,
            left: 0,
            was_hurt: false,
        }
    }

    /// Answer a hit on the first free tick after it, if the foe is still close: gust or
    /// dash away, on a coin flip.
    fn escape(&mut self, me: &Fighter, foe: &Fighter) -> Option<Action> {
        if (foe.position.x - me.position.x).abs() >= CROWDED {
            return None;
        }
        if self.rng.random_bool(0.5) {
            let gust = Action {
                special: true,
                direction: Some(toward(me, foe)),
                ..Action::BASE
            };
            return Some(self.commit(gust));
        }
        // Keep backing off after the dash.
        self.plan = Plan::BackOff;
        self.left = self.rng.random_range(PLAN);
        Some(Action {
            dash: true,
            direction: Some(toward(foe, me)),
            ..Action::BASE
        })
    }

    /// Stab a foe that is stuck in its move after a miss and cannot hit the bot.
    fn punish(&mut self, me: &Fighter, foe: &Fighter) -> Option<Action> {
        let State::Attack { id, tick, .. } = foe.state else {
            return None;
        };
        let m = &foe.moves()[id];
        let stuck_for = (m.first_active() + m.active + m.recovery).saturating_sub(tick);
        let safe = foe
            .strike()
            .is_none_or(|(hitbox, _)| me.hurtbox().is_none_or(|body| !hitbox.intersects(&body)));
        let to = toward(me, foe);
        let lands = tick >= m.startup
            && safe
            && stuck_for > moth::STAB.startup
            && can_hit(&moth::STAB, me.position, to, foe);
        let stab = Action {
            attack: true,
            direction: Some(to),
            ..Action::BASE
        };
        lands.then(|| self.commit(stab))
    }

    /// Pick a move or a new plan by distance. Each arm's range is its chance in percent.
    fn choose(&mut self, me: &Fighter, foe: &Fighter) -> Action {
        let to = toward(me, foe);
        // The direction turns the moth before the move starts, in the same tick.
        let stab = Action {
            attack: true,
            direction: Some(to),
            ..Action::BASE
        };
        let lunge = Action {
            lunge: true,
            direction: Some(to),
            ..Action::BASE
        };
        let dive = Action {
            cast: true,
            direction: Some(to),
            ..Action::BASE
        };
        let gust = Action {
            special: true,
            direction: Some(to),
            ..Action::BASE
        };
        let gap = (foe.position.x - me.position.x).abs();
        let dive_ready = me.cooldown_left[moth::DIVE_ID] == 0 && DIVE_RANGE.contains(&gap);
        let roll = self.rng.random_range(0..100);
        if gap < TOO_CLOSE {
            self.start(Plan::BackOff, me, foe)
        } else if dive_ready && self.rng.random_bool(DIVE_CHANCE) {
            self.commit(dive)
        } else if gap < CROWDED {
            match roll {
                0..40 => self.commit(gust),
                40..70 => self.commit(stab),
                _ => self.start(Plan::BackOff, me, foe),
            }
        } else if in_stab_range(me, foe) {
            match roll {
                0..50 => self.commit(stab),
                50..75 => self.start(Plan::BackOff, me, foe),
                _ => self.start(Plan::Idle, me, foe),
            }
        } else if can_hit(&moth::LUNGE, me.position, to, foe) {
            match roll {
                0..40 => self.commit(lunge),
                40..80 => self.start(Plan::Approach, me, foe),
                _ => self.start(Plan::Idle, me, foe),
            }
        } else {
            match roll {
                0..80 => self.start(Plan::Approach, me, foe),
                _ => self.start(Plan::Idle, me, foe),
            }
        }
    }

    /// Return `action` and rest after the move it starts.
    fn commit(&mut self, action: Action) -> Action {
        self.plan = Plan::Idle;
        self.left = self.rng.random_range(REST);
        action
    }

    /// Start a plan for a random time and take its first step.
    fn start(&mut self, plan: Plan, me: &Fighter, foe: &Fighter) -> Action {
        self.plan = plan;
        self.left = self.rng.random_range(PLAN);
        self.follow(me, foe)
    }

    /// Take one step of the current plan.
    fn follow(&mut self, me: &Fighter, foe: &Fighter) -> Action {
        self.left = self.left.saturating_sub(1);
        let to = toward(me, foe);
        let direction = match self.plan {
            Plan::Idle => None,
            // Stop in stab range, so that the next choice can stab.
            Plan::Approach if in_stab_range(me, foe) => {
                self.left = 0;
                None
            }
            Plan::Approach => Some(to),
            // From the foe toward the bot is away from the foe.
            Plan::BackOff => Some(toward(foe, me)),
        };
        Action {
            direction,
            ..Action::BASE
        }
    }
}

/// Return the facing from `me` toward `foe`.
fn toward(me: &Fighter, foe: &Fighter) -> Facing {
    if foe.position.x >= me.position.x {
        Facing::Right
    } else {
        Facing::Left
    }
}

/// Return `true` if a stab now hits the foe, even if it runs.
fn in_stab_range(me: &Fighter, foe: &Fighter) -> bool {
    (foe.position.x - me.position.x).abs() < STAB_GAP
        && can_hit(&moth::STAB, me.position, toward(me, foe), foe)
}

/// Return `true` if move `m`, started at `feet`, can reach `target` where it stands now.
///
/// The hitbox is stretched by the distance that the move travels in startup and active ticks.
#[expect(clippy::cast_precision_loss, reason = "tick counts are small")]
fn can_hit(m: &Move, feet: Vec2, facing: Facing, target: &Fighter) -> bool {
    let (Some(hitbox), Some(body)) = (m.hitbox, target.hurtbox()) else {
        return false;
    };
    let travel = m.windup.x * m.startup as f32 + m.lunge.x * m.active as f32;
    let reach = Rect::new(hitbox.min, Vec2::new(hitbox.max.x + travel, hitbox.max.y));
    reach.placed(feet, facing).intersects(&body)
}
