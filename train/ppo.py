"""Train our own PPO on the arena."""

from dataclasses import dataclass

import torch
from torch.optim import Optimizer
import tyro
from numpy import bool_, float32
from numpy.typing import NDArray
from torch import Tensor

import arena
from train.agent import Agent
from train.config import Config
from train.env import FightStats


@dataclass
class Batch:
    """One rollout. Dimension 0 is the step, dimension 1 is the env."""

    obs: Tensor  # the obs that each action was chosen in
    actions: Tensor
    log_probs: Tensor
    values: Tensor
    rewards: Tensor
    dones: Tensor  # 1.0 if the fight ended in this step
    next_value: Tensor  # (num_envs,): value of the obs after the last step

    @classmethod
    def empty(cls, n_steps: int, num_envs: int) -> "Batch":
        """Return a batch of zeros."""
        return cls(
            obs=torch.zeros(n_steps, num_envs, arena.OBS_LEN),
            actions=torch.zeros(
                n_steps, num_envs, len(arena.ACTION_DIMS), dtype=torch.long
            ),
            log_probs=torch.zeros(n_steps, num_envs),
            values=torch.zeros(n_steps, num_envs),
            rewards=torch.zeros(n_steps, num_envs),
            dones=torch.zeros(n_steps, num_envs),
            next_value=torch.zeros(num_envs),
        )

    def store(
        self,
        t: int,
        obs: Tensor,
        action: Tensor,
        log_prob: Tensor,
        value: Tensor,
        reward: NDArray[float32],
        done: NDArray[bool_],
    ) -> None:
        """Fill row `t`."""
        self.obs[t] = obs
        self.actions[t] = action
        self.log_probs[t] = log_prob
        self.values[t] = value
        self.rewards[t] = torch.as_tensor(reward)
        self.dones[t] = torch.as_tensor(done)


# Based on: https://github.com/vwxyzjn/cleanrl/blob/master/cleanrl/ppo.py
@torch.no_grad()
def collect(
    env: arena.VecEnv,
    agent: Agent,
    obs: NDArray[float32],
    stats: FightStats,
    n_steps: int,
) -> tuple[Batch, NDArray[float32]]:
    """Play `n_steps` in every env. Return the batch and the obs to continue from."""
    batch = Batch.empty(n_steps, env.num_envs)
    for t in range(n_steps):
        obs_t = torch.from_numpy(obs)
        action, log_prob, _, value = agent.act(obs_t)
        obs, reward, done = env.step(action.numpy())
        stats.update(reward, done)
        batch.store(t, obs_t, action, log_prob, value, reward, done)
    batch.next_value = agent.value(torch.from_numpy(obs))
    return batch, obs


# Based on: https://arxiv.org/abs/1506.02438
def gae(batch: Batch, gamma: float, lambda_: float) -> tuple[Tensor, Tensor]:
    """Return the advantage and the return of each step."""
    masks = 1 - batch.dones  # 0 cuts the sum where a fight ended
    next_values = torch.cat([batch.values[1:], batch.next_value[None]])
    deltas = batch.rewards + gamma * next_values * masks - batch.values

    advantages = torch.zeros_like(deltas)
    next_adv = torch.zeros_like(batch.next_value)
    for t in reversed(range(len(deltas))):
        next_adv = deltas[t] + gamma * lambda_ * masks[t] * next_adv
        advantages[t] = next_adv
    return advantages, advantages + batch.values


def learn(
    agent: Agent,
    optimizer: Optimizer,
    batch: Batch,
    advantages: Tensor,
    returns: Tensor,
) -> None:
    return


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
        batch, obs = collect(env, agent, obs, stats, cfg.ppo.n_steps)
        advantages, returns = gae(batch, cfg.ppo.gamma, cfg.ppo.gae_lambda)
        learn(agent, optimizer, batch, advantages, returns)
        print(update, stats.summary())


if __name__ == "__main__":
    main()
