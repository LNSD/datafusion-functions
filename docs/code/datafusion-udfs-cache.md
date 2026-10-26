---
name: "datafusion-udfs-cache"
description: "Bounded caches for repeated DataFusion UDF preparation. Load when expensive per-row configuration may repeat"
type: "core"
scope: "global"
---

# Bounded Caches for Repeated Preparation

## Rule

When a UDF repeatedly prepares configuration supplied per row, reuse successful prepared values through
a shared cache with a fixed capacity and explicit eviction policy. Key entries by every input that
determines the prepared value. Return clones that share expensive state where possible.

Keep expensive preparation outside the cache lock so a slow miss does not block unrelated lookups.
Concurrent duplicate preparation is acceptable only when the result is equivalent. Do not cache failures:
the same invalid input must continue to produce a contextual error at its call site.

~~~rust
// ❌ Bad — repeated keys repeat expensive preparation.
let prepared = prepare(key)?;

// ✅ Good — repeated keys reuse prepared state in a bounded cache.
let prepared = Prepared::cached(key)?;
~~~

## Why It Matters

Repeated configuration values can make preparation a substantial part of row evaluation. Reuse reduces
that work, while a capacity bound prevents high-cardinality inputs from retaining unbounded memory.

## Pragmatism Caveat

A cache is useful only when preparation cost justifies synchronization and retained memory. Do not cache
cheap values, results used once, or values that depend on hidden context or external mutable state.

## Checklist

- [ ] Cache keys include every input that determines the prepared value
- [ ] Cache capacity and eviction behavior are bounded and explicit
- [ ] Expensive preparation runs outside the cache lock
- [ ] Cached values share prepared state where possible
- [ ] Failures are not cached and preserve call-site error context
- [ ] Cached and uncached execution have the same result and error behavior
