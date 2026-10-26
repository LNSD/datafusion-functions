---
name: "adr-006-instance-parsing-jiter"
description: "Parse JSON instances using jiter rather than serde_json. Load when modifying instance parsing, JSON value representation, or parsing benchmarks"
type: "adr"
status: "accepted"
---

# Parse JSON Instances Using `jiter` Instead of `serde_json`

## Context

In JSON Schema evaluation over large database columns, the majority of CPU time on conforming rows was spent constructing and deallocating fully owned `serde_json::Value` syntax trees rather than performing schema validation checks. Conforming records require string slice validation but do not need long-lived owned heap allocations for every JSON string and key.

We benchmarked replacing `serde_json::Value` instance parsing with `jiter`, an iterator-based JSON parser that borrows string slices directly from input byte slices.

## Decision

1. **Use `jiter` for instance parsing.** We parse JSON instance texts into borrowed `jiter` value representations (`JiterValue`) rather than `serde_json::Value` (answers [NFR-001](prd-001-jsonschema-validation.md#non-functional-requirements)).
2. **Compile schemas from `serde_json`.** Schemas continue to be parsed and compiled via `serde_json`, which `jsonschema` expects and which is compiled only once or cached.
3. **Number comparison.** Numbers are compared as `f64` during schema validation.
4. **Duplicate object keys.** Duplicate object keys are preserved in order: key lookup reads the last occurrence, whereas keywords that count or list members (such as `maxProperties` and `additionalProperties`) observe all occurrences.
5. **Nesting limit.** Instance nesting depth is capped at 200 levels (extending `jiter`'s default 128 limit). Nesting exceeding 200 levels yields null (answers [FR-006](prd-001-jsonschema-validation.md#functional-requirements)).

## Consequences

- Conforming rows evaluate 25% to 30% faster for medium and complex schemas because string allocations are eliminated.
- On non-conforming rows, error reporting allocates rejected values on demand, which is acceptable because non-conforming rows are the exceptional path.
- Numbers compare with floating-point semantics.

## Checklist

- [ ] Instances are parsed via `jiter` without heap-allocating borrowed strings
- [ ] Schemas compile from `serde_json`
- [ ] Number comparisons evaluate as `f64`
- [ ] Duplicate object keys are observed by member-counting keywords while lookups take the last key
- [ ] Instances deeper than 200 nesting levels produce null

## References

- [prd-001](prd-001-jsonschema-validation.md) - Source: Product requirements for JSON Schema validation
