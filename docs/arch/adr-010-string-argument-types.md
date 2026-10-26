---
name: "adr-010-string-argument-types"
description: "Support Utf8, LargeUtf8, and Utf8View string representations natively without explicit casting. Load when modifying string argument handling, arrow array extraction, or type coercion"
type: "adr"
status: "accepted"
---

# Native Support for `Utf8`, `LargeUtf8`, and `Utf8View` Arguments

## Context

Apache Arrow and DataFusion support multiple string representations:
- `Utf8`: 32-bit offset string arrays (standard strings up to 2GB per array).
- `LargeUtf8`: 64-bit offset string arrays (for large string collections).
- `Utf8View`: Arrow string view representation with 12-byte inline prefixes and out-of-line buffers, providing fast string scanning and zero-copy slicing.

If a UDF only accepts `Utf8`, DataFusion inserts runtime cast nodes for `LargeUtf8` or `Utf8View` inputs. Casting copies string buffers, doubling memory allocation and degrading scan throughput.

## Decision

1. **Native string argument extraction.** Both `instance` and `schema` arguments to `jsonschema_valid` and `jsonschema_errors` accept `Utf8`, `LargeUtf8`, and `Utf8View` types natively (answers [FR-009](prd-001-jsonschema-validation.md#functional-requirements)).
2. **Zero-copy access without type casts.** String slices are accessed directly from the respective underlying Arrow array or scalar without invoking Arrow type casting kernels.
3. **Flexible combinations.** Any valid combination of string types between instance and schema (e.g. `Utf8View` instance with `Utf8` literal schema) is accepted without type coercion.

## Consequences

- Zero unnecessary string buffer allocations or copies occur when querying string view columns or large string tables.
- Query plans remain clean and free from synthetic `CAST` expression wrappers.
- Maximum interoperability across diverse Arrow table schemas in modern DataFusion engines.

## Checklist

- [ ] `Utf8`, `LargeUtf8`, and `Utf8View` are accepted for instance arguments
- [ ] `Utf8`, `LargeUtf8`, and `Utf8View` are accepted for schema arguments
- [ ] Direct slice access is performed without invoking casting kernels

## References

- [prd-001](prd-001-jsonschema-validation.md) - Source: Product requirements for JSON Schema validation
