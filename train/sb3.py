"""Train the ready-made SB3 PPO on the arena: the reference curve for `train/ppo.py`."""

from dataclasses import asdict

import numpy as np
import tyro
from gymnasium import spaces
from stable_baselines3 import PPO
from stable_baselines3.common.callbacks import BaseCallback
from stable_baselines3.common.logger import configure
from stable_baselines3.common.vec_env import VecEnv

import arena
from train.config import Config
from train.env import FightStats


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
    cfg = tyro.cli(Config, description="Train SB3's PPO on the arena.")
    out = f"runs/sb3/seed{cfg.seed}"
    cfg.save(out)

    env = ArenaVecEnv(cfg.num_envs, seed=cfg.env_seed)
    # Same hidden layers as `Agent`, so that only the PPO code differs.
    policy_kwargs = {"net_arch": [cfg.hidden, cfg.hidden]}
    model = PPO(
        "MlpPolicy",
        env,
        policy_kwargs=policy_kwargs,
        seed=cfg.seed,
        device="cpu",
        **asdict(cfg.ppo),
    )
    model.set_logger(configure(out, ["stdout", "csv"]))
    model.learn(
        cfg.steps, callback=StatsCallback(cfg.num_envs), log_interval=cfg.log_interval
    )
    model.save(f"{out}/model")


if __name__ == "__main__":
    main()
