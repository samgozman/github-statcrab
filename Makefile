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
	cargo test --all --all-targets --workspace --all-features

# Generate language colors JSON from GitHub linguist
gen-language-colors:
	cargo run --bin generate_language_colors --features gen-language-colors

# Generate theme examples README from available themes
gen-themes-readme:
	cargo run --bin generate_themes_readme --features gen-themes-readme

# Run the server binary
run:
	cargo run --bin server -q

# Run the server in watch mode (requires `cargo install --locked watchexec-cli`)
run-watch:
	watchexec -r -w src -w Cargo.toml -e rs,toml -- cargo run --bin server -q
