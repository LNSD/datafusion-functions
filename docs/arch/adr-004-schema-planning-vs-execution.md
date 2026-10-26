---
name: "adr-004-schema-planning-vs-execution"
description: "Fail query planning for invalid literal schemas and query execution for invalid column schemas. Load when modifying schema validation lifecycle, error dispatching, or planning-time checks"
type: "adr"
status: "accepted"
---

# Schema Failure Reporting: Planning versus Execution

## Context

In DataFusion queries, schema arguments to `jsonschema_valid` and `jsonschema_errors` can be supplied in two ways:
1. As a constant literal string (e.g. `jsonschema_valid(body, '{"type": "object"}')`).
2. As a dynamically evaluated column expression (e.g. `jsonschema_valid(arguments, tool_schema)`).

If a constant literal schema is invalid (not well-formed JSON, invalid schema syntax for its draft, unknown draft `$schema`, or an unresolvable `$ref`), compiling the schema during scan execution would fail mid-scan after processing arbitrary amounts of data, wasting compute resources. Conversely, when a schema is read from a table column, the schema string varies per row and is unavailable at query planning time.

## Decision

1. **Fail literal schemas at planning.** When a schema argument is a constant literal, compile and validate it during DataFusion query planning (`ScalarUDF::simplify` / planning check). If the schema fails to compile, fail planning immediately (answers [FR-007](prd-001-jsonschema-validation.md#functional-requirements)).
2. **Fail column schemas at execution.** When a schema argument is an expression or column reference, compilation occurs during row execution. If a row's schema string fails to compile, fail execution with a DataFusion error naming the exact batch row index where compilation failed (answers [FR-008](prd-001-jsonschema-validation.md#functional-requirements)).

## Consequences

- Queries containing typographical errors or malformed literal schemas are rejected before table scanning starts.
- Execution failures for invalid column schemas report actionable row indices so users can pinpoint the exact invalid row in their dataset.
- Conforming queries with literal schemas incur zero schema parsing overhead during runtime execution batches.

## Checklist

- [ ] Literal schemas are compiled and validated during query planning
- [ ] Invalid literal schemas abort query planning before execution starts
- [ ] Column schemas compile at execution time
- [ ] Execution errors on column schemas include the row index within the record batch

## References

- [prd-001](prd-001-jsonschema-validation.md) - Source: Product requirements for JSON Schema validation
