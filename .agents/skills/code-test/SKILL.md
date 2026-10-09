---
name: code-test
description: Select and run Rust tests after formatting and Clippy pass. Use after changing UDF behavior or public APIs, when tests fail, or when asked about tests, coverage, or test effectiveness.
compatibility: Requires Rust and just. Coverage and mutation tools are optional and must already be available or separately authorized for installation.
allowed-tools: Bash(just test *) Bash(just test-doc *) Bash(just example) Bash(just bench *) Bash(cargo run *) Bash(cargo test *) Bash(cargo bench *) Bash(cargo llvm-cov *) Bash(cargo mutants *)
---

# Code Testing

Run `/code-format` and `/code-check` first. Inspect package manifests and existing targets before
selecting tests; do not assume an integration, property, example, or benchmark target exists.

| Change | Selection |
|---|---|
| Documentation or skill prose only | Skip Rust tests; run document and skill checks |
| One formula or input contract | Select the affected package, target, and test filter |
| Shared execution, public exports, or dependencies | Run `just test` |
| A runnable example | Run its existing Cargo example target |
| A benchmark harness | Smoke-check its existing Cargo benchmark target |

`just test` runs the workspace through cargo-nextest and forwards arguments to it. Use
`-p <package>` to focus a package or `--test <target>` for an existing integration target.
nextest skips doctests; run them with `just test-doc`.
A filter that runs zero tests does not verify a change. The bootstrap package contains only the
standard `cargo new --lib` unit test; it verifies the scaffold, not a vector API.

## Benchmarks

Run an existing benchmark only for a performance question or a change to its harness. Use the
recipe from `just --list` when one exists. Record dimensions, row count, toolchain, and baseline.
A debug smoke run supplies no performance evidence.

## Coverage

Use coverage when asked which code the tests reach or when investigating a concrete gap. If
`cargo-llvm-cov` is available, inspect `cargo llvm-cov --help`, then use its supported options, such as:

```bash
cargo llvm-cov --all-features
```

Coverage measures execution, not assertion strength. It is not a configured CI gate. If the tool is
missing, report that fact rather than silently installing it or substituting a made-up recipe.

## Mutation testing

Run mutation tests only for an explicit test-effectiveness investigation. They are expensive. Inspect
`cargo mutants --help` when the tool is available and select the affected files. Read
[references/mutation-testing.md](references/mutation-testing.md) before acting on survivors.

Do not broaden or rerun passing tests unless a new change, failure, or unresolved concern warrants it.
Report the actual commands, test counts, and any unsupported or skipped checks.
