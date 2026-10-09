---
name: "rust-docs-todo"
description: "TODO comments: one keyword, optional GitHub handle owner, one-space continuation, placed between rustdoc and the item. Load when leaving or reviewing deferred work"
type: "core"
scope: "global"
---

# TODO Comments

A TODO is the one comment about work that is not there yet. That is project process, so it is a `//` comment,
and it is written for a stranger who was not in the conversation that produced it. The editor's channel is
owned by [rust-docs-comments](rust-docs-comments.md). This document owns the TODO's shape and placement.

## 1. A TODO States the Work and Why It Waits

```
// TODO(handle): what must be done, in the imperative, and enough of why to act on it.
```

The keyword is uppercase `TODO`. The parenthetical is optional, the colon is not. The text is a complete
sentence stating the work and the reason it is not done; the reason is what tells a stranger whether the
moment has come. A thought not worth describing precisely enough for someone else to do is deleted or filed
as an issue, not left as a comment.

```rust
// ❌ Bad — a mood, then a task with no reason. Neither can be acted on or closed.
// TODO: this is ugly
// TODO: split this module

// ✅ Good — task, evidence, and cost in one sentence a stranger can act on.
// TODO(dana): split this module into signature and kernels; every kernel change edits the planning code too.
```

## 2. The Parenthetical Is a GitHub Handle

The parenthetical holds one thing: the GitHub handle of the person who owns the TODO and will address it.
Written bare, without `@`. Only a user handle: not a package, module, ticket, category, or team, because none
of those picks up work and each reads like ownership without conferring any. Unowned work is a bare `TODO:`.

| Form           | Means                                        | Use when                                       |
|----------------|----------------------------------------------|------------------------------------------------|
| `TODO(handle)` | This person owns the debt and will settle it | Someone has accepted the work                  |
| `TODO:`        | Unowned                                      | The work is small, local, and obvious in place |

```rust
// ❌ Bad — an area, so it looks owned and is not; everyone assumes someone else is on it.
// TODO(vector-kernels): `Float16` vectors are unsupported.

// ✅ Good — one person's name, so a reviewer knows who to ask whether it is still deferred on purpose.
// TODO(dana): widen `Float16` vectors to `Float32` in `coerce_types` instead of rejecting them.

// ✅ Good — unowned, because the work is small and obvious in place.
// TODO: drop this alias once the last caller moves to `register_all`.
```

A handle is a claim on someone's time, so it is written only with their agreement.

## 3. Multiline TODOs Indent One Space

A TODO that needs more than one line continues on `//` lines indented one space past the first line's text,
on every continuation line. IDE TODO tooling follows the indent; a flush continuation starts a new, truncated
item.

```rust
// ❌ Bad — read as two items, the second a fragment.
// TODO(dana): `Float16` vectors are unsupported;
// one is rejected at planning instead of widened.

// ✅ Good — one item, hanging indent.
// TODO(dana): `Float16` vectors are unsupported;
//  one is rejected at planning, because the kernels read `Float32` values only.
```

## 4. TODO Is the Only Keyword

`FIXME`, `XXX`, and `HACK` are not used. A search for `TODO` is then the complete debt list. Severity goes in
the sentence, which states it more precisely than a keyword can.

```rust
// ❌ Bad — a second keyword; a search for TODO misses it.
// FIXME: f16 vectors get rejected

// ✅ Good — one keyword, and the sentence says what a caller loses.
// TODO(dana): `Float16` vectors are rejected, so a half-precision column cannot be queried without a cast.
```

## 5. A TODO Never Goes in Rustdoc

No TODO appears in a `///` or `//!` comment. Rustdoc is rendered on every doc page and hover, and that reader
cannot act on the backlog. The TODO sits after the rustdoc and immediately before the item it concerns.

```rust
// ❌ Bad — backlog on the doc page, in every hover, until someone acts.
/// Coerce the argument types to the vector types the kernels accept.
///
/// TODO(dana): `Float16` vectors are rejected.
fn coerce_types(&self, arg_types: &[DataType]) -> Result<Vec<DataType>> {}

// ✅ Good — the rustdoc describes the function; the TODO sits where only an editor sees it.
/// Coerce the argument types to the vector types the kernels accept.
// TODO(dana): `Float16` vectors are rejected, so a half-precision column needs an explicit cast.
fn coerce_types(&self, arg_types: &[DataType]) -> Result<Vec<DataType>> {}
```

If the unfinished work changes what a caller can rely on today, that is a fact about the API. Write the
current behavior in the rustdoc as a plain sentence, and leave the debt in the comment below it.

## 6. A TODO Is Not a Justification

A `// SAFETY:` comment, a lint `reason`, and the comment on a swallowed error each explain why the code is
correct as it stands. A TODO says the code is not finished. One never replaces the other; a TODO may sit
beside a justification, not in its place.

```rust
// ❌ Bad — the TODO stands in for the SAFETY comment, so the unchecked call ships unexamined.
// TODO: validate this properly
let dimension = Dimension::new_unchecked(list_size);

// ✅ Good — the justification says why this is correct now.
// SAFETY: `return_type` rejected a zero list size for this argument, so `list_size` is non-zero.
let dimension = Dimension::new_unchecked(list_size);
```

## Checklist

Before committing code, verify:

- [ ] Every TODO states the work in the imperative, with enough of the reason that a stranger can act on it
- [ ] The parenthetical, if present, is an individual's GitHub handle, written bare, and nothing else
- [ ] A TODO naming a handle was agreed with that person; otherwise it is written unowned as `TODO:`
- [ ] Every continuation line of a multiline TODO is indented one space past the first line's text
- [ ] The keyword is `TODO`; no `FIXME`, `XXX`, or `HACK` was introduced
- [ ] No TODO appears inside a `///` or `//!` comment; it sits between the rustdoc and the item it concerns
- [ ] No TODO stands in for a `// SAFETY:` comment, a lint `reason`, or any other required justification

## References

- [rust-docs-comments](rust-docs-comments.md) - Extends: The editor's channel a TODO lives in, and the
  justification rules it does not replace
- [rust-docs-rustdoc](rust-docs-rustdoc.md) - Related: The caller's channel a TODO never enters
- [rust-fn-unchecked](rust-fn-unchecked.md) - Related: Owns the `// SAFETY:` comment a TODO does not replace
- [rust-attrs-lints](rust-attrs-lints.md) - Related: Owns the `reason` on a suppression, which is a
  justification rather than a debt
