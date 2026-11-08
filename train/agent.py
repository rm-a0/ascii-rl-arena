import torch
from torch import Tensor, nn, stack
from torch.distributions import Categorical

import arena


class Agent(nn.Module):
    def __init__(self, hidden: int) -> None:
        super().__init__()
        state_dim = arena.OBS_LEN
        act_dim = sum(arena.ACTION_DIMS)
        self.actor = nn.Sequential(
            nn.Linear(state_dim, hidden),
            nn.Tanh(),
            nn.Linear(hidden, hidden),
            nn.Tanh(),
            nn.Linear(hidden, act_dim),
        )
        self.critic = nn.Sequential(
            nn.Linear(state_dim, hidden),
            nn.Tanh(),
            nn.Linear(hidden, hidden),
            nn.Tanh(),
            nn.Linear(hidden, 1),
        )

    def value(self, obs: Tensor) -> Tensor:
        return self.critic(obs).squeeze(-1)

    def act(
        self,
        obs: Tensor,
        action: Tensor | None = None,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor]:
        logits = self.actor(obs)
        chunks = torch.split(logits, arena.ACTION_DIMS, dim=1)
        dists = [Categorical(logits=chunk) for chunk in chunks]

        if action is None:
            action = stack([dist.sample() for dist in dists], dim=1)

        log_prob = stack(
            [dist.log_prob(action[:, i]) for i, dist in enumerate(dists)]
        ).sum(dim=0)
        entropy = stack([dist.entropy() for dist in dists]).sum(dim=0)

        return action, log_prob, entropy, self.value(obs)
