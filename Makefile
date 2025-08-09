# Check if the code is formatted correctly
fmt-check:
	cargo fmt -- --check

# Run the clippy linter check
clippy-check:
	cargo clippy --locked --all-targets --all-features --workspace -- -D warnings

# Run the tests
test:
	cargo test --locked --all-targets --workspace --all-features
