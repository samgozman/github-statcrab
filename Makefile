# Check if the code is formatted correctly
fmt-check:
	cargo fmt -- --check

# Format the code
fmt:
	cargo fmt --all

# Run the clippy linter check
clippy-check:
	cargo clippy --locked --all-targets --all-features --workspace -- -D warnings

# Run the clippy linter and fix some issues
clippy-fix:
	cargo clippy --fix --allow-dirty --allow-staged --locked --all-targets --all-features --workspace

# Run the tests
test:
	cargo test --locked --all-targets --workspace --all-features
