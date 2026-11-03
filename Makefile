.PHONY: test lint typecheck fmt run

run:
	cd crates/btlock-app && npm run tauri dev

test:
	cargo test --workspace

lint:
	cargo clippy --workspace --all-targets -- -D warnings

typecheck:
	cargo check --workspace --all-targets

fmt:
	cargo fmt --all
