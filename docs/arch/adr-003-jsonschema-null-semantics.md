---
name: "adr-003-jsonschema-null-semantics"
description: "Return null when an input cannot be assessed rather than false or empty list. Load when implementing or modifying null handling, error reporting, or input validation semantics"
type: "adr"
status: "accepted"
---

# Null Semantics for JSON Schema Functions

## Context

In SQL three-valued logic, boolean expressions distinguish `TRUE`, `FALSE`, and `NULL` (unknown / unassessable). For validation functions like `jsonschema_valid`, returning `false` indicates that a document was parsed, assessed against the schema contract, and found non-conforming.

If an instance is `NULL`, or if it is unparseable malformed text rather than valid JSON, or if it exceeds safe recursion depth limits, the input cannot be assessed against schema rules. Treating unparseable text as `false` would conflate "valid JSON that violates schema constraints" with "malformed string that is not JSON at all", preventing analysts from distinguishing contract breaches from pipeline serialization corruption. Similarly, for diagnostic functions like `jsonschema_errors`, returning an empty list would signify a valid conforming document, which is misleading when the document could not be evaluated.

## Decision

1. **Null inputs yield null.** If the instance argument or schema argument is null, the result is null for both `jsonschema_valid` and `jsonschema_errors` (answers [FR-003](prd-001-jsonschema-validation.md#functional-requirements), [FR-004](prd-001-jsonschema-validation.md#functional-requirements)).
2. **Malformed instances yield null.** When the instance argument is not valid JSON text, `jsonschema_valid` produces null, never false (answers [FR-005](prd-001-jsonschema-validation.md#functional-requirements)).
3. **Nesting limit yields null.** When an instance exceeds 200 levels of JSON nesting, evaluation halts safely and produces null (answers [FR-006](prd-001-jsonschema-validation.md#functional-requirements)).
4. **Diagnostic null row.** When an instance cannot be assessed (null input, malformed JSON, or nesting limit exceeded), `jsonschema_errors` returns a null row rather than an empty list or struct with empty fields (answers [FR-002](prd-001-jsonschema-validation.md#functional-requirements), [FR-005](prd-001-jsonschema-validation.md#functional-requirements)).
5. **Conforming documents.** A conforming instance evaluated by `jsonschema_errors` produces a non-null struct containing an empty `errors` list and `truncated = false` (answers [FR-013](prd-001-jsonschema-validation.md#functional-requirements)).

## Consequences

- Callers can reliably identify unparseable or malformed rows using `WHERE body IS NOT NULL AND jsonschema_valid(body, '{}') IS NULL`.
- Contract violations remain distinct from syntax and parsing errors.
- Downstream queries unnesting `jsonschema_errors` naturally ignore unassessable rows when filtering on non-null error structs.

## Checklist

- [ ] A null instance or schema returns null in both functions
- [ ] Non-JSON instance text produces null rather than false in `jsonschema_valid`
- [ ] Instances exceeding 200 nesting levels produce null
- [ ] Unassessable rows return a null struct in `jsonschema_errors`
- [ ] Conforming documents produce an empty list in `jsonschema_errors` with `truncated = false`

## References

- [prd-001](prd-001-jsonschema-validation.md) - Source: Product requirements for JSON Schema validation
