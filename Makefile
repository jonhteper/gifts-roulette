all: prepare release

prepare:
	rm -R bin || true
	mkdir bin

dev:
	cargo fmt
	cargo build
	cp target/debug/gifts-roulette bin/

release:
	cargo fmt
	cargo build --release
	cp target/release/gifts-roulette bin/