---
name: "principle-validate-at-edge"
description: "Validate at the Edge (hard shell, soft core) — parse untrusted input once at the boundary. Load when checking UDF argument types, parsing literal or option arguments, or decoding vectors from external encodings"
type: "principle"
scope: "global"
---

# Validate at the Edge (Hard Shell, Soft Core)

## Rule

Every value that enters a function from outside arrives untrusted and typed wider than reality: the argument
`DataType`s a planner hands to `return_type`, a literal option such as a metric name, a session config
option, a vector encoded in a `Utf8` or `Binary` column. It is parsed once, at the boundary, into a type the
rest of the code trusts. Past that point no function re-checks. The boundary is the hard shell; the kernel is
the soft core.

- Parsing lives in `FromStr` or `TryFrom`, never in a standalone `parse` function or an `invoke_with_args`
  body. That is the one place a newtype's invariant is established, and it composes with serde's `try_from`
  and `?`.
- Planning-time checks (`return_type`, `coerce_types`) turn argument types into domain types. A kernel takes
  those types and the downcast arrays, never raw `ColumnarValue`s it must re-inspect.
- Malformed input becomes a typed error at the edge: `plan_err!` for a bad signature, `exec_err!` naming the
  row for a bad value. It never becomes a panic inside a kernel.
- Unit and convention conversions happen once, at the boundary: degrees to radians, distance to similarity,
  a string encoding to `f32` values.

## Examples

1. **Parse a literal option into a domain type**

```rust
// ❌ Bad — the kernel matches strings per batch; a typo is discovered only when the first batch runs.
fn distance(metric: &str, a: &[f32], b: &[f32]) -> Result<f32> {
    match metric {
        "cosine" => Ok(cosine(a, b)),
        "l2" => Ok(l2(a, b)),
        other => exec_err!("unknown metric {other}"),
    }
}
```

```rust
// ✅ Good — the literal is parsed once; the kernel takes a `DistanceMetric` and cannot see a bad name.
let metric: DistanceMetric = metric_literal.parse()?;
let distances = distance_kernel(metric, &left, &right)?;
```

2. **Cross-argument constraints belong to a composite type**

```rust
// ✅ Good — matching dimensions is an invariant of the pair, checked once at planning time.
pub struct VectorPair { dimension: Dimension }

impl TryFrom<(&DataType, &DataType)> for VectorPair {
    type Error = DataFusionError;

    fn try_from((left, right): (&DataType, &DataType)) -> Result<Self> {
        let left = Dimension::try_from(left)?;
        let right = Dimension::try_from(right)?;
        if left != right {
            return plan_err!("vector dimensions differ: {left} and {right}");
        }
        Ok(Self { dimension: left })
    }
}
```

3. **External encodings are decoded fallibly**

```rust
// ❌ Bad — one malformed string panics the query.
let vector: Vec<f32> = serde_json::from_str(text).unwrap();
```

```rust
// ✅ Good — the error names the row and the cause.
let vector: Vec<f32> = serde_json::from_str(text)
    .map_err(|err| exec_datafusion_err!("row {row}: invalid vector literal: {err}"))?;
```

## Why It Matters

A UDF sits between a planner that hands it whatever types the user wrote, literal options typed as strings,
and data whose encoding nobody promised. A value trusted where it lands fails somewhere else: a panic deep in
a kernel caused by a mismatched dimension the planner accepted, with nothing in the error tying the two
together.

One narrow point localizes the fix. Because the invariant lives in `FromStr` or `TryFrom`, every entry point
enforces it identically: the SQL path, the DataFrame API, a direct kernel call in a test. Scattered per-layer
checks are three partial contracts, and no one of them is the whole contract.

## Pragmatism Caveat

What a check depends on decides where it belongs:

- **Only the incoming value or type: the edge.** Argument count, data types, dimensions, option names,
  cross-argument constraints.
- **Per-row data: the kernel.** A null, a zero-norm vector, a list whose length differs from its neighbours
  in a variable-length `ListArray`. These need the values, and planning does not have them.

A per-row check stays out of planning. A type check stays out of the kernel. A function taking a parsed type
does not check its fields again. An undocumented re-validation is dead code that hides where the real
contract lives.

## Checklist

Before committing code, verify:

- [ ] Every invariant is established in `FromStr` or `TryFrom`, never in an `invoke_with_args` body or a
      standalone `parse`
- [ ] Argument types are checked at planning time and turned into domain types; kernels take the parsed
      types, never raw `ColumnarValue`s to re-inspect
- [ ] Cross-argument constraints are invariants of a composite type, never checks in one function that
      needed them
- [ ] Literal options and config values are parsed into validated types once; consumers do not re-check
- [ ] External encodings are decoded fallibly, with errors naming the row and the cause, never `unwrap`
- [ ] Downstream functions contain no re-validation of what the boundary guaranteed
- [ ] Unit and convention conversions happen once, at the boundary
- [ ] Checks that require per-row data stay in the kernel

## References

- [principle-type-driven-design](principle-type-driven-design.md) - Related: The edge produces the validated
  types that make illegal states unrepresentable
- [pattern-newtype](pattern-newtype.md) - Related: The newtype is where a parsed invariant lives
- [principle-least-surprise](principle-least-surprise.md) - Related: A function whose parameter is a parsed
  type behaves as though it trusts it

## External References

- [Parse, Don't Validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
- [Using Types To Guarantee Domain Invariants](https://lpalmieri.com/posts/2020-12-11-zero-to-production-6-domain-modelling/)
- [Architecture Patterns with Python (O'Reilly)](https://www.oreilly.com/library/view/architecture-patterns-with/9781492052197/)
