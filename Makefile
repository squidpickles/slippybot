default: build

build:
	cargo build --release

run:
	target/release/slippybot

install:
	systemctl --user enable slippybot
