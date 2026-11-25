"""Train our own PPO on the arena."""

import csv
import time
from dataclasses import dataclass

import torch
import torch.nn.functional as F
import tyro
from numpy import bool_, float32
from numpy.typing import NDArray
from torch import Tensor, nn
from torch.optim import Optimizer

import arena
from train.agent import Agent
from train.config import Config, PPOSettings
from train.env import FightStats

# Columns of `progress.csv`: one row per update.
LOG_COLUMNS = (
    "steps", "time", "fights", "reward", "length", "win_rate",
    "pg_loss", "v_loss", "entropy", "clip_frac",
)  # fmt: skip


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


# Based on: https://arxiv.org/abs/1707.06347
def learn(
    agent: Agent,
    optimizer: Optimizer,
    batch: Batch,
    advantages: Tensor,
    returns: Tensor,
    ppo_cfg: PPOSettings,
) -> dict[str, float]:
    """Train on one rollout for `n_epochs`. Return the stats of the last minibatch."""
    b_obs = batch.obs.flatten(0, 1)
    b_actions = batch.actions.flatten(0, 1)
    b_log_probs = batch.log_probs.flatten(0, 1)
    b_advantages = advantages.flatten(0, 1)
    b_returns = returns.flatten(0, 1)
    clip = ppo_cfg.clip_range

    for _ in range(ppo_cfg.n_epochs):
        perm = torch.randperm(len(b_obs))
        for start in range(0, len(b_obs), ppo_cfg.batch_size):
            mb_idx = perm[start : start + ppo_cfg.batch_size]

            _, new_log_prob, entropy, new_value = agent.act(
                b_obs[mb_idx], b_actions[mb_idx]
            )

            mb_advantages = b_advantages[mb_idx]
            mb_advantages = (mb_advantages - mb_advantages.mean()) / (
                mb_advantages.std() + 1e-8
            )
            ratio = (new_log_prob - b_log_probs[mb_idx]).exp()

            unclipped = ratio * mb_advantages
            clipped = ratio.clamp(1 - clip, 1 + clip) * mb_advantages
            pg_loss = -torch.min(unclipped, clipped).mean()
            v_loss = F.mse_loss(new_value, b_returns[mb_idx])
            loss = (
                pg_loss + ppo_cfg.vf_coef * v_loss - ppo_cfg.ent_coef * entropy.mean()
            )

            optimizer.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(agent.parameters(), ppo_cfg.max_grad_norm)
            optimizer.step()

    return {
        "pg_loss": pg_loss.item(),
        "v_loss": v_loss.item(),
        "entropy": entropy.mean().item(),
        "clip_frac": ((ratio - 1).abs() > clip).float().mean().item(),
    }


def main() -> None:
    cfg = tyro.cli(Config, description="Train our own PPO on the arena.")
    cfg.save()
    torch.manual_seed(cfg.seed)

    env = arena.VecEnv(cfg.num_envs, seed=cfg.env_seed)
    agent = Agent(cfg.hidden)
    optimizer = torch.optim.Adam(agent.parameters(), lr=cfg.ppo.learning_rate)
    stats = FightStats(cfg.num_envs)
    obs = env.reset()

    start = time.time()
    with open(f"{cfg.out}/progress.csv", "w", newline="") as file:
        log = csv.DictWriter(file, LOG_COLUMNS)
        log.writeheader()
        for update in range(cfg.num_updates):
            batch, obs = collect(env, agent, obs, stats, cfg.ppo.n_steps)
            advantages, returns = gae(batch, cfg.ppo.gamma, cfg.ppo.gae_lambda)
            info = learn(agent, optimizer, batch, advantages, returns, cfg.ppo)

            steps = (update + 1) * cfg.num_envs * cfg.ppo.n_steps
            row = {"steps": steps, "time": time.time() - start} | stats.summary()
            log.writerow(row | info)
            if update % cfg.log_interval == 0:
                print(" ".join(f"{k}={v:.4g}" for k, v in row.items()))

    torch.save(agent.state_dict(), f"{cfg.out}/model.pt")


if __name__ == "__main__":
    main()
