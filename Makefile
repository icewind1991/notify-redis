all: target/x86_64-unknown-linux-musl/release/notify-redis

target/x86_64-unknown-linux-musl/release/notify-redis: Cargo.toml src/main.rs
	docker run --rm -it -v "$(CURDIR):/home/rust/src" ekidd/rust-musl-builder cargo build --release

.PHONY: test

test: target/x86_64-unknown-linux-musl/release/notify-redis
	./test.sh