---
name: code-test
description: Select and run Rust tests after formatting and Clippy pass. Use after changing UDF behavior or public APIs, when tests fail, or when asked about tests.
compatibility: Requires Rust, just, and cargo-nextest.
allowed-tools: Bash(just test *) Bash(just test-doc *)
---

# Code Testing

Run `/code-format` and `/code-check` first. Inspect package manifests and existing targets before
selecting tests; do not assume an integration, property, example, or benchmark target exists.

| Change | Selection |
|---|---|
| Documentation or skill prose only | Skip Rust tests; run document and skill checks |
| One formula or input contract | Select the affected package, target, and test filter |
| Shared execution, public exports, or dependencies | Run `just test` |
| A runnable example or a benchmark harness | Run its recipe from `just --list`; without one, add the recipe first |

`just test` runs the workspace through cargo-nextest and forwards arguments to it. Use
`-p <package>` to focus a package or `--test <target>` for an existing integration target.
nextest skips doctests; run them with `just test-doc`.
A filter that runs zero tests does not verify a change.

## Benchmarks

Run a benchmark only for a performance question or a change to its harness, through its recipe from
`just --list`. Record dimensions, row count, toolchain, and baseline. A debug smoke run supplies no
performance evidence.

Do not broaden or rerun passing tests unless a new change, failure, or unresolved concern warrants it.
Report the actual commands, test counts, and any unsupported or skipped checks.
