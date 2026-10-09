---
name: "rust-docs-comments"
description: "Editor-facing // comments: placement, justifying discarded errors and lossy casts, history, provenance, no section separators. Load when writing a // comment or discarding an error"
type: "core"
scope: "global"
---

# Rust Comments

A `//` comment is the editor's channel. It addresses the person about to change the code under it: it
justifies a decision they would otherwise undo, or recounts a past they are about to repeat. It is never
rendered to a caller, so it carries what rustdoc must not: history, provenance, and the argument for an
exception. Channel choice, voice, and length are owned by [rust-docs](rust-docs.md). This document owns where
a comment sits and which decisions must carry one.

## 1. A Comment Accompanies Code

Every comment is attached to a specific piece of code and makes a claim about it. Two placements exist:

- **At the line**: immediately above the statement it justifies.
- **Leading**: a comment above a declaration, after the doc comment when the item has one. Rustdoc first,
  then the comment, then the code. At module level it sits below the `//!` comment, before the imports.

A comment that accompanies nothing, such as a label for the next region of the file, is not a comment
([§5](#5-no-section-separators)). A comment that restates the declaration beside it (`// The vector dimension.`
above `dimension: Dimension`) is noise.

## 2. Comments Justify Decisions

A comment explains a decision a reader would otherwise undo. Two sentences: the decision, and what breaks
without it. It never narrates the line below it ([rust-docs](rust-docs.md#5-the-shared-voice)); the budget is
owned by [rust-docs](rust-docs.md#6-every-doc-comment-has-a-budget).

```rust
// ✅ Good — why an index that looks off by one is right, which is what the next reader will "fix".
// A `ListArray` has one more offset than rows, and row `i` ends at `offsets[i + 1]`. Dropping the `+ 1` reads
// every vector one row early.
let end = offsets[row + 1];
```

Two decisions never appear bare.

**A discarded error.** Every `let _ =`, `.ok()`, `.unwrap_or_default()` on a `Result`, and every `Err(_)` arm
that does not propagate carries a comment saying what would break if the error escaped.

```rust
// ❌ Bad — a silent discard. The next reader cannot tell a decision from a bug.
let folded = match fold_literal_metric(&args) {
    Ok(metric) => metric,
    Err(_) => return Ok(ExprSimplifyResult::Original(args)),
};

// ✅ Good — what the discard protects.
// Folding is an optimization; on failure the original call runs and reports the error with its row.
let folded = match fold_literal_metric(&args) {
    Ok(metric) => metric,
    Err(_) => return Ok(ExprSimplifyResult::Original(args)),
};
```

**A lossy or unchecked conversion.** Every `as` cast that can truncate, wrap, or change sign carries the bound
that makes it safe here. Prefer `try_into()` and handle the failure.

```rust
// ❌ Bad — a silent wrap waiting for a dimension past `i32::MAX`, which builds a negative list size.
let list_size = dimension.get() as i32;

// ✅ Good — the bound, so a reader checks the claim and not the arithmetic.
// `Dimension` caps its value at `MAX_DIMENSION` (`65_535`), so this cannot wrap.
let list_size = dimension.get() as i32;
```

The `// SAFETY:` comment above an unchecked constructor is owned by [rust-fn-unchecked](rust-fn-unchecked.md).
The `reason` on a lint suppression is owned by [rust-attrs-lints](rust-attrs-lints.md). Both name the fact
that makes the exception correct, not the fact that an exception was made.

## 3. History Lives Here, or in the Commit

What the code used to be is process, so it goes in this channel and never in rustdoc
([rust-docs](rust-docs.md#2-route-by-audience)). Write it as a leading comment only when the story is what
stops the next editor from reintroducing the defect. Otherwise the commit message holds it. Three sentences at
most: what it was, what broke, what changed. One clause of history fits inline as a parenthetical.

```rust
// ✅ Good — the argument against "simplifying" the start back to a count of non-null rows.
/// The values of the vector at `row`. A null row still occupies `dimension` slots.
// The start used to skip null rows; every vector after the first null was read from the wrong slots.
pub fn vector_at(values: &[f32], row: usize, dimension: Dimension) -> &[f32] {}
```

## 4. Provenance Defends a Design

A claim checked against an upstream source is recorded as a leading comment on the code the check defends.
It stops an editor from re-litigating the design.

```rust
// ✅ Good — the doc states the fact; the comment says it was checked, on the code whose shape depends on it.
/// Negated inner product of `left` and `right`, so that a smaller value is closer.
// Verified against pgvector's `<#>` operator, which also negates; `ORDER BY` queries ported from it depend on
// the sign.
pub fn inner_product_distance(left: &[f32], right: &[f32]) -> f32 {}
```

## 5. No Section Separators

Banner comments that partition a file into titled regions are banned. A file that wants chapters has more than
one reason to change ([principle-single-responsibility](principle-single-responsibility.md)). Split it along
the banners instead. A bare one-line label (`// helpers`) is the same smell.

```rust
// ❌ Bad — chapters. A change to the arithmetic edits the file that also owns planning.
// --------------------------------- Signature -------------------------------
pub struct DistanceSignature { /* ... */ }
// --------------------------------- Kernels ---------------------------------
fn cosine_kernel(left: &[f32], right: &[f32]) -> f32 { /* ... */ }

// ✅ Good — the banner became a module, and its `//!` says what the banner tried to.
//! Distance kernels: per-row arithmetic over vector values, with no planning logic.
fn cosine_kernel(left: &[f32], right: &[f32]) -> f32 { /* ... */ }
```

## Checklist

Before committing code, verify:

- [ ] Every comment accompanies specific code and makes a claim about it; none is a bare label
- [ ] Leading comments sit after the doc comment and before the item they annotate
- [ ] Every discarded error (`let _ =`, `.ok()`, a non-propagating `Err(_)` arm) carries a comment naming what
      would break if the error escaped
- [ ] Every lossy `as` cast carries the bound that makes it safe, or is replaced with `try_into()`
- [ ] History appears only where it stops a defect from returning, in at most three sentences; the rest stays
      in commit messages
- [ ] Upstream-verification notes sit on the code they defend, not in rustdoc
- [ ] No comment section separators; a file that wants them is split instead

## References

- [rust-docs](rust-docs.md) - Extends: The intent rule, the audience routing, the shared voice, and the
  sentence budgets, applied to the editor's channel
- [rust-docs-todo](rust-docs-todo.md) - Related: The one comment that is about work rather than about code
- [rust-fn-unchecked](rust-fn-unchecked.md) - Related: Owns the `// SAFETY:` comment this document's voice
  applies to
- [rust-attrs-lints](rust-attrs-lints.md) - Related: Owns the `reason` on a lint suppression
- [principle-single-responsibility](principle-single-responsibility.md) - Foundation: A section separator is an
  SRP violation drawn in ASCII
