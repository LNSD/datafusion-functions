---
name: "rust-mods-members"
description: "Member ordering within a module: main API first, errors after the function that returns them. Load when organizing module contents"
type: "core"
scope: "global"
---

# Module Member Ordering

**A module reads top-down: its public interface first, then the code that supports it.** A reader meets the
API before the machinery. The prologue that precedes these items is owned by [rust-imports](rust-imports.md).
This document owns the order of the items that follow it.

## 1. The Public Interface Comes First

Order module members as follows:

1. **Module prologue**: doc comment, imports, `mod` declarations, re-exports ([rust-imports](rust-imports.md))
2. **Constants and statics** (`const`, `static`)
3. **Type aliases** (`type Foo = ...`)
4. **Main module members**: the public types, their constructors, and their trait implementations, such as a
   UDF struct, its `new`, and its `impl ScalarUDFImpl`
5. **Helper types and functions**, in dependency order ([§3](#3-helpers-follow-the-code-that-calls-them))

```rust
// ❌ Bad — the reader meets a private struct and a kernel before the UDF callers register.
struct NormPair { /* ... */ }
fn l2_norm(values: &[f32]) -> f32 { /* ... */ }
pub struct CosineDistance { signature: Signature }
```

```rust
// ✅ Good — the UDF, its `ScalarUDFImpl`, then the machinery `invoke_with_args` calls.
pub struct CosineDistance { signature: Signature }

impl ScalarUDFImpl for CosineDistance { /* ... */ }

fn l2_norm(values: &[f32]) -> f32 { /* ... */ }
```

## 2. An Error Type Follows the Function That Returns It

An error type the package declares sits immediately after the function or `impl` that returns it, never
before it and never in a separate module. The function is what the reader came for; its error is the detail
they need second. An adjacent pair makes a change to a failure path a single-file edit. An `error.rs` module
that collects every error type separates every pair.

```rust
// ❌ Bad — the reader meets the variants before learning what produces them.
#[derive(Debug)]
pub enum UnknownMetricError { /* ... */ }

impl std::str::FromStr for DistanceMetric {
    type Err = UnknownMetricError;
    /* ... */
}
```

```rust
// ✅ Good — the parser, then the failure it can produce.
impl std::str::FromStr for DistanceMetric {
    type Err = UnknownMetricError;
    /* ... */
}

/// Error returned when parsing a [`DistanceMetric`] from an unknown name.
#[derive(Debug)]
pub enum UnknownMetricError { /* ... */ }
```

A function that returns `DataFusionError` through `plan_err!` or `exec_err!` declares no error type, so this
rule has nothing to place.

## 3. Helpers Follow the Code That Calls Them

Helper types and functions come after the members they support, in dependency order: if `A` calls `B`, `A`
comes first. A private item above the public API it serves is the same defect as a kernel above the UDF that
calls it.

Common violations:

- A UDF struct or its `ScalarUDFImpl` below the kernels it calls
- An error type separated from the function that returns it, or collected at the end of the file
- A helper struct or function above the main type it supports
- Private implementation details scattered before the public API

## Checklist

Before committing code, verify:

- [ ] The main public type and its constructor appear early in the file
- [ ] Public structs and types appear before private helpers
- [ ] Each error type sits immediately after the function or `impl` that returns it
- [ ] Helper functions appear after the code that uses them
- [ ] No private implementation detail sits before the public API

## References

- [rust-mods](rust-mods.md) - Extends: The module invariants these rules make operational
- [rust-imports](rust-imports.md) - Related: The module prologue that precedes these members
