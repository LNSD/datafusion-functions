---
name: "adr-012-shell-command-parser"
description: "Parse shell command lines using pure-Rust POSIX parsing with task-runner awareness and SQL null semantics"
type: "adr"
status: "proposed"
---

# Shell Command Parser and Taxonomic Classification

## Context

Analyzing agent command lines requires decomposing composite shell pipelines and evaluating command intent without external side effects ([prd-011](prd-011-shell-command-analysis.md)). Agent commands frequently combine utilities via chaining, pipelines, redirections, and wrappers. In DataFusion, scalar UDFs must adhere to standard null propagation, provide pure in-memory execution, and avoid requiring specialized non-Rust toolchains or C compiler dependencies in downstream crates where feasible.

## Decision

1. **Pure-Rust POSIX parsing baseline.** The parsing engine implements shell parsing in 100% safe, pure Rust (evaluating against `conch-parser` or an internal zero-allocation lexer/parser), while maintaining benchmark parity and comparing parse coverage against `tree-sitter-bash` (answers [NFR-001](prd-011-shell-command-analysis.md#requirements), [NFR-002](prd-011-shell-command-analysis.md#requirements)).
2. **Per-command structural representation.** Each decomposed command in `shell_commands` is emitted as an Arrow struct containing `program: Utf8`, `args: List<Utf8>`, `wrappers: List<Utf8>`, `nesting: UInt32`, `operator: Utf8`, and its own operational `kind: Utf8` (answers [FR-001](prd-011-shell-command-analysis.md#requirements), [FR-002](prd-011-shell-command-analysis.md#requirements)).
3. **Shell invocation unwrapping.** Literal shell wrappers (`sh -c`, `bash -c`, `zsh -c`, and common flag variations such as `-lc`, `-ec`) are unwrapped, recursing into the inner command string. Container and remote execution commands (`docker exec`, `kubectl exec`, `ssh`, `xargs`) remain outer command invocations (answers [FR-003](prd-011-shell-command-analysis.md#requirements)).
4. **Hierarchical precedence classification.** The line-level scalar function `shell_command_kind` evaluates the highest-precedence command kind across the line based on a fixed compiled-in taxonomy table (delete > publish > edit > install > vcs > network > test > build > run > search > read > list > shell > unknown) (answers [FR-004](prd-011-shell-command-analysis.md#requirements), [FR-005](prd-011-shell-command-analysis.md#requirements)).
5. **Task-runner target classification.** For standard task runners and build systems (`just`, `make`, `npm run`, `pnpm run`, `cargo`), the primary subcommand or recipe target is inspected and classified into semantic categories (e.g., `just test` maps to `test`, `make check` maps to `test`, `just build` maps to `build`) (answers [FR-006](prd-011-shell-command-analysis.md#requirements)).
6. **Configurable registration extension.** The default precedence table is statically compiled into `datafusion-functions-shell`, with optional programmatic configuration hooks during `SessionContext` UDF registration to extend or override rules for proprietary binaries.
7. **Structural fingerprinting.** `shell_fingerprint` reduces command lines to executable names, known subcommands, and flags, collapsing variable arguments and file paths to placeholder markers (`?`) (answers [FR-007](prd-011-shell-command-analysis.md#requirements), [FR-008](prd-011-shell-command-analysis.md#requirements)).
8. **SQL null semantics on malformed inputs.** In conformance with SQL three-valued logic and [adr-003](adr-003-jsonschema-null-semantics.md), any input string that is null, syntactically truncated, or fails parse validation produces `NULL` across all shell scalar functions (answers [FR-009](prd-011-shell-command-analysis.md#requirements), [FR-010](prd-011-shell-command-analysis.md#requirements)).

## Consequences

- The crate `datafusion-functions-shell` compiles cleanly without mandatory C toolchain requirements in CI.
- Downstream SQL queries can query per-command items via `UNNEST(shell_commands(cmd))` or evaluate high-level session activity via `shell_command_kind(cmd)`.
- Malformed and truncated agent traces cleanly yield `NULL`, allowing queries to filter or flag unparseable commands using `WHERE shell_command_kind(cmd) IS NULL`.

## References

- [prd-011](prd-011-shell-command-analysis.md) - Source: Product requirements for shell command analysis functions
- [adr-003](adr-003-jsonschema-null-semantics.md) - Precedent: Null semantics for unassessable inputs in DataFusion scalar UDFs
