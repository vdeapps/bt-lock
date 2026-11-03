.PHONY: test lint typecheck fmt

test:
	cargo test --workspace

lint:
	cargo clippy --workspace --all-targets -- -D warnings

typecheck:
	cargo check --workspace --all-targets

fmt:
	cargo fmt --all
