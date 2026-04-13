.PHONY: mihomo-start run release

mihomo-start:
	mihomo -f resources/config.yaml -ext-ctl-unix /tmp/akasha/mihomo.sock

run:
	cargo run

release:
	cargo build --release
