---
name: "pattern-builder"
description: "Builder pattern for complex object construction with required fields. Load when designing constructors with multiple required parameters or optional configuration, such as a UDF with a name, signature, and options"
type: core
scope: "global"
---

# Builder Pattern for Required Fields

## Rule

Use a builder when construction has several required fields. The builder holds the optionality. `build()`
checks completeness. The built type has no `Option` field for data that is always present, so no consumer
unwraps a field the builder already guaranteed.

## Examples

```rust
// ❌ Bad — every `ScalarUDFImpl` method unwraps a field that is always set, and one forgotten field is a
// runtime panic during planning.
pub struct DistanceUdf { name: Option<String>, metric: Option<DistanceMetric>, signature: Signature }

fn name(&self) -> &str { self.name.as_deref().expect("missing name") }
```

```rust
// ✅ Good — the builder holds the `Option`, `build()` refuses an incomplete UDF, `DistanceUdf` is plain.
pub struct DistanceUdf { name: String, metric: DistanceMetric, signature: Signature }

#[derive(Default)]
pub struct DistanceUdfBuilder { name: Option<String>, metric: Option<DistanceMetric> }

impl DistanceUdfBuilder {
    pub fn metric(mut self, metric: DistanceMetric) -> Self {
        self.metric = Some(metric);
        self
    }

    pub fn build(self) -> Result<DistanceUdf, BuildError> {
        Ok(DistanceUdf {
            name: self.name.ok_or(BuildError::MissingName)?,
            metric: self.metric.ok_or(BuildError::MissingMetric)?,
            signature: Signature::any(2, Volatility::Immutable),
        })
    }
}
```

```rust
// ✅ Good — a typestate builder; `build()` exists only once every required field is set.
pub struct Missing;
pub struct Set<T>(T);

pub struct DistanceUdfBuilder<Name, Metric> { name: Name, metric: Metric }

impl<Metric> DistanceUdfBuilder<Missing, Metric> {
    pub fn name(self, name: String) -> DistanceUdfBuilder<Set<String>, Metric> {
        DistanceUdfBuilder { name: Set(name), metric: self.metric }
    }
}

impl DistanceUdfBuilder<Set<String>, Set<DistanceMetric>> {
    pub fn build(self) -> DistanceUdf {
        DistanceUdf {
            name: self.name.0,
            metric: self.metric.0,
            signature: Signature::any(2, Volatility::Immutable),
        }
    }
}
```

## Why It Matters

A required field typed as `Option` makes every consumer handle a `None` that cannot occur. For a UDF those
consumers are `name`, `return_type`, and `invoke_with_args`, each called by the planner or executor. The
builder puts construction in one place. The built type guarantees its fields, so the
unwrap-a-guaranteed-field panic cannot be written.

## Pragmatism Caveat

A struct with two or three required fields that are all available at once takes a plain `new()`. Most UDFs
are this case: `ScalarUDF::from(CosineDistance::new())`. Use a builder when there are many fields, a mix of
required and optional, or an order that matters. Use a typestate builder when a misuse would be a serious
bug. A runtime `build() -> Result` is enough for options objects, where a clear error message is the whole
requirement.

## Checklist

Before committing code, verify:

- [ ] The built type has no `Option` field for data that is always present
- [ ] `build()` checks that every required field is set
- [ ] No consumer of the built type unwraps a field the builder guarantees
- [ ] A struct with few required fields uses `new()`, not a builder
- [ ] A typestate builder was considered where a misuse would be serious enough to warrant compile-time
      enforcement

## References

- [principle-type-driven-design](principle-type-driven-design.md) - Foundation: The principle this pattern implements
- [pattern-typestate](pattern-typestate.md) - Related: The typestate form of a builder
