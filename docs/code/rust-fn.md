---
name: "rust-fn"
description: "Function names and signatures against std: new is infallible, create or try_new is fallible, Self names the enclosing type, as_/to_/into_ predict cost and ownership. Load when naming a function or choosing a receiver"
type: "core"
scope: "global"
---

# Function Names and Signatures

**The standard library is the reference.** A name that matches `std` needs no documentation. A name that
contradicts it costs a trip to the source. Where `std` has a convention for a name, a receiver, or a return
type, follow it.

The principle, and the cases about behavior instead of naming, are owned by
[principle-least-surprise](principle-least-surprise.md).

## 1. `new` Is Infallible, `create` Is Not

`new` is infallible, cheap, and free of I/O. It takes the values the type needs and returns the type. A
constructor that can fail is named `create` or `try_new` and returns `Result`, so the failure path is visible
at the call site. `try_new` fits a type whose ecosystem already uses it, as the arrow crate does with
`FixedSizeListArray::try_new`. `create` is the default everywhere else.

| Situation                                  | The constructor                                    |
|--------------------------------------------|----------------------------------------------------|
| Always succeeds, no I/O                    | `new`                                              |
| Can fail, built from the type's own inputs | `create` or `try_new`, returning `Result`          |
| Infallible conversion from another type    | `From` (which yields `Into` for free)              |
| Fallible conversion from another type      | `TryFrom` / `TryInto`                              |
| Parsed from a string                       | `FromStr` ([rust-fn-parse](rust-fn-parse.md))            |
| Reads a file or does expensive work        | A named constructor: `open`, `load`                |
| Many fields, some optional                 | A builder ([pattern-builder](pattern-builder.md))  |

`TryFrom` converts one source value into this type and is what `?` and `.try_into()` reach for. `create`
builds from several inputs, such as a dimension and a distance metric, where there is no single source
value.

```rust
// ✅ Good — the name says a file is read, so its cost and its failure surprise nobody.
pub fn load(path: &Path) -> Result<Self> {}
```

A `create` or `try_new` that cannot fail is renamed `new`. `make` is not used. `build` is reserved for the
terminal method of a builder.

## 2. `Self` Names the Enclosing Type

Inside an `impl`, the type being implemented is written `Self`: in the return position, inside the `Ok`
of a `Result`, and in the struct or enum literal the body builds. The `impl` header already states the name.

The cost of repeating it is paid at rename. With `Self`, renaming a type touches the header alone. With the
name repeated, the rename lands as a diff over the whole `impl` and buries whatever real change ships with it.

The rule covers the enclosing type only. A method that returns a different type names that type.

```rust
// ❌ Bad — renaming `VectorColumn` rewrites every constructor and literal in the `impl`.
impl VectorColumn {
    pub fn new(values: Float32Array, dimension: Dimension) -> VectorColumn {
        VectorColumn { values, dimension }
    }
}

// ✅ Good — the header is the only place the name appears.
impl VectorColumn {
    pub fn new(values: Float32Array, dimension: Dimension) -> Self {
        Self { values, dimension }
    }
}
```

Trait impls follow the same rule, as `std` does: `fn from(value: Raw) -> Self`, `fn default() -> Self`.

## 3. Conversions Predict Cost and Ownership

The three conversion prefixes promise what a call costs and what it does to the receiver. A caller reads
`as_` and puts the call in a per-row loop.

| Prefix   | Receiver | Cost                  | Returns                            |
|----------|----------|-----------------------|------------------------------------|
| `as_*`   | `&self`  | Free; never allocates | A borrowed view (`&str`, `&[f32]`) |
| `to_*`   | `&self`  | May allocate          | A new owned value                  |
| `into_*` | `self`   | Free or near-free     | A different owned type             |

```rust
// ❌ Bad — `as_` allocates on every row, and `to_` consumes the receiver.
pub fn as_vec(&self) -> Vec<f32> { self.values.to_vec() }
pub fn to_validated(self) -> ValidatedOptions {}

// ✅ Good — each prefix matches its receiver and its cost.
pub fn to_vec(&self) -> Vec<f32> { self.values.to_vec() }
pub fn into_validated(self) -> ValidatedOptions {}
```

`into_inner` is the name for handing back the value a wrapper wraps. A newtype that hides its field provides
it, so callers never reach for `.0`. The iterator trio follows the same rule: `iter(&self)`,
`iter_mut(&mut self)`, `into_iter(self)`.

## 4. Predicates Return `bool`

`is_*` and `has_*` return `bool` and nothing else. A predicate that returns data does not compile in an `if`,
so callers write `.is_some()` and the name bought nothing.

```rust
// ❌ Bad — `if column.has_nulls()` does not compile.
pub fn has_nulls(&self) -> Option<usize> { self.null_count }

// ✅ Good — a predicate and an accessor, each doing one thing.
pub fn has_nulls(&self) -> bool { self.null_count > 0 }
pub fn null_count(&self) -> usize { self.null_count }
```

A collection with `len` also has `is_empty`. `std` pairs them, and clippy expects the pair on a public type.

## 5. `try_` Is the Fallible Variant

`try_*` names the fallible sibling of an operation that otherwise panics or is total: `try_reserve`,
`try_into`. The prefix is correct only when the un-prefixed operation exists. A function with no infallible
counterpart is `something` returning `Result`, never `try_something`. The one constructor exception is
`try_new`, admitted by [§1](#1-new-is-infallible-create-is-not) without an infallible `new` beside it.

## 6. Accessors Are Bare Nouns

A field accessor is named for what it returns: `len()`, `dimension()`, `metric()`. `std` reserves `get` for a
lookup that can miss (`slice::get`, `HashMap::get`) and returns `Option`. A `get_` prefix on an infallible
read makes every caller check for a miss that cannot happen.

A mutating setter takes `&mut self` and is named `set_*`. A method that returns a modified copy is `with_*`
and takes `self`.

## 7. The Receiver Matches the Prefix

The receiver is part of the promise. `as_*` and `to_*` borrow. `into_*` and `with_*` consume. `set_*` takes
`&mut self`. A method that consumes `self` under a borrowing name forces clones the caller should not need.
One that borrows under a consuming name leaves a value alive the caller expected to have given away.

Taking `self` is how a type moves through a pipeline without a clone, and it makes an invalid intermediate
state unconstructible ([pattern-typestate](pattern-typestate.md)).

## 8. A Domain Verb Replaces a Prefix Only When Documented

Every rule above is a `std` convention, and together they produce a function a Rust developer can use
correctly from its signature. `fn as_slice(&self) -> &[f32]` is free, borrowing, and infallible before anyone
opens it. `From`, `TryFrom`, `FromStr`, `is_empty`, and `into_iter` wire the type into `?`, `.into()`,
`.parse()`, serde, clippy, and `for` loops with no extra code.

A domain term that is clearer than the convention may replace it. `normalize` on a vector beats
`into_normalized`, even though it consumes `self`, because normalize is the established verb. The trade is
documented at the declaration ([principle-least-surprise](principle-least-surprise.md)). A name that reads
like a convention and does something else is never acceptable.

## Checklist

Before committing code, verify:

- [ ] `new` is infallible, cheap, and performs no I/O; anything that reads a file or does expensive work has
      a name that says so
- [ ] A fallible constructor is `create` or `try_new` returning `Result`, never a `new` that returns
      `Result`
- [ ] `TryFrom` is used where there is a single source value to convert; `create` where there is not
- [ ] No `make` was introduced, and `build` appears only as a builder's terminal method
- [ ] Inside an `impl`, the enclosing type is written `Self` in return types, in `Result`, and in literals
- [ ] `as_*` borrows without allocating, `to_*` returns an owned value, `into_*` consumes `self`
- [ ] A wrapper exposes `into_inner` rather than leaving callers to reach for `.0`
- [ ] `is_*` and `has_*` return `bool`; a type with `len` also has `is_empty`
- [ ] `try_*` is used only where the infallible counterpart exists, with `try_new` as the one constructor
      exception
- [ ] Field accessors are bare nouns; `get` is reserved for lookups that can miss
- [ ] Every receiver matches its prefix: borrowing names take `&self`, consuming names take `self`

## References

- [principle-least-surprise](principle-least-surprise.md) - Foundation: Why a name is a contract, and the
  behavioral cases that are not about naming
- [rust-fn-conv](rust-fn-conv.md) - Related: Conversions between types, and when an `as` cast is allowed
- [rust-fn-parse](rust-fn-parse.md) - Related: `FromStr` as the parsing constructor, and its call sites
- [rust-fn-fmt](rust-fn-fmt.md) - Related: Rendering a type as text through the `std::fmt` traits
- [rust-fn-unchecked](rust-fn-unchecked.md) - Related: Constructors that skip validation, and how they are named
- [pattern-newtype](pattern-newtype.md) - Related: The wrappers that need `as_*` and `into_inner`
- [pattern-builder](pattern-builder.md) - Related: Where `build` is the correct terminal name
- [pattern-typestate](pattern-typestate.md) - Related: Consuming receivers as a correctness tool

## External References

- [Rust API Guidelines — Naming](https://rust-lang.github.io/api-guidelines/naming.html)
