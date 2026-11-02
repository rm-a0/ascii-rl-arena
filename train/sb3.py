"""Train the ready-made SB3 PPO on the arena: the reference curve for `train/ppo.py`."""

import argparse

import numpy as np
from gymnasium import spaces
from stable_baselines3 import PPO
from stable_baselines3.common.callbacks import BaseCallback
from stable_baselines3.common.logger import configure
from stable_baselines3.common.vec_env import VecEnv

import arena
from train.env import FightStats

NUM_ENVS = 16
FIGHT_SEEDS_PER_RUN = 1_000_000_000
LOG_INTERVAL = 5
PPO_SETTINGS = {
    "n_steps": 256,
    "batch_size": 1024,
    "n_epochs": 4,
    "learning_rate": 3e-4,
    "gamma": 0.99,
    "gae_lambda": 0.95,
    "ent_coef": 0.01,
}


class ArenaVecEnv(VecEnv):
    """Give `arena.VecEnv` the interface that SB3 expects."""

    def __init__(self, num_envs: int, seed: int) -> None:
        self.env = arena.VecEnv(num_envs, seed=seed)
        super().__init__(
            num_envs,
            spaces.Box(-np.inf, np.inf, (arena.OBS_LEN,), np.float32),
            spaces.MultiDiscrete(arena.ACTION_DIMS),
        )

    def reset(self):
        return self.env.reset()

    def step_async(self, actions):
        self.actions = actions

    def step_wait(self):
        obs, rewards, dones = self.env.step(self.actions)
        return obs, rewards, dones, [{} for _ in range(self.num_envs)]

    # SB3 requires these. The arena has no gym attributes, methods or wrappers.
    def close(self):
        pass

    def get_attr(self, attr_name, indices=None):
        return [None] * self.num_envs  # SB3 asks only for `render_mode`.

    def set_attr(self, attr_name, value, indices=None):
        raise NotImplementedError

    def env_method(self, method_name, *method_args, indices=None, **method_kwargs):
        raise NotImplementedError

    def env_is_wrapped(self, wrapper_class, indices=None):
        return [False] * self.num_envs


class StatsCallback(BaseCallback):
    """Count the fights during training and log their stats under `fights/`."""

    def __init__(self, num_envs: int) -> None:
        super().__init__()
        self.stats = FightStats(num_envs)

    def _on_step(self) -> bool:
        self.stats.update(self.locals["rewards"], self.locals["dones"])
        return True

    def _on_rollout_end(self) -> None:
        for key, value in self.stats.summary().items():
            self.logger.record(f"fights/{key}", value)


def main() -> None:
    parser = argparse.ArgumentParser(description="Train SB3's PPO on the arena.")
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--steps", type=int, default=5_000_000)
    args = parser.parse_args()
    out = f"runs/sb3/seed{args.seed}"

    env = ArenaVecEnv(NUM_ENVS, seed=args.seed * FIGHT_SEEDS_PER_RUN)
    model = PPO("MlpPolicy", env, seed=args.seed, device="cpu", **PPO_SETTINGS)
    model.set_logger(configure(out, ["stdout", "csv"]))
    model.learn(args.steps, callback=StatsCallback(NUM_ENVS), log_interval=LOG_INTERVAL)
    model.save(f"{out}/model")


if __name__ == "__main__":
    main()
