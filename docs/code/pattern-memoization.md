---
name: "pattern-memoization"
description: "Reuse expensive key-determined values through a bounded shared cache. Load when repeated preparation can reuse the same result"
type: "core"
scope: "global"
---

# Memoization Pattern with a Bounded Cache

## Rule

Memoize an expensive value when its complete inputs form a stable key and callers may request the same
value repeatedly. Store prepared values in a shared cache with a fixed capacity and return clones that
share the expensive state where possible. Prepare cache misses outside the lock so slow work does not
block unrelated lookups. A bounded cache prevents workloads with many distinct keys from retaining every
value indefinitely; eviction may cause later recomputation.

## Examples

```rust
// ❌ Bad — every request repeats preparation for the same key.
fn use_value(key: &str) -> Result<Output> {
    let value = prepare(key)?;
    Ok(apply(value))
}
```

```rust
// ✅ Good — repeated keys reuse prepared state with bounded retention.
fn use_value(key: &str) -> Result<Output> {
    let value = Compiled::cached(key)?;
    Ok(apply(value))
}
```

## Why It Matters

Preparation can cost more than a cache lookup. Reusing prepared values reduces repeated work when callers
reuse the same key. A capacity bound controls memory when callers supply many distinct keys, while shared
ownership lets a caller use a cached value without holding the cache lock during its work.

## Pragmatism Caveat

Do not add a shared cache for cheap computations, values used only once, or results whose correctness
depends on hidden context. Cache keys must include every input that affects preparation. For example,
schema compilation belongs in a bounded cache when its package contract permits reuse by schema text.

## Checklist

- [ ] The cache key contains every input that determines the compiled result.
- [ ] Cache capacity is bounded and eviction can safely trigger recompilation.
- [ ] Expensive compilation does not hold the shared cache lock.
- [ ] Cached results can be used without holding the cache lock.
- [ ] Cache failures are returned through the normal result path, not hidden as a fallback value.

## References

- [package-datafusion-functions-jsonschema](package-datafusion-functions-jsonschema.md) - Related: Schema compilation and cache ownership
