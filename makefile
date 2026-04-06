.PHONY: run build sandbox

run: build
	cargo run -p code-agent

build:
	cargo build

sandbox:
	cargo build --manifest-path ../codex/codex-rs/Cargo.toml -p codex-linux-sandbox --release
