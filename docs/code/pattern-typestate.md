---
name: "pattern-typestate"
description: "Typestate pattern — model state machines with distinct types to enforce valid transitions at compile time. Load when designing accumulators, multi-step kernels, or objects with lifecycle states"
type: core
scope: "global"
---

# Typestate Pattern (State Machines with Types)

## Rule

Give each state of a lifecycle its own type. A transition consumes `self` and returns the next state's type.
Only the operations valid in a state exist on that state's type, so an invalid transition is a compile error.

Apply it where a struct guards transitions with a status enum and runtime assertions. Those catch a bad
transition only when the wrong path runs.

## Examples

```rust
// ❌ Bad — the assertion fires at runtime, on the first caller that queries before fitting.
impl VectorIndex {
    pub fn search(&self, query: &[f32], k: usize) -> Vec<usize> {
        assert_eq!(self.status, IndexStatus::Built);
        self.nearest(query, k)
    }
}
```

```rust
// ✅ Good — `search` exists only on `BuiltIndex`, so searching an unbuilt index does not compile.
pub struct EmptyIndex { dimension: Dimension }
pub struct LoadingIndex { dimension: Dimension, vectors: Vec<f32> }
pub struct BuiltIndex { dimension: Dimension, vectors: Vec<f32>, norms: Vec<f32> }

impl EmptyIndex {
    pub fn load(self, vectors: &FixedSizeListArray) -> Result<LoadingIndex> {
        Ok(LoadingIndex { dimension: self.dimension, vectors: copy_values(vectors)? })
    }
}

impl LoadingIndex {
    pub fn build(self) -> BuiltIndex {
        let norms = compute_norms(&self.vectors, self.dimension);
        BuiltIndex { dimension: self.dimension, vectors: self.vectors, norms }
    }
}
```

```rust
// ✅ Good — one generic struct carries the shared fields; the state parameter carries the rest.
pub struct VectorIndex<S> { dimension: Dimension, state: S }

pub struct Empty;
pub struct Built { vectors: Vec<f32>, norms: Vec<f32> }

impl VectorIndex<Empty> {
    pub fn build(self, vectors: Vec<f32>) -> VectorIndex<Built> {
        let norms = compute_norms(&vectors, self.dimension);
        VectorIndex { dimension: self.dimension, state: Built { vectors, norms } }
    }
}
```

## Why It Matters

A runtime assertion fails when the wrong path executes, which can first happen in a user's query. Typestate
makes the invalid transition a compile error. The type signature also lists the operations each state
allows, so the enforcement is the documentation.

## Pragmatism Caveat

An object with two states, or with simple and well-tested transitions, is clearer as a status enum. Apply
typestate when an invalid transition would be a serious bug, when the state machine is large enough that an
assertion is easy to forget, or when several callers must agree on the transition order. DataFusion drives
some lifecycles through traits it owns: an `Accumulator` is called through `&mut self` for `update_batch`,
`merge_batch`, and `evaluate`, and is stored as `Box<dyn Accumulator>`. Such an object needs one concrete
type, so it keeps a status enum. Typestate fits in-memory, linear workflows the crate itself drives.

## Checklist

Before committing code, verify:

- [ ] Every transition consumes `self`, so the old state cannot be reused
- [ ] Each state type exposes only the operations valid in that state
- [ ] No `assert!` or `panic!` checks a state the types could enforce
- [ ] State-specific data lives only in the states that have it (`norms` in `Built`, not `Empty`)
- [ ] A two-state object, or one driven through a DataFusion trait object, keeps a status enum where
      typestate would add complexity

## References

- [principle-type-driven-design](principle-type-driven-design.md) - Foundation: The principle this pattern implements
- [pattern-builder](pattern-builder.md) - Related: A builder can use typestate to enforce required fields at compile time
