# Short commands for the workspace. Run `make` to list them.
# Pass preview flags with ARGS, for example: make run ARGS="--stage lake"

ARGS ?=

.DEFAULT_GOAL := help
.PHONY: help build run kitty boxes test lint fmt py smoke sb3 clean

help: ## List the commands.
	@grep -E '^[a-z0-9]+:.*## ' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  make %-6s %s\n", $$1, $$2}'

build: ## Build every crate and example.
	cargo build --workspace --all-targets

run: ## Run the preview in this terminal.
	cargo run -p arena-render --example preview -- $(ARGS)

kitty: ## Run the preview in a full-screen kitty window that fits the grid.
	scripts/preview.sh $(ARGS)

boxes: ## Print every move's hitbox over the fighter art.
	cargo run -q -p arena-render --example boxes

test: ## Run all tests.
	cargo test --workspace

lint: ## Check formatting and run clippy.
	cargo fmt --all --check
	cargo clippy --workspace --all-targets

fmt: ## Format all code.
	cargo fmt --all

py: ## Build the Python module `arena` into .venv.
	uv sync

smoke: ## Step random actions through the Python env and print the speed.
	uv run python train/smoke.py

sb3: ## Train the ready-made SB3 PPO to check the env. Pass flags with ARGS.
	uv run --group baseline python train/sb3_check.py $(ARGS)

clean: ## Remove build output.
	cargo clean
