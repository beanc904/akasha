.PHONY: mihomo-start run release

mihomo-start:
	mihomo -f config/config.yaml -ext-ctl-unix /tmp/akasha/mihomo.sock

run:
	cargo run

release:
	cargo build --release
