use crate::{decode::Choice, env::Env, observe::OBS_LEN, reward::Weights};

/// Fights that step together. A fight that ends starts again in the same step.
#[derive(Clone, Debug)]
pub struct VecEnv {
    envs: Vec<Env>,
    /// Seed of the next fight to start. Each fight gets its own seed.
    next_seed: u64,
}

impl VecEnv {
    #[must_use]
    pub fn new(num_envs: usize, weights: Weights, frame_skip: u32, seed: u64) -> Self {
        Self {
            envs: vec![Env::new(weights, frame_skip); num_envs],
            next_seed: seed,
        }
    }

    #[must_use]
    pub fn num_envs(&self) -> usize {
        self.envs.len()
    }

    /// Start a new fight in every env. `obs` holds `num_envs * OBS_LEN` values.
    pub fn reset(&mut self, obs: &mut [f32]) {
        for (env, out) in self.envs.iter_mut().zip(obs.chunks_mut(OBS_LEN)) {
            env.reset(self.next_seed, out);
            self.next_seed += 1;
        }
    }

    /// Step every fight once. `obs` holds `OBS_LEN` values per env, the others one per env.
    ///
    /// The observation of a done fight is the first one of its next fight.
    pub fn step(
        &mut self,
        choices: &[Choice],
        obs: &mut [f32],
        rewards: &mut [f32],
        dones: &mut [bool],
    ) {
        for (i, out) in obs.chunks_mut(OBS_LEN).enumerate() {
            let env = &mut self.envs[i];
            (rewards[i], dones[i]) = env.step(choices[i], out);
            if dones[i] {
                env.reset(self.next_seed, out);
                self.next_seed += 1;
            }
        }
    }
}
