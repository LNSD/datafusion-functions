---
name: "rust-imports"
description: "Module prologue order: doc, std, external, mod declarations, then self/crate/super; extension traits as _. Load when adding imports or declaring submodules"
type: "core"
scope: "global"
---

# Import and Module Declaration Order

**Every module opens with the same prologue: documentation, `std`, external crates, `mod` declarations, then
local imports.** A reader who knows the order finds any name without searching. Which references between
modules are legal is owned by [rust-mods-graph](rust-mods-graph.md). The items after the prologue are owned by
[rust-mods-members](rust-mods-members.md). This document owns the prologue and the form of every path in it.

## 1. The Prologue Has Five Parts in a Fixed Order

Every module opens with these parts, in this order, separated by blank lines:

0. **Module documentation**: the `//!` comment, before any item.
1. **`std` imports**.
2. **External crate imports**: `datafusion`, `arrow`, and every other dependency.
3. **`mod` declarations**: `pub` and private together, in one alphabetical group.
4. **Local imports and re-exports**: `use` and `pub use` of `self::`, `crate::`, and `super::`.

The `mod` group comes before the local imports because the local imports refer to it. A reader meets the
module's structure first.

```rust
// ❌ Bad — the doc comment is a plain comment, the groups are interleaved, and the `mod` line sits under an import that uses it.
// Distance metrics for vector UDFs.
use crate::dimension::Dimension;
mod cosine;
use std::sync::Arc;
pub use self::cosine::CosineDistance;
```

```rust
// ✅ Good — documentation, std, external, declarations, then local imports.
//! Distance metrics for vector UDFs.

use std::sync::Arc;

use arrow::array::Float32Array;

mod cosine;
mod euclidean;

pub use self::{
    cosine::CosineDistance,
    euclidean::EuclideanDistance,
};
```

## 2. rustfmt Sorts the Groups; the Author Places the `mod` Group

`just fmt` runs nightly rustfmt with the repository's `.rustfmt.toml`. `group_imports = "StdExternalCrate"`
splits each run of `use` statements into the `std`, external, and local groups. `imports_granularity =
"Crate"` merges imports per crate. `imports_layout = "Vertical"` puts each item of a multi-item brace on its
own line. `reorder_modules = true` sorts `mod` declarations alphabetically. None of that is reviewed by hand.

The formatter does not move the `mod` group. Item order is preserved as written, so a prologue with the group
in the wrong place formats cleanly and stays wrong. Placement is checked in review.

## 3. Submodule Types Travel Through `self::`

A parent refers to a type declared in its submodule through `self::`, in the re-export and in every import. A
bare module name reads like an external crate name and becomes ambiguous the day a dependency of that name is
added. A `crate::` path to a child breaks when the module moves.

```rust
// ❌ Bad — the parent reaches its own child through the crate root; moving the module breaks every line.
mod metric;

pub use crate::distance::metric::DistanceMetric;
```

```rust
// ✅ Good — the child is addressed relative to the parent that declares it.
mod metric;

pub use self::metric::DistanceMetric;
use self::metric::UnknownMetricError;
```

The rule covers the private form. A parent that uses a submodule type without re-exporting it writes
`use self::metric::UnknownMetricError;`.

## 4. Import From the Defining Module

Import an item from the module that declares it. A re-export is for consumers outside the package. Inside the
package it hides where the item lives and gives one type two paths.

```rust
// ✅ Good — the path names the defining module; `use crate::DistanceMetric;` would hide where it is declared.
use crate::distance::DistanceMetric;
```

## 5. Siblings Use `super::`; `super::super::` Is Prohibited

A module reaches a sibling with `use super::sibling::Item;`. That is the only relative upward path. Which
edges may exist is owned by [rust-mods-graph](rust-mods-graph.md).

`super::super::` is prohibited in `use` statements, inline paths, and intra-doc links. A path that climbs two
levels cannot be read without knowing where the file sits, and it resolves to something else when either
module moves. Address the item from the crate root.

```rust
// ❌ Bad — unreadable without the file's position in the tree; moving either module changes what it resolves to.
use super::super::DimensionError;
```

```rust
// ✅ Good — an absolute path inside the crate, readable in isolation.
use crate::dimension::DimensionError;
```

A `super::super::` that seems necessary is a placement problem. The item belongs closer to its users, or the
two modules belong under a shared parent.

## 6. Extension Traits Are Imported As `_`

A trait imported only so its methods resolve is imported without binding its name.

```rust
// ✅ Good — `.len()` and `.is_null()` resolve on Arrow arrays, and no name is claimed.
use arrow::array::Array as _;
```

Two things follow. Same-named traits coexist: `use std::fmt::Write; use std::io::Write;` is error `E0252`,
and both imported `as _` resolve. The import stays live only through method calls: a named import is kept
alive by a bound, an `impl`, or a qualified call, so it outlives the calls it was added for, and an `as _`
import is reported unused the day the last call goes. `just clippy` rejects that warning.

Import the trait by name when the name is used: in a bound (`fn f<A: Array>(..)`), an
`impl ScalarUDFImpl for CosineDistance`, or a qualified call (`<T as ScalarUDFImpl>::name`).

```rust
// ❌ Bad — both names are bound and never referenced; each blocks another `Array` or `Write` the module may need.
use arrow::array::Array;
use std::io::Write;
```

## 7. One-Off `std` Items, Attribute Macros, and Globs Are Not Imported

Three cases stay inline:

- **One-off `std` items** used once or twice in a file are written fully qualified: `std::iter::zip`,
  `std::cmp::Ordering`. The qualification is the documentation. Two `std` modules are never imported from:
  `std::fmt` ([rust-fn-fmt](rust-fn-fmt.md)) and `std::str::FromStr` ([rust-fn-parse](rust-fn-parse.md)).
- **Attribute macros** from external crates are written by their full path at the use site, such as
  `#[datafusion_macros::user_doc(...)]`.
- **Glob imports** do not appear in production code. The one accepted glob is `use super::*;` at the top of
  a `#[cfg(test)] mod tests` module.

```rust
// ✅ Good — qualified at the use site; the prologue carries no name the file mentions once.
fn nearest_first(a: &Neighbor, b: &Neighbor) -> std::cmp::Ordering {
    a.distance.total_cmp(&b.distance)
}
```

## Checklist

Before committing code, verify:

- [ ] The module opens with its `//!` documentation, before any item
- [ ] `std` imports, then external crate imports, each in its own group
- [ ] `mod` declarations form one group after the external imports and before the local imports
- [ ] Local `use` and `pub use` of `self::`, `crate::`, and `super::` come last
- [ ] Submodule types are imported and re-exported through `self::`, never through `crate::` or a bare name
- [ ] Items are imported from the module that declares them, never through a re-export
- [ ] A sibling is reached with `super::`; no path contains `super::super::`, doc links included
- [ ] Traits imported only for their methods are imported `as _`; a trait is imported by name only when the
      name appears in a bound, an `impl`, or a qualified call
- [ ] One-off `std` items and attribute macros are written fully qualified, never imported
- [ ] No glob import outside `use super::*;` in a test module
- [ ] `just fmt` has run, so grouping, granularity, layout, and `mod` sorting are the formatter's output

## References

- [rust-mods-files](rust-mods-files.md) - Related: Module file layout; this document owns the prologue inside each file
- [rust-mods-graph](rust-mods-graph.md) - Related: Which references between modules are legal; this document owns the form those paths take
- [rust-mods-members](rust-mods-members.md) - Related: Ordering of the items that follow the prologue
- [rust-fn-fmt](rust-fn-fmt.md) - Related: Owns the never-import rule for `std::fmt`
- [rust-fn-parse](rust-fn-parse.md) - Related: Owns the never-import rule for `std::str::FromStr`
- [rust-docs-rustdoc](rust-docs-rustdoc.md) - Related: The `//!` comment that opens the prologue

## External References

- [rustfmt configuration](https://rust-lang.github.io/rustfmt/) - The `group_imports`, `imports_granularity`, `imports_layout`, and `reorder_modules` options
