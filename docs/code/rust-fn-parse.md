---
name: "rust-fn-parse"
description: "FromStr fully qualified and never imported, .parse() at call sites, annotate only when inference needs it. Load when implementing FromStr or parsing a string"
type: "core"
scope: "global"
---

# String Parsing With `FromStr`

**`FromStr` is named once per type, in the `impl` header, and never imported.** That a parseable type
implements `FromStr` is owned by [principle-least-surprise](principle-least-surprise.md). That its invariant is
checked there and nowhere else is owned by [principle-validate-at-edge](principle-validate-at-edge.md). This
document is about how the impl and its call sites are written.

## 1. `FromStr` Is Implemented Fully Qualified

Write `impl std::str::FromStr for T`. Never import the trait. `FromStr` appears once per type, in the `impl`
header, so an import shortens nothing. The qualified path names the trait without a trip to the imports. A
bare `impl FromStr for T` could name any of several traits called `FromStr` in the dependency graph.

```rust
// ✅ Good — no import, and the header names the trait unambiguously.
impl std::str::FromStr for DistanceMetric {
    type Err = DataFusionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "cosine" => Ok(Self::Cosine),
            "l2" => Ok(Self::L2),
            other => plan_err!("unknown distance metric '{other}', expected 'cosine' or 'l2'"),
        }
    }
}
```

The associated error type is `Err`. A type parsed from a SQL literal or option uses `DataFusionError`, so
the failure reaches the user as a planning error through `?`.

## 2. Call Sites Use `.parse()`

`str::parse` is an inherent method bounded by `FromStr`, so the trait need not be in scope to call it. Nothing
outside the `impl` names the trait. Never call `T::from_str(s)` or `FromStr::from_str(s)`. They reach the same
function by a longer path, and `T::from_str` needs the trait in scope, which brings back the import.

```rust
// ❌ Bad — the trait path at the call site, and an import to make it resolve.
use std::str::FromStr;

let metric = DistanceMetric::from_str(metric_literal)?;
```

```rust
// ✅ Good — the inherent method; no import, no trait named.
let metric = metric_literal.parse::<DistanceMetric>()?;
```

The turbofish here exists only because the snippet has nothing to infer from. Most parse sites need none
([§3](#3-annotate-only-when-inference-needs-it)).

## 3. Annotate Only When Inference Needs It

Write `.parse()?` bare. If it compiles, the compiler knew the type, and naming it again adds a token the
reader checks against the signature that decides it. Most sites are inferred: a direct return, a struct
field initializer, an argument to a typed parameter, a comparison against a typed value, or a later use of
the binding pins the type.

```rust
// ❌ Bad — the return type already fixes the type; the annotation becomes a lie when the signature changes.
fn distance_metric(raw: &str) -> Result<DistanceMetric> {
    raw.parse::<DistanceMetric>()
}
```

```rust
// ✅ Good — the type is stated once, by the signature.
fn distance_metric(raw: &str) -> Result<DistanceMetric> {
    raw.parse()
}
```

When inference cannot resolve the target, state it with a turbofish on the call, never as an annotation on the
binding. The turbofish rides with the call into any expression position, including one with no binding.

```rust
// ❌ Bad — the type sits on the binding; inline the binding and the annotation has nowhere to go.
let dimension: Dimension = raw_dimension.parse()?;
```

```rust
// ✅ Good — the type rides with the call.
let dimension = raw_dimension.parse::<Dimension>()?;
```

## 4. Not Every `from_str` Is `FromStr`

`serde_json::from_str` and its siblings in other format crates are deserializer entry points, not the
`FromStr` method. They take a `&str` and a `Deserialize` target and are called by their qualified path:
`serde_json::from_str::<Vec<f32>>(text)`. Nothing here applies to them.

## Checklist

Before committing code, verify:

- [ ] `FromStr` is implemented as `impl std::str::FromStr for T`, fully qualified
- [ ] No file imports `std::str::FromStr`
- [ ] Call sites use `.parse()`; no `T::from_str(..)` or `FromStr::from_str(..)`
- [ ] `.parse()` is written bare wherever the type is inferable; no annotation restates what a signature,
      field type, or parameter already pins
- [ ] Where inference cannot resolve the target, it is stated with a turbofish on the call, never as an
      annotation on the binding
- [ ] `serde_json::from_str` and other deserializer entry points are left alone

## References

- [rust-fn](rust-fn.md) - Extends: What a constructor is named, and why parsing gets `FromStr` instead of
  a custom one
- [principle-least-surprise](principle-least-surprise.md) - Foundation: Why a parseable type implements
  `FromStr` and not a custom constructor
- [principle-validate-at-edge](principle-validate-at-edge.md) - Foundation: `FromStr` is where the invariant is
  established, and the only place it is checked
- [rust-fn-fmt](rust-fn-fmt.md) - Related: The `Display` half of the round-trip
- [pattern-newtype](pattern-newtype.md) - Related: The types that earn a `FromStr` impl
- [rust-imports](rust-imports.md) - Related: How imports are written, and which paths stay qualified
- [rust-fn-unchecked](rust-fn-unchecked.md) - Related: The narrow cases where construction may bypass
  `from_str`

## External References

- [`std::str::FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html)
- [`str::parse`](https://doc.rust-lang.org/std/primitive.str.html#method.parse)
