---
name: code-format
description: Format Rust source with the repository's nightly rustfmt configuration. Use after editing .rs files, when formatting differs, or before linting and commits.
compatibility: Requires Rust with nightly rustfmt and the just task runner.
allowed-tools: Bash(just fmt) Bash(just fmt-check)
---

# Code Formatting

Run from the repository root:

```bash
just fmt
just fmt-check
```

The recipes format all Cargo targets. `.rustfmt.toml` uses nightly options, and the recipes select nightly explicitly.
Keep one rustfmt configuration; the release pinned in `rust-toolchain.toml` remains the build and lint
toolchain.

Format a coherent change before `/code-check`. Inspect the diff after formatting: report pre-existing
formatting changes outside the task instead of silently absorbing them. Reformat after a manual lint fix
if it changed source layout.

These recipes take no extra arguments. They do not format Markdown.
