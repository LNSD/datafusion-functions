---
name: "adr-005-schema-compilation-cache"
description: "Lend literal schemas across rows and cache column schemas in an LRU cache. Load when modifying schema caching, memory bounds, or concurrency in schema evaluation"
type: "adr"
status: "accepted"
---

# Schema Compilation and Caching

## Context

Compiling a JSON Schema involves parsing JSON text into abstract schema structures, resolving local `$defs` / definitions, and constructing validation keyword validators. This compilation process is substantially more expensive than validating an individual instance.

In typical SQL queries, two patterns emerge:
- A literal schema is shared across every row in the record batch. Profiling showed that cloning an `Arc` reference for every row introduced atomic increment and decrement overhead across millions of rows.
- A schema column typically draws from a modest set of recurring schemas across rows (e.g. dozens of distinct tool definition schemas or API contracts evaluated across millions of invocations). Compiling a schema per row without caching would devastate query throughput.

## Decision

1. **Lend literal schemas without reference count churning.** When evaluating a constant literal schema across a record batch, borrow and lend a single compiled schema reference to the inner row validation loop instead of acquiring an atomic reference per row (answers [NFR-003](prd-001-jsonschema-validation.md#non-functional-requirements)).
2. **Process-wide LRU cache for column schemas.** Column-evaluated schemas resolve through a process-wide LRU cache that holds the 64 most recently used compiled schemas, keyed by their schema text (answers [NFR-003](prd-001-jsonschema-validation.md#non-functional-requirements)).
3. **Synchronized cache access.** Access to the 64-entry schema cache is guarded by a mutex to ensure safe concurrent thread access across DataFusion execution threads.

## Consequences

- Literal schemas achieve maximum validation throughput without atomic increment/decrement penalties.
- High-volume queries evaluating column schemas over recurring contracts avoid repeated recompilation overhead.
- Memory consumption remains strictly bounded by the 64-schema LRU capacity.

## Checklist

- [ ] Literal schemas are borrowed and lent across row loops without per-row `Arc` cloning
- [ ] Column schemas use a process-wide LRU cache with capacity 64
- [ ] Cache keys use raw schema text strings
- [ ] Cache access is thread-safe across concurrent query workers

## References

- [prd-001](prd-001-jsonschema-validation.md) - Source: Product requirements for JSON Schema validation
