"""Test every trained model in `runs/` on fights that no training run played."""

import json
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import torch
import tyro
from numpy.typing import NDArray

import arena
from train.agent import Agent

# Seed of the first test fight. Training seeds count up from seed * 10^9, far below.
TEST_SEED = 2**62

Policy = Callable[[NDArray[np.float32]], NDArray[np.int64]]


@dataclass(frozen=True)
class EvalConfig:
    """Settings of the test."""

    fights: int = 1000  # test fights per model, each with its own seed
    runs: str = "runs"  # folder with one `<name>/seed<k>` folder per run


def load(run: Path, greedy: bool) -> Policy:
    """Return the policy of the model in `run`. Greedy takes the likeliest option."""
    if (run / "model.zip").exists():
        from stable_baselines3 import PPO

        model = PPO.load(run / "model.zip", device="cpu")
        return lambda obs: model.predict(obs, deterministic=greedy)[0]

    hidden = json.loads((run / "config.json").read_text())["hidden"]
    agent = Agent(hidden)
    agent.load_state_dict(torch.load(run / "model.pt"))

    @torch.no_grad()
    def policy(obs: NDArray[np.float32]) -> NDArray[np.int64]:
        obs_t = torch.from_numpy(obs)
        if not greedy:
            return agent.act(obs_t)[0].numpy()
        chunks = torch.split(agent.actor(obs_t), arena.ACTION_DIMS, dim=1)
        return torch.stack([chunk.argmax(dim=1) for chunk in chunks], dim=1).numpy()

    return policy


def play(policy: Policy, fights: int) -> dict[str, float]:
    """Play each test fight once. Return the win rate and the mean reward and length."""
    torch.manual_seed(0)  # the same samples for every model
    env = arena.VecEnv(fights, seed=TEST_SEED)
    obs = env.reset()
    rewards = np.zeros(fights)
    lengths = np.zeros(fights)
    wins = np.zeros(fights, dtype=bool)
    running = np.ones(fights, dtype=bool)
    # An env that is done starts a new fight. Count only its first one.
    while running.any():
        obs, reward, done = env.step(policy(obs))
        rewards += reward * running
        lengths += running
        ended = done & running
        wins[ended] = reward[ended] > 0
        running &= ~done
    return {
        "win_rate": float(wins.mean()),
        "reward": float(rewards.mean()),
        "length": float(lengths.mean()),
    }


def spread(values: list[float]) -> str:
    """Return the mean and the standard deviation over seeds as text."""
    std = np.std(values, ddof=1) if len(values) > 1 else 0.0
    return f"{np.mean(values):6.2f} ± {std:4.2f}"


def main() -> None:
    cfg = tyro.cli(EvalConfig, description=__doc__)
    results: dict[str, list[dict[str, dict[str, float]]]] = {}
    for run in sorted(Path(cfg.runs).glob("*/seed*")):
        if not any((run / model).exists() for model in ("model.pt", "model.zip")):
            continue  # still training
        result = {
            mode: play(load(run, greedy=mode == "greedy"), cfg.fights)
            for mode in ("greedy", "sampled")
        }
        (run / "eval.json").write_text(json.dumps(result, indent=2))
        results.setdefault(run.parent.name, []).append(result)
        print(f"{run}: win rate {result['greedy']['win_rate']:.2f} greedy")

    print(f"\n{cfg.fights} test fights per model, mean ± std over seeds")
    print(f"{'method':8} {'seeds':>5}  {'win greedy':>13}  {'win sampled':>13}  {'reward':>13}")
    for name, runs in results.items():
        greedy = [r["greedy"] for r in runs]
        sampled = [r["sampled"] for r in runs]
        print(
            f"{name:8} {len(runs):5}  {spread([r['win_rate'] for r in greedy]):>13}"
            f"  {spread([r['win_rate'] for r in sampled]):>13}"
            f"  {spread([r['reward'] for r in greedy]):>13}"
        )


if __name__ == "__main__":
    main()
