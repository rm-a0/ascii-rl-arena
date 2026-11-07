"""Train our own PPO on the arena."""

import torch
import tyro

import arena
from train.agent import Agent
from train.config import Config
from train.env import FightStats


def main() -> None:
    cfg = tyro.cli(Config, description="Train our own PPO on the arena.")
    out = f"runs/ppo/seed{cfg.seed}"
    cfg.save(out)

    env = arena.VecEnv(cfg.num_envs, seed=cfg.env_seed)
    agent = Agent(cfg.hidden)
    optimizer = torch.optim.Adam(agent.parameters(), lr=cfg.ppo.learning_rate)
    stats = FightStats(cfg.num_envs)
    obs = env.reset()

    for update in range(cfg.num_updates):
        # batch, obs = collect(env, agent, obs, stats)      # 1. play
        # advantages, returns = gae(batch)                   # 2. score
        # learn(agent, optimizer, batch, advantages, returns)  # 3. improve
        print(update, stats.summary())


if __name__ == "__main__":
    main()
