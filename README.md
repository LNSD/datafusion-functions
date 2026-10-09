# DataFusion Functions

A Rust workspace for Apache DataFusion extension functions.

The `datafusion-functions-vector` package at the workspace root currently contains the
standard `cargo new --lib` scaffold. Vector UDFs will be implemented in subsequent changes.

Run `just --list` for development commands. Rust formatting uses nightly rustfmt;
builds and linting use the stable toolchain. Documentation specifications live in
`docs/__meta__/`, and `AGENTS.md` describes the contributor workflow.

Licensed under MIT or Apache-2.0.
