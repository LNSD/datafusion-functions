---
name: "principle-information-hiding"
description: "Information Hiding — reveal as little as possible; every item takes the most restrictive visibility that works. Load when choosing pub or pub(crate), or reviewing a package's public surface"
type: "principle"
scope: "global"
---

# Information Hiding (Reveal As Little As Possible)

## Rule

A module is defined by the design decision it hides. Its surface reveals as little as possible about how it
works, so the decision can change without any caller noticing.

Every item carries the most restrictive visibility that lets it do its job. Private is the default. Each
widening needs a reason:

| Widen to     | When                                    |
|--------------|-----------------------------------------|
| private      | Always, unless something below applies  |
| `pub(crate)` | A sibling module in this crate needs it |
| `pub`        | A consumer outside the crate needs it   |

There is no fourth case. `pub(super)` and `pub(in path)` signal that the module tree is wrong: an item
exactly one parent may see belongs to that parent.

Widening is one-way. Retracting a `pub` is a breaking change, so guess small. Rust's privacy boundary is the
module. Hide by what a module declares and re-exports; an accessor on every field hides nothing.

## Examples

1. **Gate an internal module at its declaration**
   A `pub` written "in case someone needs it" is a promise nobody can withdraw. State an internal boundary
   once, where the module is declared.

```rust
// ❌ Bad — the gate is repeated on every item, and the next item added defaults to `pub` and escapes.
pub mod kernels;

pub(super) fn l2_distance(left: &[f32], right: &[f32]) -> f32 {}
pub struct RowPairs<'a> { /* ... */ } // exported forever by accident
```

```rust
// ✅ Good — one gate at the declaration; items inside are plain `pub` and nothing leaks past it.
pub(crate) mod kernels;

pub fn l2_distance(left: &[f32], right: &[f32]) -> f32 {}
pub struct RowPairs<'a> { /* ... */ }
```

2. **Hide the decision, not just the data**
   A UDF whose surface mirrors its representation has hidden nothing, however private its fields are.

```rust
// ❌ Bad — the accessors re-expose the metric table; switching to an enum breaks every signature.
pub struct VectorDistance { signature: Signature, metrics: HashMap<String, fn(&[f32], &[f32]) -> f32> }

impl VectorDistance {
    pub fn metrics(&self) -> &HashMap<String, fn(&[f32], &[f32]) -> f32> {}
    pub fn metrics_mut(&mut self) -> &mut HashMap<String, fn(&[f32], &[f32]) -> f32> {}
}
```

```rust
// ✅ Good — the surface is what callers need; the table, its keys, and the kernels can all change.
pub fn vector_distance_udf() -> Arc<ScalarUDF> {}
```

## Why It Matters

A system is decomposed by what is likely to change, and each module hides one such decision. A module that
reveals its representation has published a decision. Every consumer of it becomes a reason not to revise it.

For a function library the surface that matters is small: the constructors that hand a `ScalarUDF` or
`AggregateUDF` to a `SessionContext`, and the SQL names and signatures they register. Kernels, coercion
helpers, and `ScalarUDFImpl` structs are decisions the package should stay free to revise.

The cost is asymmetric. Keeping an item private costs one `pub(crate)` the day a sibling needs it. Making it
public costs a major version bump the day it is withdrawn, plus an edit in every downstream crate that
reached for it.

## Pragmatism Caveat

`pub(crate)` is a normal visibility and needs no defence. Internal helpers, shared constants, and
cross-module types live there. `pub(super)` and `pub(in path)` are the smell. Where one seems necessary, ask
whether the item belongs in the module it is being shown to.

Test-only access is not a reason to widen. A private item is testable from a `#[cfg(test)]` module in the
same file. Widening it for an integration test exports an implementation detail to every consumer.

A plain data type with no invariant may have public fields: an options struct for a function, a row of test
inputs. Accessors there add nothing. Where an invariant exists, such as a vector dimension that must be
positive, the field stays private and the constructor enforces it.

Deliberate over-exposure is documented at the declaration: a kernel exposed for a downstream crate's hot
path, an internal type a macro must name. An undocumented `pub` on something the package does not intend to
support is indistinguishable from an oversight.

## Checklist

Before committing code, verify:

- [ ] Every item is private unless a specific caller requires otherwise
- [ ] `pub(crate)` is used for cross-module needs inside the crate; `pub` only for items a consumer outside
      the crate calls
- [ ] No `pub(super)` or `pub(in path)` was introduced; where one seemed necessary, the module tree was
      reconsidered instead
- [ ] An internal module is gated at its `mod` declaration, never by annotating each of its items
- [ ] A type's public surface is what callers need, never a mirror of its representation
- [ ] No item was made public solely so a test could reach it
- [ ] Public fields appear only on types with no invariant to protect
- [ ] Every deliberate over-exposure is documented at the declaration

## References

- [principle-single-responsibility](principle-single-responsibility.md) - Related: A module owns one
  responsibility only while callers cannot reach past its surface
- [principle-open-closed](principle-open-closed.md) - Related: Internals can be restructured freely only while
  nothing depends on them
- [principle-law-of-demeter](principle-law-of-demeter.md) - Related: A surface that reveals its internals
  makes reach-through possible
- [principle-type-driven-design](principle-type-driven-design.md) - Related: A private field lets a
  constructor be the only way to build a valid value

## External References

- [On the Criteria To Be Used in Decomposing Systems into Modules — D. L. Parnas](https://dl.acm.org/doi/10.1145/361598.361623)
- [Effective Java, Item 15 — Minimize the accessibility of classes and members](https://github.com/clxering/Effective-Java-3rd-edition-Chinese-English-bilingual/blob/dev/Chapter-4/Chapter-4-Item-15-Minimize-the-accessibility-of-classes-and-members.md)
- [Effective Rust, Item 22 — Minimize visibility](https://effective-rust.com/visibility.html)
- [Information Hiding and Encapsulation — David Gries](https://www.cs.cornell.edu/courses/JavaAndDS/files/infoHiding.pdf)
- [Least Privilege Principle — OWASP](https://owasp.org/www-community/controls/Least_Privilege_Principle)
