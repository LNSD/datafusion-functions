---
name: "rust-fn-fmt"
description: "std::fmt impls, fully qualified and never imported; Display/Debug/LowerHex/UpperHex with doctests pinning the rendering. Load when implementing Display, Debug, or a hex trait"
type: "core"
scope: "global"
---

# Formatting Traits (`std::fmt`)

**Every `std::fmt` item is spelled at its full path, and every rendering is pinned by a doctest.** A
rendering lands in error messages, `EXPLAIN` output, and the SQL a user types back, so it is a contract and
is tested as one. The `FromStr` half of the round-trip is owned by [rust-fn-parse](rust-fn-parse.md).

## 1. Fully Qualified, Never Imported

Every `std::fmt` item is written at its full path: the trait in the `impl` header, `std::fmt::Formatter<'_>`
in the signature, `std::fmt::Result` as the return type. **Nothing from `std::fmt` is ever imported.**

The names collide. `Result` and `Error` from `std::fmt` collide with `datafusion::error::Result` and the
crate's own. `Write` collides with `std::io::Write`. A bare `Result` in a `fmt` signature reads as
DataFusion's `Result` to anyone who did not scroll to the prologue. Qualifying costs ten characters and
removes the ambiguity.

```rust
// ❌ Bad — `Result` here is `std::fmt`'s, nothing at the use site says so, and the import shadows DataFusion's.
use std::fmt::{
    Display,
    Formatter,
    Result,
};

impl Display for Dimension {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {}
}
```

```rust
// ✅ Good — no import, and every type in the signature names itself.
impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {}
}
```

## 2. Delegate Through the Trait, Not Through `write!`

A newtype that renders as its inner value calls the trait function directly.

```rust
// ❌ Bad — `write!` starts a fresh format spec, so the caller's `{:>12}` pads nothing and `{:#x}` loses its prefix.
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
}
```

```rust
// ✅ Good — the same formatter reaches the inner impl, so width, fill, precision, and the alternate flag propagate.
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(&self.0, f)
}
```

The delegation names the trait. `self.0.fmt(f)` is forbidden. An inner byte array or integer implements
several formatting traits, so `self.0.fmt(f)` resolves by inference and silently picks a different rendering
when the surrounding impl changes.

## 3. The Type Documents Its Formatting Surface

A type with more than one formatting trait carries a `## Formatting` section in its docs. The section states
what each trait renders and links to the impls. A reader choosing between `{}`, `{:?}`, and `{:x}` then reads
one section and no impl bodies.

```rust
// ✅ Good — one section answers the choice between `{}` and `{:x}`, and each entry links to its impl.
/// A 64-bit binary quantization code for one vector.
///
/// ## Formatting
///
/// - Use [`std::fmt::Display`] for the 64-character bit string, as compared by Hamming distance.
/// - Use [`std::fmt::LowerHex`] (or [`std::fmt::UpperHex`]) for the 16-digit hexadecimal form.
///
/// See the [`Display`], [`LowerHex`], and [`UpperHex`] impls for usage examples.
///
/// [`Display`]: #impl-Display-for-BinaryCode
/// [`LowerHex`]: #impl-LowerHex-for-BinaryCode
/// [`UpperHex`]: #impl-UpperHex-for-BinaryCode
pub struct BinaryCode(u64);
```

## 4. Every `fmt` Impl Carries a Doctest

The rustdoc goes on the `fmt` method, and it states the rendering with a doctest that asserts the exact
output.

A rendering is a contract. It lands in `plan_err!` and `exec_err!` messages, in the `EXPLAIN` text of a
plan, and in the literal a user passes back as a function argument. Prose describing it drifts; an
assertion does not. Without one, a change to the rendering breaks every consumer and no test.

```rust
// ✅ Good — the doctest is the specification, and it fails the moment the rendering changes.
impl std::fmt::Display for DistanceMetric {
    /// Format the metric as the name accepted in SQL, such as `cosine`.
    ///
    /// ```rust
    /// # use datafusion_functions_vector::DistanceMetric;
    /// assert_eq!(DistanceMetric::Cosine.to_string(), "cosine");
    /// assert_eq!(DistanceMetric::InnerProduct.to_string(), "inner_product");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_sql_name())
    }
}
```

## 5. Hex Traits Come in Pairs and Document the Alternate Flag

A byte- or bit-backed type that implements `LowerHex` implements `UpperHex` too. Callers pick the case at the
format site. A type offering only one forces callers into `to_uppercase()` on a formatted string.

Both impls document that the alternate flag `#` prepends `0x`, and both doctests assert it. The behavior is
inherited from the inner type's impl, so swapping the inner type can change it by accident.

```rust
// ✅ Good — the doc states the flag and the doctest pins both forms; `UpperHex` mirrors this with `{code:X}`.
impl std::fmt::LowerHex for BinaryCode {
    /// Lowercase hex representation of the code. The alternate flag, `#`, adds a `0x` prefix.
    ///
    /// ```rust
    /// # use datafusion_functions_vector::BinaryCode;
    /// let code = BinaryCode::from(0x8f2c_4a77_9f66_bde2);
    /// assert_eq!(format!("{code:x}"), "8f2c4a779f66bde2");
    /// assert_eq!(format!("{code:#x}"), "0x8f2c4a779f66bde2");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::LowerHex::fmt(&self.0, f)
    }
}
```

## 6. `Debug` Is Written When the Derive Is Unhelpful

`#[derive(Debug)]` is the default. A hand-written `Debug` is warranted when the derived output is unreadable.
A wrapper over a 64-bit code derives into a decimal integer that matches no hex dump and no bit string, and
a struct holding a large `Float32Array` derives into thousands of floats.

A hand-written `Debug` picks one of two shapes and documents which:

- **Wrapped**: `BinaryCode(8f2c4a779f66bde2)`, keeping the type name visible in a struct dump.
- **Delegated**: whatever the inner type's `Debug` renders, when the type must be indistinguishable from it
  in output.

The doc on the `fmt` method says what it produces, points readers at the hex traits for a specific case, and
asserts the result in a doctest. A type a doctest cannot construct, such as one that exists only inside a
kernel invocation, carries a `//` comment on the method saying so in place of the doctest.

## 7. `Display` and `FromStr` Round-Trip

When a type implements both, `value.to_string().parse::<T>()` returns the same value. A `Display` that
renders a form its own `FromStr` rejects breaks every query a user writes from an error message or an
`EXPLAIN` line: the metric shown there must parse as an argument. It also silently breaks `serde` round-trips
wherever the pair backs a serialization impl.

`FromStr` may accept more than `Display` produces. A metric that renders `inner_product` and parses
`inner_product` or `dot` is fine. `FromStr` never accepts less.

## Checklist

Before committing code, verify:

- [ ] Formatting traits are implemented as `impl std::fmt::<Trait> for T`, fully qualified
- [ ] No file imports anything from `std::fmt`
- [ ] `f: &mut std::fmt::Formatter<'_>` and `-> std::fmt::Result` are written at full path
- [ ] Delegation calls the trait function (`std::fmt::Display::fmt(&self.0, f)`), never
      `write!(f, "{}", self.0)` and never `self.0.fmt(f)`
- [ ] A type with more than one formatting trait has a `## Formatting` section linking to each impl
- [ ] Every `fmt` impl has rustdoc on the method with a doctest asserting the exact output, or a `//` comment
      saying why a doctest cannot construct the type
- [ ] `LowerHex` and `UpperHex` are implemented together, and both document and assert the `#` alternate flag
- [ ] A hand-written `Debug` says which shape it produces and why the derive was not used
- [ ] `Display` output parses back through `FromStr` to the same value

## References

- [rust-fn](rust-fn.md) - Extends: What a conversion method is named, and why rendering goes through the
  formatting traits
- [rust-fn-parse](rust-fn-parse.md) - Related: The `FromStr` half of the round-trip, and the same fully-qualified rule
- [rust-imports](rust-imports.md) - Related: The general rule that one-off `std` paths stay qualified
- [rust-docs-rustdoc](rust-docs-rustdoc.md) - Related: The rustdoc sections and the doctest carve-out this
  document relies on
- [pattern-newtype](pattern-newtype.md) - Related: The wrappers that need a formatting surface at all

## External References

- [Rust standard library — `std::fmt`](https://doc.rust-lang.org/std/fmt/index.html)
