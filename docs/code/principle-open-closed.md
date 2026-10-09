---
name: "principle-open-closed"
description: "Open/Closed — add behaviour by adding a registry entry or a trait impl, never by editing logic that already works. Load when adding functions, metrics, or element types, extending behaviour across packages, or reviewing repeated match arms"
type: "principle"
scope: "global"
---

# Open/Closed Principle (OCP)

## Rule

A software entity is open for extension and closed to modification of established behavior. New behavior is
added as a registry entry, a new type implementing an existing trait, or a new value handed to a composition
root. Logic that already works is left unedited.

Introduce an extension point when one of these signals fires:

1. **Cross-package extension.** Another package adds the behavior. A package hands DataFusion a value
   implementing `ScalarUDFImpl`; the engine never learns the package exists.
2. **Externally growing variant space.** The variants track something outside the repository: distance
   metrics, embedding element types, SQL function names. Every new variant is additive.
3. **Repeated branching sites.** The same `match` or `if` over the same variant set appears in more than one
   place. One lookup replaces every site.

When no signal fires, a plain `match` is correct. A variant set fixed by a spec, matched in one place, and
local to one package stays a `match`.

## Examples

1. **Registry entry over branching matches**
   Distance metrics are an externally growing variant space. They are a table of specs and one lookup, with
   per-metric behavior carried in the entry.

```rust
// ❌ Bad — every new metric edits two proven functions; a metric added to one and missed in the
// other silently fell through to the default and sorted results in the wrong direction.
pub fn sql_name_for(metric: Metric) -> &'static str {
    match metric {
        Metric::Cosine => "cosine_distance",
        Metric::L2 => "l2_distance",
        Metric::InnerProduct => "inner_product",
    }
}

pub fn smaller_is_closer(metric: Metric) -> bool {
    match metric {
        Metric::InnerProduct => false,
        _ => true,
    }
}
```

```rust
// ✅ Good — adding a metric is one table entry; no existing code changes.
pub struct MetricSpec {
    pub metric: Metric,
    pub sql_name: &'static str,
    pub smaller_is_closer: bool,
    pub distance: fn(&[f32], &[f32]) -> f32,
}

pub const METRICS: &[MetricSpec] = &[
    MetricSpec { metric: Metric::Cosine, sql_name: "cosine_distance", smaller_is_closer: true, distance: cosine },
    MetricSpec { metric: Metric::L2, sql_name: "l2_distance", smaller_is_closer: true, distance: l2 },
];

pub fn spec_for(metric: Metric) -> Option<&'static MetricSpec> {
    METRICS.iter().find(|spec| spec.metric == metric)
}
```

2. **Traits as the cross-package seam**
   DataFusion declares `ScalarUDFImpl`. Each package supplies implementations. The application composes them.

```rust
// ❌ Bad — one shared UDF dispatches on its name; adding a function edits a working `invoke_with_args`.
fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
    match self.name.as_str() {
        "l2_distance" => l2_distance(&args.args),
        "cosine_distance" => cosine_distance(&args.args),
        other => exec_err!("unknown vector function: {other}"),
    }
}
```

```rust
// ✅ Good — each function is its own `ScalarUDFImpl`; adding one is a type and one list entry.
pub fn all_functions() -> Vec<Arc<ScalarUDF>> {
    vec![
        Arc::new(ScalarUDF::new_from_impl(L2Distance::new())),
        Arc::new(ScalarUDF::new_from_impl(CosineDistance::new())),
    ]
}
```

## Why It Matters

Editing working code to add a variant introduces regressions. The existing arms are tested. The arm added to
each of two functions is the one that ships broken. A registry entry cannot break the entries above it. A
trait implementation cannot break the implementation beside it.

The three signals make the decision checkable. A metric registry is open because users keep asking for new
metrics. A `match` over Arrow's fixed set of float `DataType`s in one kernel stays closed, because Arrow pins
the variants.

## Pragmatism Caveat

A `match` over a variant set frozen by an external spec, matched in one or two co-located places, and
internal to one package stays a `match`. An indirection layer there costs more than it saves. An enum with a
`match` also gives exhaustiveness checking; the compiler cannot report a missing table entry.

When a signal fires and the branching stays, a brief comment at the site states why: the spec pins the
variants, or the match sites are three lines apart. An undocumented violation is always wrong.

## Checklist

Before committing code, verify:

- [ ] New functions, metrics, or element types are added as data entries or trait impls, never as new
      branches in working code
- [ ] Per-variant behavior lives in the variant's own entry (a function field or trait impl), never in a
      shared function's `match` arms
- [ ] A package extends the engine by handing it a value implementing DataFusion's traits; nothing shared
      imports the extending package
- [ ] No variant set is matched in more than one module, unless the sites are co-located and the set is
      frozen by an external spec
- [ ] A retained `match` over a spec-frozen variant set is exhaustive and carries a comment saying it is
      closed on purpose

## References

- [principle-single-responsibility](principle-single-responsibility.md) - Related: An extension point works
  only when each variant owns one concern
- [principle-dry-wet](principle-dry-wet.md) - Related: An extension point is the home for shared behavior; a
  flag on a shared helper is not
- [principle-least-surprise](principle-least-surprise.md) - Related: A registry entry satisfies the same
  behavioral contract callers already expect

## External References

- [Understanding the Open/Closed Principle](https://dev.to/dazevedo/understanding-the-openclosed-principle-ocp-from-solid-keep-code-flexible-yet-stable-jo7)
