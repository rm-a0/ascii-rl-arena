# Short commands for the workspace. Run `make` to list them.
# Pass preview flags with ARGS, for example: make run ARGS="--stage lake"

ARGS ?=

.DEFAULT_GOAL := help
.PHONY: help build run kitty boxes test lint fmt clean

help: ## List the commands.
	@grep -E '^[a-z]+:.*## ' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  make %-6s %s\n", $$1, $$2}'

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

clean: ## Remove build output.
	cargo clean
