---
name: "rust-mods-files"
description: "Module file layout without mod.rs: a named module file beside its directory, declaring its sub-modules and re-exporting their public types. Load when creating a module, adding a sub-module, or splitting a file that has grown"
type: "core"
scope: "global"
---

# Module Files

**A module with sub-modules is a named file beside a directory of the same name.** The file declares its
children and re-exports what callers may reach. The invariants behind these rules are stated in
[rust-mods](rust-mods.md). This document owns where a module's files sit and what the named module file
carries.

## 1. Never Use `mod.rs`

Do not use `mod.rs` files. A module with sub-modules is a named file next to a directory of the same name.
The file name is the module name, so a search finds it and a backtrace names it.

```
// ❌ Bad — every module file is called mod.rs; a tab bar and a backtrace both say "mod.rs" and neither says which module.
src/
  distance/
    mod.rs
    metric.rs
```

```
// ✅ Good — the file name is the module name.
src/
  distance.rs
  distance/
    metric.rs
```

Test modules keep `mod.rs`. Every top-level file in `tests/` compiles as its own binary, so a module shared
between suites has no file outside the directory to declare it. Test trees use `mod.rs`, in `tests/` and in
`#[cfg(test)]` trees under `src/` alike. A `mod.rs` anywhere else is a violation.

## 2. The Module File Declares Its Children and Re-Exports Them

The named module file declares its sub-modules and re-exports their public types. Each sub-module file
carries one concern. The re-export is the module's public surface: a type callers outside name is re-exported,
and a type only the sub-modules use is not.

```rust
// ✅ Good — callers depend on `distance::DistanceMetric`, so moving the type between sub-modules edits one line.
mod cosine;
mod metric;

pub use self::{
    cosine::CosineDistance,
    metric::DistanceMetric,
};
```

Code the module file holds beyond declarations and re-exports is reachable only downward, never from the
sub-modules it declares ([rust-mods-graph](rust-mods-graph.md)).

## Checklist

Before committing code, verify:

- [ ] No `mod.rs` file exists outside test trees
- [ ] Named module files (e.g. `distance.rs`) sit next to directories of the same name
- [ ] Sub-modules are declared with `mod` in the parent module file
- [ ] Types callers outside the module need are re-exported from it with `pub use self::`

## References

- [rust-mods](rust-mods.md) - Extends: The invariants these rules make operational
- [rust-mods-graph](rust-mods-graph.md) - Related: Which references between these files are legal
- [rust-imports](rust-imports.md) - Related: Owns the prologue and the `self::` re-export form

## External References

- [The Rust Reference: Module source filenames](https://doc.rust-lang.org/reference/items/modules.html#module-source-filenames) - The `name.rs` plus `name/` layout this rule requires
