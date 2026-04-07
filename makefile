.PHONY: run build

run: build
	cargo run -p custodex

build:
	cargo build
