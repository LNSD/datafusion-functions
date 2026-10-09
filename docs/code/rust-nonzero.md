---
name: "rust-nonzero"
description: "NonZero integer types for values where zero is not representable: dimensions, limits, counts, sizes; compile-time-checked constants via const-evaluated expect. Load when declaring a numeric limit, count, or size, or reviewing one typed as a plain integer"
type: "core"
scope: "global"
---

# Non-Zero Integers (`std::num::NonZero`)

**A quantity for which zero is not a value is typed `NonZero*`.** A plain integer admits a state the domain
has no meaning for, and every consumer re-checks it or inherits a divide-by-zero, an empty chunk, or a
`chunks_exact(0)` panic. The `NonZero` type carries the proof once, at construction, and the compiler
enforces it everywhere the value flows.

Validation of externally supplied values is owned by
[principle-validate-at-edge](principle-validate-at-edge.md). `*_unchecked` constructors and their `// SAFETY:`
discipline are owned by [rust-fn-unchecked](rust-fn-unchecked.md). This document is about when a value is a
`NonZero` type and how its constants are written.

## 1. Type the Domain, Not the Wire

A vector dimension, a top-k count, a chunk length, a quantization bit width: any value whose zero has no
meaning is declared as the matching `NonZero*` type where it enters the type system, option structs
included. The conversion happens at the boundary: `Dimension::try_from` on a `FixedSizeList` size at plan
time, or a literal `k` argument rejected with `plan_err!` before any batch runs
([principle-validate-at-edge](principle-validate-at-edge.md)).

"Zero means unbounded" is spelled `Option<NonZero*>`: `None` is unbounded, `Some` is a limit. The niche makes
`Option<NonZeroUsize>` the same size as `usize`.

```rust
// ❌ Bad — zero is representable but meaningless; the one kernel that forgot to check panicked in `chunks_exact(0)` mid-query.
/// Rows scored per kernel call. `0` disables chunking.
pub chunk_rows: usize,
```

```rust
// ✅ Good — zero is unrepresentable, and "unbounded" is a state the type spells out.
/// Rows scored per kernel call.
pub chunk_rows: NonZeroUsize,
/// Largest vector dimension accepted. `None` means unbounded.
pub max_dimension: Option<NonZeroUsize>,
```

## 2. Constants Are Const-Evaluated, So a Bad Literal Fails the Build

A `NonZero*` constant is written `NonZero*::new(N).expect("...")`. A `const` item is evaluated at compile
time, so a zero literal is a compile error at the declaration and the `expect` message is the diagnostic. The
message states what the value is, so the build error reads as a sentence.

```rust
// ✅ Good — a typo'd `0` is a compile error, never a runtime panic in the first query to call the UDF.
const DEFAULT_CHUNK_ROWS: NonZeroUsize = NonZeroUsize::new(1024).expect("1024 is a non-zero chunk length");
```

An inline `const { ... }` expression gives the same guarantee in expression position. The usual site is a `Default` impl,
where the literal sits in a fn body and would otherwise be checked at runtime.

```rust
// ❌ Bad — evaluated at runtime; a typo'd `0` panics the first time a UDF is built with default options.
impl Default for TopKOptions {
    fn default() -> Self {
        Self { k: NonZeroUsize::new(10).expect("10 is a non-zero neighbour count") }
    }
}
```

```rust
// ✅ Good — the `const` expression forces compile-time evaluation, the same check a named constant gets.
impl Default for TopKOptions {
    fn default() -> Self {
        Self { k: const { NonZeroUsize::new(10).expect("10 is a non-zero neighbour count") } }
    }
}
```

A named `const` item is the default form. The inline form is for a literal used in exactly one place. A value
used twice is a named constant.

## 3. Const Evaluation Replaces `new_unchecked` for Literals

`NonZero*::new_unchecked` is never called on a literal. It is `unsafe`, it carries the call-site obligations
of [rust-fn-unchecked](rust-fn-unchecked.md), and it buys nothing: the const-evaluated `new(...).expect(...)`
form compiles to the same constant and checks the literal at build time.

```rust
// ❌ Bad — unsafe with nothing to buy; the next edit changes the literal and the SAFETY comment stops being true.
// SAFETY: 8 is not zero.
const SIMD_LANES: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(8) };
```

```rust
// ✅ Good — the same constant, checked by the compiler.
const SIMD_LANES: NonZeroUsize = NonZeroUsize::new(8).expect("8 is a non-zero SIMD lane count");
```

When `new_unchecked` on a runtime value is warranted, and what its call sites must carry, is owned by
[rust-fn-unchecked](rust-fn-unchecked.md).

## Checklist

Before committing code, verify:

- [ ] No option field, dimension, limit, count, or size for which zero is meaningless is typed as a plain
      integer
- [ ] No field uses a zero sentinel for "disabled" or "unbounded" where `Option<NonZero*>` states it in the
      type
- [ ] Every `NonZero*` constant is built with `new(...).expect(...)` in const position, with a message naming
      the value
- [ ] No inline `const { ... }` expression wraps a value used more than once; repeated values are named constants
- [ ] No `new_unchecked` call takes a literal argument

## References

- [principle-type-driven-design](principle-type-driven-design.md) - Foundation: The type carries the proof, so
  consumers stop re-checking it
- [principle-validate-at-edge](principle-validate-at-edge.md) - Foundation: Zero from a function argument or
  an Arrow type is rejected at the boundary that let it in
- [rust-fn-unchecked](rust-fn-unchecked.md) - Related: Owns the `# Safety` / `// SAFETY:` discipline that
  governs `new_unchecked` on runtime values
- [pattern-newtype](pattern-newtype.md) - Related: `Dimension` wraps a `NonZeroUsize` and adds the
  vector-specific identity

## External References

- [Rust standard library — `std::num::NonZero`](https://doc.rust-lang.org/std/num/struct.NonZero.html)
