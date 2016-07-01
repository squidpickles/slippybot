default: build

build:
	cargo build --release

run:
	env RUST_LOG=slippybot=debug target/release/slippybot

install:
	systemctl --user enable slippybot
