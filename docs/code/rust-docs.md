---
name: "rust-docs"
description: "The two doc channels — rustdoc for callers, // for editors — audience routing, intent over implementation, shared voice, sentence budgets, one home per fact. Load when writing any doc comment or comment"
type: "core"
scope: "global"
---

# Rust Documentation Style

**Document intent, never implementation.** Say what the code is for and what a reader may rely on. Do not
say how the body works. Every other rule here follows from that one.

Source has two documentation channels with two readers. Rustdoc (`///`, `//!`) is read by the caller.
A `//` comment is read by the editor. This document owns the choice between them and the voice both share.
Rustdoc's levels and sections are owned by [rust-docs-rustdoc](rust-docs-rustdoc.md), comment placement by
[rust-docs-comments](rust-docs-comments.md), and TODOs by [rust-docs-todo](rust-docs-todo.md).

## 1. Two Channels, Two Audiences

Rustdoc is the hover text at every call site and the rendered API page. Its reader is deciding whether and how
to call the item. A `//` comment is read by the person with the file open, about to change the line under it.
A `// SAFETY:` note and a lint `reason` are comments held to a higher bar.

Pick the channel by reader, never by length. A one-line fact can be rustdoc. A five-line argument can be a
comment.

## 2. Route by Audience

Ask who needs the fact. The table says where it goes.

| The information                                     | Its home                                          |
|-----------------------------------------------------|---------------------------------------------------|
| What the package is for, and why depend on it       | Crate root `//!`                                  |
| Why the module exists, what it defends against      | Module `//!`                                      |
| What an item does, and the contract a caller gets   | Item `///`                                        |
| The default of an optional config field             | Field `///`, with the literal value               |
| The conditions under which a function panics        | `# Panics` section                                |
| What a caller must uphold at an unchecked call      | `# Safety` section, and `// SAFETY:` at the call  |
| Why this line is shaped this way, what breaks else  | Comment at the line                               |
| What the code used to be, and what that cost        | Leading comment, or only the commit message       |
| What was verified against an upstream source        | Leading comment on the code it defends            |
| Why a lint is suppressed                            | `reason = "..."` on the `#[expect]`               |
| Work that is deferred, and who owes it              | `TODO` comment                                    |
| How a design was decided                            | `docs/feat/`                                      |
| Why this change was made                            | The commit message                                |

Both channels can sit on one declaration. Rustdoc first, then the comment, then the code.

```rust
// ✅ Good — the contract in rustdoc; the shipped mistake in the comment, for the next editor.
/// The distance column type: `Float64` for `Float64` vectors, `Float32` for every other element type.
// This used to return `Float64` for every input; `Float32` queries paid a widening copy per batch.
fn return_type(&self, arg_types: &[DataType]) -> Result<DataType> {}
```

Rustdoc is written in the timeless present. Only a comment recounts the past. Rustdoc never states
implementation status ("phase 2 pending"); that is a schedule, not an API.

## 3. A Rule Document Is Never Cited as Authority

No sentence in source points at `docs/code/` to justify the code under it. The path tells a caller nothing
and breaks on the next reorganisation. Name the mechanism instead.

```rust
// ❌ Bad — a path in place of a reason, rendered on every docs page.
/// A vector dimension known to be non-zero. See docs/code/pattern-newtype.md.
pub struct Dimension(NonZeroUsize);

// ✅ Good — the mechanism, in one sentence.
/// A vector dimension validated by `TryFrom`; no other constructor exists.
pub struct Dimension(NonZeroUsize);
```

Two exceptions. A module whose domain is documentation tooling names those paths as data. Feature documents
under `docs/feat/` describe behavior, so rustdoc may cite them.

## 4. Intent Over Implementation

When a sentence could say what the code does or what it is for, say what it is for. A sentence about the body
is rewritten with the body or left behind as a lie.

Name a mechanism only when a decision the reader must make turns on it. In rustdoc, a named mechanism is a
promise the package must keep. In a comment, it is a claim the next commit can break. The test: would this
sentence stop a reasonable person from "simplifying" the code back into the bug?

```rust
// ❌ Bad — the mechanism is the whole sentence, and the caller's one need (what a null yields) is missing.
/// Downcast `rows` to `Float32Array` values, walk each row, and call `cosine` against the broadcast query.
pub fn cosine_distances(rows: &FixedSizeListArray, query: &[f32]) -> Float32Array {}

// ✅ Good — the guarantee, then the one behavior the caller plans around.
/// Cosine distance from each row to `query`. A null row or a zero-norm row yields a null distance.
pub fn cosine_distances(rows: &FixedSizeListArray, query: &[f32]) -> Float32Array {}
```

History follows the same rule. A comment that records the past records the logical change, what the code now
guarantees that it did not before. The diff already records the code change.

```rust
// ❌ Bad — a changelog entry. It records the edit, not the reason, so it stops nobody repeating it.
// Changed vector_at to take a Dimension instead of a usize, and moved the null check above the loop.
```

## 5. The Shared Voice

Both channels are prose: complete sentences, aimed at one reader, making a claim that reader could doubt.

- **The smallest true statement.** One line when one line is the whole truth. Nothing when the code already
  says it.
- **Never narrate.** A sentence that paraphrases the next line is wrong after the next edit.
- **Spell values as the code spells them.** `Default 1_024`, not `1024`, so a search finds both.
- **Backtick every identifier and value.** Rustdoc renders markdown; a bare identifier reads as a word.
- **Link with intra-doc links.** `[`Dimension`]` survives a rename; plain text does not.

```rust
// ✅ Good — why, and what breaks otherwise.
// A sliced `ListArray` keeps its parent's values, so its first offset need not be zero; assuming zero reads
// the vectors of rows before the slice.
let start = offsets[0];
```

## 6. Every Doc Comment Has a Budget

An item `///` is one sentence of contract, at most one more for a behavior the signature does not show, then
the mandatory sections. A module `//!` is at most three sentences. A `//` comment is at most two. A comment over
budget is not tightened; it is moved. A contract that needs more goes on the type or in a feature doc, and the
item links to it. History longer than two sentences goes in the commit message.

The budget counts sentences, not lines, so a long sentence is no escape. One claim per sentence, in the order
a reader needs them.

```rust
// ❌ Bad — four sentences to say one thing, and the second half argues with a reader who is not there.
/// The metric parsed from the literal option. The parse admits the canonical names only, which is
/// validation rather than a formatting choice: `cosine`, `l2`, and `inner_product` name one kernel
/// each, while aliases such as `euclidean` would be ambiguous and are refused at planning.
pub enum DistanceMetric { Cosine, L2, InnerProduct }

// ✅ Good — the contract, then the one rule a caller must know.
/// Distance metric named by a UDF's literal option. Accepted names are `cosine`, `l2`, and
/// `inner_product`; any other name is a plan error.
pub enum DistanceMetric { Cosine, L2, InnerProduct }
```

## 7. State Each Fact Once

An invariant has one home: the item that upholds it. Everywhere else links to that item with an intra-doc link
or names the feature doc, and adds nothing. A fact restated at the module, the type, the field, the function,
and the test file is five copies that drift, and five comments a reader must check against each other before
trusting any of them.

```rust
// ❌ Bad — the same rule at three levels; each is a copy that will drift.
//! Distance UDFs. Both arguments must share one dimension, which planning checks.

/// The dimension of both arguments. Both must share one dimension, which planning checks.
pub dimension: Dimension,

/// Distance kernel. Assumes both arguments share one dimension, which planning checks.
fn distance_kernel(pair: &VectorPair, left: &FixedSizeListArray, right: &FixedSizeListArray) {}

// ✅ Good — one home, and the others point at it.
//! Distance UDFs over two vector arguments of one dimension ([`VectorPair`]).

/// The dimension both arguments share; see [`VectorPair`].
pub dimension: Dimension,

/// Distance from each row of `left` to the same row of `right`, under the planned [`VectorPair`].
fn distance_kernel(pair: &VectorPair, left: &FixedSizeListArray, right: &FixedSizeListArray) {}
```

## 8. Say the Fact, Not the Contrast

State what is true. Do not frame it against what a reader might have thought. "X rather than Y", "not X but
Y", "which is what keeps Z", and "exactly what W admits" each spend a clause naming a wrong reading instead of
the right one. Delete the contrast and keep the claim. If the wrong reading is a real trap, that is a `//`
comment naming what breaks, not a turn of phrase in rustdoc.

```rust
// ❌ Bad — three contrasts, no added fact.
/// The metric is a name, not a kernel: it selects an implementation, it does not compute anything.
/// Dispatch happens per batch in `invoke_with_args`, which is the only place a kernel runs rather
/// than somewhere at planning.
pub fn metric(&self) -> DistanceMetric {}

// ✅ Good — the same facts, flat.
/// The metric this UDF was planned with. [`DistanceMetric::kernel`] returns the function that computes it.
pub fn metric(&self) -> DistanceMetric {}
```

## Checklist

Before committing code, verify:

- [ ] Every doc comment and comment is aimed at one reader: the caller in rustdoc, the editor in `//`
- [ ] Contracts and domain sit in rustdoc; process, history, and justification sit in comments
- [ ] Rustdoc is in the timeless present, recounts no past, and states no implementation status
- [ ] No sentence cites `docs/code/` as authority for the code beneath it
- [ ] Every doc comment and comment states intent and names a mechanism only where a reader's decision turns
      on it
- [ ] History records the logical change, never the code change
- [ ] Nothing narrates the next line or restates a declaration
- [ ] Values and identifiers are spelled as the code spells them, in backticks, and types use intra-doc links
- [ ] Every `///` is one sentence of contract plus at most one more, every `//!` at most three sentences, and
      every `//` at most two; anything longer was moved to a type, a feature doc, or the commit message
- [ ] Every invariant is stated once, on the item that upholds it, and linked from everywhere else
- [ ] No sentence is shaped as a contrast ("X rather than Y", "not X but Y"); each states the fact

## References

- [rust-mods-members](rust-mods-members.md) - Related: Where doc comments and comments sit in a file's
  member order
- [principle-least-surprise](principle-least-surprise.md) - Foundation: Documentation exists to stop the next
  reader from "fixing" something on purpose
