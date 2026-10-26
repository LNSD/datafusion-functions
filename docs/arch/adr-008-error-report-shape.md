---
name: "adr-008-error-report-shape"
description: "Structure diagnostic validation errors into path and keyword fields without echoing rejected values. Load when modifying error reporting structures, error limits, or Arrow builder integration"
type: "adr"
status: "accepted"
---

# Diagnostic Error Report Shape and Privacy Guarantees

## Context

When JSON validation fails, downstream applications, debugging pipelines, and SQL queries need to know where and why the document broke schema rules. However, analytical databases often process sensitive data (PII, credentials, proprietary business payloads, unredacted tool arguments). Echoing the rejected instance value back inside query output arrays poses data leakage risks, increases row payload size, and allocates unbounded memory for large failing payloads.

Additionally, non-conforming rows can trigger hundreds or thousands of repetitive validation errors, exhausting memory buffers and slowing down query execution.

## Decision

1. **Error struct fields.** Each validation error record contains three string fields: `instance_path` (JSON Pointer to failing value), `schema_path` (JSON Pointer to violated rule), and `keyword` (name of the violated rule) (answers [FR-002](prd-001-jsonschema-validation.md#functional-requirements)).
2. **Never echo rejected values.** Diagnostic error records never include or echo the rejected instance value, protecting against payload leakage in logs and query results (answers [FR-015](prd-001-jsonschema-validation.md#functional-requirements), [SC-004](prd-001-jsonschema-validation.md#success-criteria)).
3. **Error truncation limit.** At most 100 errors are collected per row. If more than 100 errors occur, the list is capped at 100 and the top-level `truncated` boolean field is set to `true` (answers [FR-014](prd-001-jsonschema-validation.md#functional-requirements)).
4. **Evaluation order.** Errors are emitted in the schema's natural evaluation order without post-hoc sorting.
5. **Direct Arrow builder integration.** Error fields are written directly into Arrow columnar array builders during evaluation without allocating intermediate Rust `String` or vector collections per error (answers [NFR-002](prd-001-jsonschema-validation.md#non-functional-requirements)).

## Consequences

- Analytical queries safely inspect and aggregate errors (`GROUP BY error['keyword']`) without leaking raw values.
- Memory consumption per failing row is bounded by the 100-error cap.
- Writing directly to Arrow builders reduces non-conforming row overhead by 25% to 35%.

## Checklist

- [ ] Error struct defines `instance_path`, `schema_path`, and `keyword`
- [ ] Rejected instance values are never present in error outputs
- [ ] At most 100 errors are returned per row, setting `truncated = true` when exceeded
- [ ] Errors follow natural schema evaluation order
- [ ] Errors are written directly into Arrow builders

## References

- [prd-001](prd-001-jsonschema-validation.md) - Source: Product requirements for JSON Schema validation
