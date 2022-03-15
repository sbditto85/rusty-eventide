
.PHONY: test
test:
	cargo nextest run

.PHONY: it
it:
	cargo nextest run --features integration_tests

.PHONY: it_full
it_full:
	make mdb && make it && make mdb_down

.PHONY: all_tests
all_tests:
	cargo nextest run --features all_tests

.PHONY: mdb
mdb:
	podman-compose up -d

.PHONY: mdb_down
mdb-down:
	podman-compose down

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