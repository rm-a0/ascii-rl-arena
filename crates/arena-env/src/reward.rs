use arena_sim::event::Event;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weights {
    hit_dealt: f32,
    hit_taken: f32,
    win: f32,
    loss: f32,
}

impl Weights {
    #[must_use]
    pub fn new(hit_dealt: f32, hit_taken: f32, win: f32, loss: f32) -> Self {
        Self {
            hit_dealt,
            hit_taken,
            win,
            loss,
        }
    }

    #[must_use]
    pub fn reward(&self, events: &[Event], me: usize) -> f32 {
        let mut reward: f32 = 0.0;
        for &event in events {
            match event {
                Event::Timeout => reward += self.loss,
                Event::Hit {
                    attacker, target, ..
                } => {
                    if attacker == me {
                        reward += self.hit_dealt;
                    } else if target == me {
                        reward += self.hit_taken;
                    }
                }
                Event::Died { fighter } => {
                    if fighter == me {
                        reward += self.loss;
                    } else {
                        reward += self.win;
                    }
                }
            }
        }
        reward
    }
}
