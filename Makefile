
.PHONY: test
test:
	cargo nextest run


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