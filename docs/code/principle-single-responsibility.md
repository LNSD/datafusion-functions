---
name: "principle-single-responsibility"
description: "Single Responsibility — one struct, one reason to change; split when it spans several framework surfaces or mixes plumbing with pure computation. Load when designing UDF structs, splitting modules, or reviewing types that do both"
type: "principle"
scope: "global"
---

# Single Responsibility Principle (SRP)

## Rule

A struct or module owns one responsibility. Split it when one of these signals is present:

1. **Multiple outside surfaces.** The methods touch several outside worlds: the DataFusion planner contract
   (`Signature`, coercion, return types), Arrow array traversal, session configuration, the filesystem. Each
   boundary is its own concern. Three is a strong signal. Two warrants a split when they change
   independently.
2. **Disjoint field access.** The methods partition into groups that touch non-overlapping sets of fields.
   The groups are separate types sharing a struct by coincidence.
3. **Mixed plumbing and computation.** One unit both performs effects or framework plumbing (unpacking
   `ColumnarValue`, downcasting arrays, building result arrays, reading files) and does pure computation
   (a distance, a norm, a normalization). The pure part is extracted. It is the part worth unit testing,
   and the plumbing makes that tedious.

When a signal fires, the unit is split into focused units and the caller composes them.

## Examples

1. **One concern per module in a package**
   A vector function is four modules with one purpose each: argument checking, the numeric kernel, the
   `ScalarUDFImpl`, and registration with a `FunctionRegistry`.

```rust
// ❌ Bad — one struct owns the signature, the coercion rules, the math, and registration; the methods
// that read `metric` never read `signature`.
pub struct VectorFunctions {
    signature: Signature,
    metric: Metric,
}

impl VectorFunctions {
    pub fn coerce(&self, args: &[DataType]) -> Result<Vec<DataType>> {}
    pub fn distance(&self, a: &[f32], b: &[f32]) -> f32 {}
    pub fn register_all(&self, ctx: &SessionContext) -> Result<()> {}
}
```

```rust
// ✅ Good — four units, each describable in one sentence, composed by the caller.
pub fn check_vector_args(args: &[DataType]) -> Result<Dimension>;     // argument checking
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32;                  // kernel
pub struct CosineDistance { signature: Signature }                    // ScalarUDFImpl
pub fn register(registry: &mut dyn FunctionRegistry) -> Result<()>;   // registration
```

2. **Separate the pure computation from the plumbing**
   `invoke_with_args` is a thin shell over a pure kernel. Only the kernel holds the invariant: a
   zero-length vector yields null, never `NaN`.

```rust
// ❌ Bad — the math is trapped inside array plumbing; the first version divided by a zero norm and
// returned NaN, and no unit test could reach it without building Arrow arrays.
fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
    let (left, right) = as_vector_pair(&args.args)?;
    let values = left.iter().zip(right.iter()).map(|(a, b)| {
        let (dot, norm_a, norm_b) = /* thirty lines of arithmetic */;
        Some(1.0 - dot / (norm_a.sqrt() * norm_b.sqrt()))
    });
    Ok(ColumnarValue::Array(Arc::new(Float32Array::from_iter(values))))
}
```

```rust
// ✅ Good — the plumbing calls a pure core tested with two slices.
/// Cosine distance, or `None` when either vector has zero norm.
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Option<f32> {}

fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
    let (left, right) = as_vector_pair(&args.args)?;
    let values = left.iter().zip(right.iter()).map(|(a, b)| cosine_distance(a, b));
    Ok(ColumnarValue::Array(Arc::new(Float32Array::from_iter(values))))
}
```

## Why It Matters

A type with one responsibility has one reason to change. A coercion rule widening to accept `ListArray`
cannot break the distance kernel, because the kernel contains none. A new metric cannot break registration.

Logic fused to plumbing is exercised only through SQL queries against a `SessionContext`. Those need record
batches, a planner, and a full execution, so the awkward cases (zero norms, nulls, mismatched dimensions) go
untested. Pure logic is tested with a slice and an assertion, in milliseconds, at the point of the change.

## Pragmatism Caveat

A small struct that touches two surfaces is not automatically wrong. An aggregate `Accumulator` owns both its
running state and the `merge_batch` logic, because the type exists for the coupling between them: partial
states must merge to the same result as one pass. Splitting them would leave the invariant in neither half. A
`ScalarUDFImpl` owns its `Signature` because the signature's lifetime is the function's lifetime.

When a signal fires and the concerns stay together, the module `//!` docs or a comment say why: a shared
invariant, a lifetime that cannot be split, a trait that demands both. An undocumented violation is always
wrong.

## Checklist

Before committing code, verify:

- [ ] Each struct or module is describable in one sentence without "and"
- [ ] No type both does array plumbing or I/O and holds a non-trivial pure algorithm; the algorithm is a free
      function
- [ ] Fields form one cohesive group, never two groups touched by disjoint method sets
- [ ] A change to one concern (coercion rules, kernel math, registration) touches one module
- [ ] Deliberate co-location of concerns is explained in the module docs

## References

- [principle-law-of-demeter](principle-law-of-demeter.md) - Related: An overloaded type is the one callers
  navigate through
- [principle-open-closed](principle-open-closed.md) - Related: An extension point requires variants that each
  own one concern
- [principle-inversion-of-control](principle-inversion-of-control.md) - Related: Separated pure logic is what
  makes injecting collaborators worthwhile
- [principle-dry-wet](principle-dry-wet.md) - Related: An abstraction serving two responsibilities is the
  wrong abstraction
- [principle-rate-of-change](principle-rate-of-change.md) - Related: Two rates of change are two reasons to
  change; the same split reached from the other side

## External References

- [SOLID: The Single Responsibility Principle (Uncle Bob)](https://blog.cleancoder.com/uncle-bob/2014/05/08/SingleReponsibilityPrinciple.html)
- [Single Responsibility Principle with a Rust Example](https://medium.com/@dogabudak/single-responsibility-principle-with-a-rust-example-2940504e3ebd)
