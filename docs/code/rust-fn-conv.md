---
name: "rust-fn-conv"
description: "Conversions: implement From, TryFrom when fallible, .into() at call sites, and when an as cast is allowed. Load when converting between types or reviewing an as cast"
type: "core"
scope: "global"
---

# Type Conversions

**A conversion that cannot fail is `From`; one that can is `TryFrom`; `as` is allowed only with a stated
bound; `transmute` is never used.** Two of the four are checked by the compiler, one is silent about lost
data, and one is unchecked entirely.

| Mechanism             | Use it                                                 |
|-----------------------|--------------------------------------------------------|
| `From` / `Into`       | Whenever the conversion cannot fail                    |
| `TryFrom` / `TryInto` | Whenever it can                                        |
| `as`                  | Only where the bound is proven and stated in a comment |
| `transmute`           | Never                                                  |

This document owns which mechanism to use. The name of a conversion method (`as_*`, `to_*`, `into_*`) is
owned by [rust-fn](rust-fn.md). Parsing from a string is owned by [rust-fn-parse](rust-fn-parse.md).

## 1. Implement `From`, Never `Into`

Write the `From` impl. The standard library's blanket `impl<T, U: From<T>> Into<U> for T` gives `Into` for
free. A hand-written `Into` gives nothing back: `RowIndex::from(raw)` fails to compile, and the impl cannot
satisfy a `From` bound.

```rust
// ✅ Good — one impl, both call forms.
impl From<usize> for RowIndex {
    fn from(raw: usize) -> Self {}
}
```

A trait bound is the reverse. Accept `impl Into<T>` (`pub fn with_alias(alias: impl Into<FunctionName>)`).
An `Into` bound is satisfied by every `From` impl and by any direct `Into` a foreign crate wrote.

## 2. Call `.into()`, Not `T::from()`

At a call site the target type is already known from the parameter, the field, or the return type. `.into()`
reads in the order the data flows and repeats nothing.

```rust
// ❌ Bad — the destination type is named again, inside an expression that already establishes it.
fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
    let distances: ArrayRef = Arc::new(distance_kernel(&args)?);
    Ok(ColumnarValue::from(distances))
}

// ✅ Good — the value, then the conversion; the type comes from the return type.
fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
    let distances: ArrayRef = Arc::new(distance_kernel(&args)?);
    Ok(distances.into())
}
```

`T::from(x)` stays where inference has nothing to work with, and in a `map` where the function reference is
the point: `rows.map(RowIndex::from)`.

## 3. `TryFrom` Carries the Failure

A conversion that can fail is a `TryFrom` impl with an error the caller can report. It is never a free
function returning `Option`, and never a `From` impl that asserts its input. A panicking `From` claims
totality it does not have, and every caller inherits a panic the signature does not show.

```rust
// ✅ Good — the failure is in the type, with an error that reaches the user as a planning error.
impl TryFrom<usize> for Dimension {
    type Error = DataFusionError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        NonZeroUsize::new(value)
            .map(Self)
            .ok_or_else(|| plan_datafusion_err!("vector dimension must be non-zero"))
    }
}
```

`TryFrom` gives `TryInto`, and both compose with `?`.

## 4. `as` Is Silent About What It Loses

`as` always compiles and always produces a value. It reports nothing about what it discarded.

| Conversion                     | What `as` does                                 |
|--------------------------------|------------------------------------------------|
| Wider integer to narrower      | Truncates, keeping the low bits                |
| Signed to unsigned (same size) | Reinterprets the bit pattern                   |
| Float to integer               | Saturates at the bounds; `NaN` becomes `0`     |
| Integer to float               | Rounds, losing precision past the mantissa     |

Each row is a bug that ships silently. Use `try_into()` and handle the failure.

```rust
// ❌ Bad — Arrow's FixedSizeList size is an i32; a dimension past i32::MAX wraps negative, and no test fails.
let size = dimension as i32;

// ✅ Good — the overflow is a typed error at the point it becomes possible.
let size = i32::try_from(dimension)
    .map_err(|_| plan_datafusion_err!("vector dimension {dimension} exceeds the Arrow list size limit"))?;
```

An `as` cast is acceptable only where the bound is structurally guaranteed, and a comment at the cast states
the bound ([rust-docs-comments](rust-docs-comments.md)). "It won't be that big" is not a bound. "Planning
rejected anything above `MAX_DIMENSION`" is.

```rust
// ✅ Good — a proven bound, stated where a reader can check it.
// `return_type` rejects dimensions above MAX_DIMENSION (65_536), so this cannot truncate.
let size = dimension as i32;
```

`transmute` is never used. Nothing in this workspace needs a reinterpretation the type system cannot check.

## 5. Borrowed Conversions Are `AsRef`, Not `Deref`

A borrowed view of an inner value is `AsRef<T>` or an inherent `as_*` method ([rust-fn](rust-fn.md)).

`Deref` exposes every method of the target on the wrapper. Deref to an immutable borrowed type (`str`,
`[f32]`, `Path`) is acceptable; everything it exposes is read-only, and the newtype's invariant survives.
Deref to an owned or mutable target is never written. `Deref<Target = String>` on `FunctionName` offers
`push_str` and `clear`, so any caller with a `&mut` can edit the invariant away.

```rust
// ✅ Good — an explicit borrowed view, and nothing beyond it.
impl AsRef<str> for FunctionName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// 🔶 Acceptable — Deref to the immutable borrowed form exposes only read-only methods.
impl std::ops::Deref for FunctionName {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}
```

A function that only reads a value accepts `impl AsRef<Path>` or `impl AsRef<str>`. That admits `String`,
`&str`, and every newtype above without an allocation at the call site.

## Checklist

Before committing code, verify:

- [ ] Conversions are written as `From` impls; no `Into` impl was written by hand
- [ ] Trait bounds accept `impl Into<T>` rather than requiring `From`
- [ ] Call sites use `.into()` / `.try_into()`; `T::from` appears only where inference has nothing to work
      with, or as a function reference in a `map`
- [ ] Fallible conversions are `TryFrom` with a reportable error; no `From` impl can panic
- [ ] Numeric narrowing uses `try_into()` and handles the failure
- [ ] Every remaining `as` cast has a structurally guaranteed bound, stated in a comment at the cast
- [ ] No `transmute`
- [ ] Borrowed views are `AsRef` or an inherent `as_*` method
- [ ] Any `Deref` impl targets an immutable borrowed type (`str`, `[f32]`, `Path`), never an owned or
      mutable one

## References

- [rust-fn](rust-fn.md) - Extends: What a conversion method is named, and what its receiver promises
- [rust-fn-parse](rust-fn-parse.md) - Related: `FromStr`, the conversion that turns a string into a domain type
- [pattern-newtype](pattern-newtype.md) - Related: The wrappers most of these conversions exist for
- [rust-docs-comments](rust-docs-comments.md) - Related: Owns the comment an `as` cast must carry
- [principle-type-driven-design](principle-type-driven-design.md) - Foundation: A conversion that cannot fail
  is unable to fail by construction

## External References

- [`std::convert`](https://doc.rust-lang.org/std/convert/index.html)
- [Rust API Guidelines — Conversions](https://rust-lang.github.io/api-guidelines/interoperability.html#conversions-use-the-standard-traits-from-asref-asmut-c-conv-traits)
