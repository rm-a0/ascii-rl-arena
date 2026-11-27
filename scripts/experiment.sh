#!/usr/bin/env bash
# Train PPO, the actor-critic and SB3 on each seed, then test and plot them.
# Usage: scripts/experiment.sh [steps per run]. Default: 10M steps.
# A short check: SEEDS=0 scripts/experiment.sh 500000
set -euo pipefail
cd "$(dirname "$0")/.."

STEPS=${1:-10000000}
SEEDS=${SEEDS:-0 1 2 3 4}
JOBS=${JOBS:-4} # runs at the same time
export OMP_NUM_THREADS=1 # one core per run

for seed in $SEEDS; do
    echo "train.ppo --seed $seed --steps $STEPS"
    # One gradient step on the whole rollout: the ratio stays 1 and the clip never acts.
    echo "train.ppo --name ac --seed $seed --steps $STEPS --ppo.n-epochs 1 --ppo.batch-size 4096"
    echo "train.sb3 --seed $seed --steps $STEPS"
done | xargs -P "$JOBS" -I{} sh -c 'uv run python -m {} > /dev/null && echo "done: {}"'

uv run python -m train.eval
uv run python -m train.plot
