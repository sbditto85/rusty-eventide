
.PHONY: test
test:
	cargo nextest run


.PHONY: install-test
install-test:
	cargo install cargo-nextest

.PHONY: b
b:
	cargo build

.PHONY: br
br:
	cargo build --release