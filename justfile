# Display available commands and their descriptions (default target)
default:
    @just --list


## Workspace management

alias clean := cargo-clean

# Clean cargo workspace (cargo clean)
[group: 'workspace']
cargo-clean:
    cargo clean


## Code formatting and linting

alias format := fmt

# Format Rust code (cargo fmt --all)
[group: 'format']
fmt:
    cargo +nightly fmt --all

alias format-check := fmt-check

# Check Rust code format (cargo fmt --check)
[group: 'format']
fmt-check:
    cargo +nightly fmt --all -- --check


## Check

# Check Rust code (cargo check --workspace --all-targets)
[group: 'check']
check *EXTRA_FLAGS:
    cargo check --workspace --all-targets {{EXTRA_FLAGS}}

# Check specific crate with tests (cargo check -p <crate> --all-targets)
[group: 'check']
check-crate CRATE *EXTRA_FLAGS:
    cargo check --package {{CRATE}} --all-targets {{EXTRA_FLAGS}}

# Lint Rust code and reject warnings (cargo clippy --workspace --all-targets)
[group: 'check']
clippy *EXTRA_FLAGS:
    cargo clippy --workspace --all-targets {{EXTRA_FLAGS}} -- -D warnings

# Lint specific crate and reject warnings (cargo clippy -p <crate> --all-targets --no-deps)
[group: 'check']
clippy-crate CRATE *EXTRA_FLAGS:
    cargo clippy --package {{CRATE}} --all-targets --no-deps {{EXTRA_FLAGS}} -- -D warnings

alias check-deps := check-unused-deps

# Check for unused Rust dependencies (cargo machete)
[group: 'check']
check-unused-deps:
    cargo machete


## Testing

# Run all tests (cargo nextest run)
[group: 'test']
test *EXTRA_FLAGS:
    #!/usr/bin/env bash
    set -e # Exit on error

    if command -v "cargo-nextest" &> /dev/null; then
        cargo nextest run {{EXTRA_FLAGS}} --workspace --all-features
    else
        >&2 echo "================================================="
        >&2 echo "ERROR: This command requires 'cargo-nextest' ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-nextest to use this command:"
        >&2 echo "  cargo install --locked cargo-nextest@^0.9"
        >&2 echo "================================================="
        exit 1
    fi

# Run doctests, which nextest does not run (cargo test --doc)
[group: 'test']
test-doc *EXTRA_FLAGS:
    cargo test --doc --workspace --all-features {{EXTRA_FLAGS}}


## Docs

# Check governed documents and agent skills (lorecraft check)
[group: 'docs']
docs-check *EXTRA_FLAGS:
    {{env_var_or_default("LORECRAFT", "lorecraft")}} check --root . {{EXTRA_FLAGS}}

# Show the corpora, specifications, documents, and skills Lorecraft resolves (lorecraft inspect)
[group: 'docs']
docs-inspect *EXTRA_FLAGS:
    {{env_var_or_default("LORECRAFT", "lorecraft")}} inspect . {{EXTRA_FLAGS}}


## Release

# List files selected for the distributable Cargo packages (cargo package --list)
[group: 'release']
package-list *EXTRA_FLAGS:
    cargo package --workspace --list {{EXTRA_FLAGS}}

# Build and verify the distributable Cargo packages (cargo package)
[group: 'release']
package *EXTRA_FLAGS:
    cargo package --workspace {{EXTRA_FLAGS}}


## Misc

PRECOMMIT_CONFIG := ".github/pre-commit-config.yaml"
PRECOMMIT_DEFAULT_HOOKS := "pre-commit pre-push"

# Install Git hooks
[group: 'misc']
install-git-hooks HOOKS=PRECOMMIT_DEFAULT_HOOKS:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if pre-commit is installed
    if ! command -v "pre-commit" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'pre-commit' not available ❌"
        >&2 echo ""
        >&2 echo "Please install pre-commit using your preferred package manager"
        >&2 echo "  pip install pre-commit"
        >&2 echo "  pacman -S pre-commit"
        >&2 echo "  apt-get install pre-commit"
        >&2 echo "  brew install pre-commit"
        >&2 echo "=============================================================="
        exit 1
    fi

    # Install all Git hooks (see PRECOMMIT_DEFAULT_HOOKS for default hooks)
    pre-commit install --config {{PRECOMMIT_CONFIG}} {{replace_regex(HOOKS, "\\s*([a-z-]+)\\s*", "--hook-type $1 ")}}

# Remove Git hooks
[group: 'misc']
remove-git-hooks HOOKS=PRECOMMIT_DEFAULT_HOOKS:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if pre-commit is installed
    if ! command -v "pre-commit" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'pre-commit' not available ❌"
        >&2 echo ""
        >&2 echo "Please install pre-commit using your preferred package manager"
        >&2 echo "  pip install pre-commit"
        >&2 echo "  pacman -S pre-commit"
        >&2 echo "  apt-get install pre-commit"
        >&2 echo "  brew install pre-commit"
        >&2 echo "=============================================================="
        exit 1
    fi

    # Remove all Git hooks (see PRECOMMIT_DEFAULT_HOOKS for default hooks)
    pre-commit uninstall --config {{PRECOMMIT_CONFIG}} {{replace_regex(HOOKS, "\\s*([a-z-]+)\\s*", "--hook-type $1 ")}}

# Install cargo-nextest
[group: 'misc']
install-cargo-nextest:
    cargo install --locked cargo-nextest@^0.9

# Install Lorecraft (documentation and skill checker)
[group: 'misc']
install-lorecraft:
    uv tool install lorecraft

# Install cargo-machete (unused dependency checker)
[group: 'misc']
install-cargo-machete:
    cargo install --locked cargo-machete
