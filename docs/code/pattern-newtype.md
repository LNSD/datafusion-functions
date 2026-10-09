---
name: "pattern-newtype"
description: "Newtypes for identity, invariants, and units: private field, FromStr or TryFrom as the only validator. Load when a domain value such as a vector dimension or metric is a bare primitive, or two same-typed parameters sit side by side"
type: "core"
scope: "global"
---

# Newtype (Wrapped Primitive)

## Rule

A domain value with an identity, an invariant, or a unit is a newtype: `struct Dimension(NonZeroUsize)`. The
compiler then refuses to interchange it with its neighbours.

A value earns a newtype when it does one of three jobs:

1. **Identity.** Same-typed values that must never be swapped: a query vector and a candidate vector in one
   call, a dimension and a row count.
2. **Invariant.** A constraint checked once at the edge: a non-zero dimension, a lowercase function name.
3. **Unit.** A convention a primitive cannot state: distance versus similarity, degrees versus radians, a
   flat value offset versus a row index.

Three signals show a missing newtype in a diff. Two parameters of the same primitive type sit side by side.
A `1.0 -` or `* dimension` is explained in a comment. A doc comment carries a constraint ("must be
non-zero", "similarity, not distance", "row index").

Declare it as a tuple struct with a private field. The validating constructor is `FromStr` or `TryFrom` and
nothing else; a constructor that skips the check is rare, named for it, and says why in a comment. Add
`Display`, `AsRef`, and `#[serde(try_from = "...")]` where the value is deserialized, so deserialization runs
the same check. Construct it at the boundary the value enters through and nowhere else.

## Examples

1. **Two counts that are not the same count**

```rust
// ❌ Bad — `vector_at(values, row, dimension)` compiles with the arguments swapped and reads garbage.
fn vector_at(values: &[f32], dimension: usize, row: usize) -> &[f32] {}
```

```rust
// ✅ Good — the transposition is a compile error at every call site.
pub struct Dimension(NonZeroUsize);
pub struct RowIndex(usize);

fn vector_at(values: &[f32], dimension: Dimension, row: RowIndex) -> &[f32] {}
```

2. **A convention written into the type**

```rust
// ❌ Bad — the convention is a doc comment; the caller that forgot `1.0 -` ranked results backwards.
/// Returns cosine similarity, not distance.
fn cosine(a: &[f32], b: &[f32]) -> f32 {}
```

```rust
// ✅ Good — the convention is the type, and the one conversion lives in one method.
pub struct CosineSimilarity(f32);

impl CosineSimilarity {
    pub fn to_distance(&self) -> CosineDistance { CosineDistance(1.0 - self.0) }
}
```

3. **Brand the subset a function requires**

```rust
// ✅ Good — only a `FixedSizeList` of `Float32` is a vector column; the narrowing happens once.
pub struct VectorType { dimension: Dimension }

impl TryFrom<&DataType> for VectorType {
    type Error = DataFusionError;

    fn try_from(data_type: &DataType) -> Result<Self> {
        match data_type {
            DataType::FixedSizeList(field, size) if field.data_type() == &DataType::Float32 => {
                Ok(Self { dimension: Dimension::try_from(*size)? })
            }
            other => plan_err!("expected a vector of Float32, got {other}"),
        }
    }
}
```

## Why It Matters

The compiler remembers the fact at every call site. A reader forgets it once.

The failures a newtype prevents are quiet. A transposed dimension and row index reads a well-formed slice
from the wrong offset. A forgotten `1.0 -` returns the farthest neighbours first. Neither panics or fails a
shallow test. Both are compile errors once the types differ.

An invariant checked at the edge stays checked. A `Dimension` cannot exist without being non-zero, so no
kernel divides by a zero dimension. That is [principle-validate-at-edge](principle-validate-at-edge.md) in
the type system.

The cost is zero at runtime. A tuple struct has the primitive's layout, and the check runs once, at the
boundary. The documentation is the signature, so it cannot go stale.

## Pragmatism Caveat

A newtype with nothing to distinguish is ceremony.

- A value with no sibling and no invariant stays a primitive: an error message, a function alias.
- Two values of the same kind in different roles want a struct with named fields. Named fields remove the
  swap hazard that positional parameters create.
- A newtype cannot validate per-row data. "Is this vector's norm zero?" depends on the values and stays a
  runtime check in the kernel.
- The check runs on every construction, so construct at a boundary and never per row.
- A newtype that forces unchecked constructors at ordinary call sites has its boundary in the wrong place.
  Move construction to the edge the value enters through.

## Checklist

Before committing code, verify:

- [ ] Every domain value with an invariant, a unit or convention, or a confusable same-typed sibling is a
      newtype
- [ ] The wrapped field is private, so construction cannot bypass the invariant
- [ ] The validating constructor is `FromStr` or `TryFrom`, and no second validator exists
- [ ] Deserialization runs the same check through `#[serde(try_from = "...")]`
- [ ] The newtype is constructed at the boundary the value enters through, and nowhere else
- [ ] A conversion between two newtypes (similarity to distance) exists in exactly one function
- [ ] `Display`, `AsRef`, and the primitive's other conveniences are provided, so callers never reach for
      the inner value
- [ ] No newtype was added to a value with nothing to confuse it with and no invariant to carry

## References

- [principle-type-driven-design](principle-type-driven-design.md) - Foundation: A newtype makes an invalid value unrepresentable
- [principle-validate-at-edge](principle-validate-at-edge.md) - Foundation: Constructed at the edge, never re-checked downstream
- [principle-least-surprise](principle-least-surprise.md) - Foundation: A signature saying `usize` twice invites a transposition
- [pattern-builder](pattern-builder.md) - Related: Construction with multiple required fields

## External References

- [Rust API Guidelines — Newtypes](https://rust-lang.github.io/api-guidelines/type-safety.html#newtypes-provide-static-distinctions-c-newtype)
- [Parse, Don't Validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
- [The Ultimate Guide to Rust Newtypes](https://www.howtocodeit.com/guides/ultimate-guide-rust-newtypes)
