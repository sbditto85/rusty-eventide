
.PHONY: test
test:
	cargo nextest run

.PHONY: it
it:
	cargo nextest run --features integration

.PHONY: tnc
tnc:
	RUST_LOG=debug cargo nextest run --no-capture

.PHONY: examples
examples:
	cargo build --example simple_consumer

.PHONY: simple_consumer
simple_consumer:
	cargo run --example simple_consumer


.PHONY: install-test
install-test:
	cargo install cargo-nextest

.PHONY: cl
cl:
	cargo clean

.PHONY: c
c:
	cargo check

.PHONY: cr
cr:
	cargo check --release

.PHONY: b
b:
	cargo build

.PHONY: br
br:
	cargo build --release