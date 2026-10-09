---
name: "rust-attrs-derived"
description: "Derive list ordering: std traits first from Debug to Hash, then fully-qualified external derive macros. Load when adding a #[derive], reordering one, or reviewing a type's derived traits"
type: "core"
scope: "global"
---

# Derive Attributes

**A derive list is written in one fixed order, so the shape of the list carries information.** A reader
scans it and does not parse it. Lint attributes are owned by [rust-attrs-lints](rust-attrs-lints.md).

## 1. Standard Traits First, External Macros Last

A derive list has two halves in a fixed order: standard-library traits, then derive macros from dependencies.
`#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]` is the shape.

The split makes a long list readable at a glance. The std half says what the type is: comparable, copyable,
hashable. The external half says what the type participates in: serialization, configuration, argument
parsing. An interleaved list must be read whole to answer either question.

## 2. The Standard Order: `Debug` First, `Hash` Last

Within the std half, derives follow this order, omitting whatever does not apply:

```
Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash
```

`Debug` leads because nearly every type has it and a reader scans for it. `Hash` trails because it must agree
with `Eq`, so it reads after the equality traits. The comparison traits stay in supertrait order: `PartialEq`
before `Eq`, `PartialOrd` before `Ord`.

A `ScalarUDFImpl` is the common case. DataFusion compares and hashes UDFs when it deduplicates and
simplifies expressions, so the implementing struct derives `Debug, Clone, PartialEq, Eq, Hash`, and its
`Signature` field supports all of them.

```rust
// ❌ Bad — no two lists agree, so answering "is this hashable?" takes a full read of each list every time.
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum DistanceMetric { /* ... */ }

#[derive(Eq, Hash, Debug, PartialEq, Clone)]
pub struct CosineDistanceUdf { signature: Signature }
```

```rust
// ✅ Good — one order everywhere, so the shape of the list carries the information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistanceMetric { /* ... */ }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosineDistanceUdf { signature: Signature }
```

## 3. External Derive Macros Are Fully Qualified

Every derive macro from a dependency is written at its full path: `serde::Serialize`, `serde::Deserialize`.
The macro is never imported. A derive macro usually shares its name with the trait it implements, so a bare
`Serialize` in a derive list names neither its crate nor which of the two it is. The qualified path answers
both. It also keeps a name out of the prologue that is used only inside an attribute.

```rust
// ❌ Bad — imports that exist only to be spelled inside `derive`; a second crate exporting `Serialize` is indistinguishable here.
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorOptions { /* ... */ }
```

```rust
// ✅ Good — no imports, and every macro names its origin.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorOptions { /* ... */ }
```

Within the external half, a crate's macros stay together, in the order the crate pairs them:
`serde::Serialize` before `serde::Deserialize`.

Attribute helpers that accompany a derive, such as `#[serde(...)]`, are never qualified. They are namespaced
by the derive that registered them, and a qualified form does not compile.

## Checklist

Before committing code, verify:

- [ ] Standard-library derives come first, external derive macros last
- [ ] The std half follows `Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash`, omitting what
      does not apply
- [ ] `Debug` is first when present; `Hash` is last of the std traits when present
- [ ] Every external derive macro is fully qualified (`serde::Serialize`, `serde::Deserialize`)
- [ ] No derive macro is imported; the prologue carries no name used only inside a `derive`
- [ ] A crate's macros are grouped, with `serde::Serialize` before `serde::Deserialize`
- [ ] Attribute helpers (`#[serde(..)]`) are left unqualified

## References

- [rust-imports](rust-imports.md) - Related: The prologue these derives keep clear, and the same rule for
  one-off `std` paths
- [rust-attrs-lints](rust-attrs-lints.md) - Related: The other attribute family, and its `#[expect]` rule
- [principle-least-surprise](principle-least-surprise.md) - Foundation: A list in a predictable order is read
  by scanning, never by parsing

## External References

- [Rust API Guidelines — Common traits](https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits)
