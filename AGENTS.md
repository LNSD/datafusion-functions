# DataFusion Functions - Agent Guide

This repository is a Rust workspace for Apache DataFusion extension functions. The initial
`datafusion-functions-vector` package is a `cargo new --lib` scaffold at the workspace root.

## Working Rules

- Keep code small, readable, and explicit. Readability outranks cleverness.
- Inspect existing files before planning a change. Update this guide when a change makes it untrue.
- Direct user instructions take precedence, followed by repository-local skills, this guide,
  applicable documents under `docs/`, and tool defaults.
- Use repository-local skills for the workflows they cover and `just` for their commands.
- Use `gh` for GitHub queries and operations.
- Create commits, tags, releases, or pull requests only when requested. Never add AI attribution.

## Workspace

The root `Cargo.toml` contains workspace configuration and shared package metadata. Library packages
live at the workspace root, with the package name as the directory name; do not apply another project's
package prefix. Members are listed alphabetically. Package manifests inherit shared metadata.

`rust-toolchain.toml` selects stable Rust for development. `.rustfmt.toml` contains nightly settings;
format through `just fmt` and `just fmt-check`. Keep the workspace MSRV and CI matrix consistent.
Inspect `.gitignore` before assuming whether build output or lockfiles are tracked.

`justfile` is the command authority. Use `just --list` to discover recipes; do not duplicate their
implementation in documentation. The bootstrap has no runtime dependencies, vector UDFs, examples,
integration tests, or benchmarks. Add behavior and its documentation together.

## Documentation and Skills

`docs/__meta__/` holds the base format specifications for feature documentation and code rules.
Feature documents belong in `docs/feat/` and describe implemented behavior. Code rules belong in
`docs/code/` and are binding when present. Do not invent rules from an absent corpus.

Skills live in `.agents/skills/`; `.claude/skills` points there, and `CLAUDE.md` points to this guide.
Keep compatibility symlinks relative and inside the checkout. No skill may depend on a symlink
pointing outside the repository.

Load `code-rules` before implementation, `docs-rules` before writing governed documents, and
`docs-rules-creator` before changing their specifications. Use `skills-check` before editing skills.
Use `code-rules-check`, `docs-rules-check`, and `code-review` for their respective review scopes.
The `commit` skill owns intent-focused messages, signing, DCO sign-off, and attribution policy.

## Development Workflow

1. Inspect affected files and load applicable rules.
2. Implement the smallest correct change and update the documentation it makes untrue.
3. Use `code-format`, then `code-check`. Fix findings instead of weakening gates.
4. Use `code-test` to select tests for the affected behavior. A filter running zero tests is no evidence.
5. After changing documents or skills, run `just check-docs` and inspect every finding.
6. Report the result and any validation that could not run.

Lorecraft is separate development tooling, not a crate dependency. `check-docs` uses `lorecraft`
by default; set `LORECRAFT='uvx lorecraft'` or a command path when needed. Do not install Python
packages into the system interpreter. Missing specifications mean a corpus is unvalidated.

## Commits and Pull Requests

Commit only when requested, with signing and DCO sign-off through the `commit` skill. Never add
AI attribution or session links. For a single-commit PR, use its commit title and body, strip the
`Signed-off-by:` trailer, and append issue links only to the PR description.

Use the user's structured local branch names and `lnsd/` remote namespace for development branches.
Do not push or publish merely because local validation passed.
