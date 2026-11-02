"""Step random actions through the env. Print how a random agent does and the speed."""

import time

import numpy as np

import arena

NUM_ENVS = 8
STEPS = 1000
SEED = 0


def main() -> None:
    env = arena.VecEnv(NUM_ENVS, seed=SEED)
    obs = env.reset()
    assert obs.shape == (env.num_envs, arena.OBS_LEN)
    rng = np.random.default_rng(SEED)

    # Reward and length of the fight that runs in each env now.
    returns = np.zeros(env.num_envs, dtype=np.float32)
    lengths = np.zeros(env.num_envs, dtype=np.int64)
    finished_returns: list[float] = []
    finished_lengths: list[int] = []

    start = time.perf_counter()
    for _ in range(STEPS):
        actions = rng.integers(
            0, arena.ACTION_DIMS, size=(env.num_envs, len(arena.ACTION_DIMS))
        )
        obs, rewards, dones = env.step(actions)
        assert obs.shape == (env.num_envs, arena.OBS_LEN)
        assert np.isfinite(obs).all()

        returns += rewards
        lengths += 1
        finished_returns.extend(returns[dones].tolist())
        finished_lengths.extend(lengths[dones].tolist())
        returns[dones] = 0.0
        lengths[dones] = 0
    seconds = time.perf_counter() - start

    print(f"{len(finished_returns)} fights finished")
    print(f"mean reward: {np.mean(finished_returns):.1f}")
    print(f"mean length: {np.mean(finished_lengths):.0f} steps")
    print(f"{env.num_envs * STEPS / seconds:,.0f} steps per second")


if __name__ == "__main__":
    main()
