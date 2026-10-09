# List available commands.
default:
    @just --list

# Format Rust code with the repository's nightly rustfmt settings.
fmt:
    cargo +nightly fmt --all

# Check formatting without changing files.
fmt-check:
    cargo +nightly fmt --all -- --check

# Compile every workspace target.
check *FLAGS:
    cargo check --workspace --all-targets {{FLAGS}}

# Lint every workspace target and reject warnings.
clippy *FLAGS:
    cargo clippy --workspace --all-targets {{FLAGS}} -- -D warnings

# Run workspace tests and doctests; accepts Cargo test filters.
test *FLAGS:
    cargo test --workspace {{FLAGS}}

# Check for unused dependencies (requires cargo-machete).
check-unused-deps:
    cargo machete

# Check governed documents and agent skills (requires Lorecraft).
check-docs *FLAGS:
    {{env_var_or_default("LORECRAFT", "lorecraft")}} check --root . {{FLAGS}}

# List files selected for the distributable Cargo packages.
package-list *FLAGS:
    cargo package --workspace --list {{FLAGS}}

# Build and verify the distributable Cargo packages.
package *FLAGS:
    cargo package --workspace {{FLAGS}}

# Remove build artifacts.
clean:
    cargo clean

# Run local formatting, compilation, linting, and tests.
ci: fmt-check check clippy test
