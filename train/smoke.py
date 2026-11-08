"""Play random actions. Print the results of a random agent and the env speed."""

import time

import numpy as np
import tyro

import arena
from train.config import Config
from train.env import FightStats


def main() -> None:
    cfg = tyro.cli(Config, default=Config(num_envs=8, steps=8_000), description=__doc__)
    steps = cfg.steps // cfg.num_envs
    env = arena.VecEnv(cfg.num_envs, seed=cfg.env_seed)
    obs = env.reset()
    assert obs.shape == (env.num_envs, arena.OBS_LEN)
    rng = np.random.default_rng(cfg.seed)
    stats = FightStats(env.num_envs, window=steps)

    start = time.perf_counter()
    for _ in range(steps):
        actions = rng.integers(
            0, arena.ACTION_DIMS, size=(env.num_envs, len(arena.ACTION_DIMS))
        )
        obs, rewards, dones = env.step(actions)
        assert obs.shape == (env.num_envs, arena.OBS_LEN)
        assert np.isfinite(obs).all()
        stats.update(rewards, dones)
    seconds = time.perf_counter() - start

    summary = stats.summary()
    print(f"{summary['fights']:.0f} fights finished")
    print(f"mean reward: {summary['reward']:.1f}")
    print(f"mean length: {summary['length']:.0f} steps")
    print(f"win rate: {summary['win_rate']:.0%}")
    print(f"{env.num_envs * steps / seconds:,.0f} steps per second")


if __name__ == "__main__":
    main()
