---
name: "rust-mods-graph"
description: "The reference graph between modules: no reference into the file that declared you, no sibling cycles. Load when splitting a module, or when a sub-module reaches back into its parent"
type: "core"
scope: "global"
---

# Module Reference Graph

**A reference between modules runs down to a child or across to a sibling, never back to the file that
declared the module.** The invariant is stated in [rust-mods](rust-mods.md). Where the files sit is owned by
[rust-mods-files](rust-mods-files.md). This document owns which references between modules are legal.

## 1. A Sub-Module Does Not Reach Into the File That Declares It

The module file may hold code. What it holds is unavailable to its children. A sub-module refers to its
siblings and to items addressed from the crate root, never to an item declared one level up. A child that
names its parent's item cannot be read, moved, or tested without the parent.

```rust
// ❌ Bad — the parent registers the UDF and the UDF names the parent's type; the split separated nothing.
// In src/distance.rs
pub enum DistanceMetric { Cosine, Euclidean }

// In src/distance/udf.rs
use super::DistanceMetric;
```

```rust
// ✅ Good — the type sits in its own sub-module, the parent re-exports it, and the UDF names a sibling.
// In src/distance.rs
mod metric;
mod udf;

pub use self::metric::DistanceMetric;

// In src/distance/udf.rs
use super::metric::DistanceMetric;
```

A `use super::Item` that names a type, constant, or function is the signal. The fix is the same every time:
move the item into a sub-module, and re-export it from the parent if callers outside need the name.

## 2. Siblings Are a Legal Edge; a Cycle Is Not

A sub-module reaches a sibling with `use super::sibling::Item`. That edge is legal. Two siblings that import
each other close a loop without either one reaching for its parent, and neither file can be read first.

```rust
// ❌ Bad — neither file reaches the parent, and the pair is unreadable in either order.
// In src/distance/udf.rs
use super::kernel::cosine_distance;

// In src/distance/kernel.rs
use super::udf::DistanceUdf;
```

Break the loop by moving what both need into a third sibling neither imports from, or by merging the two. A
pair of modules that each need the other's types is usually one concern split along the wrong seam.

The form of these paths and the ban on `super::super::` are owned by [rust-imports](rust-imports.md).

## Checklist

Before committing code, verify:

- [ ] Nothing a sub-module needs is declared in the file that declares that sub-module
- [ ] No `use super::Item` names anything but a sibling module
- [ ] No pair of sibling modules imports from each other
- [ ] No sequence of references leads from a module back to that same module

## References

- [rust-mods](rust-mods.md) - Extends: The one-way invariant these rules make operational
- [rust-mods-files](rust-mods-files.md) - Related: Where the module files these references run between sit
- [rust-imports](rust-imports.md) - Related: Owns the path form, the `self::` prefix, and the `super::super::` ban
