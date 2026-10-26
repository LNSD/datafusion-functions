---
name: "adr-002-jsonschema-library"
description: "Use the jsonschema crate as the validation engine and support drafts 4 through 2020-12. Load when modifying schema compilation, supported JSON Schema specifications, or validation execution"
type: "adr"
status: "accepted"
---

# Use the `jsonschema` Crate as the JSON Schema Engine

## Context

DataFusion SQL queries require a fast, conformant JSON Schema validation engine to execute `jsonschema_valid` and `jsonschema_errors`. The engine must support widely adopted JSON Schema drafts (Draft 4, Draft 6, Draft 7, Draft 2019-09, and Draft 2020-12) used across external schemas, OpenAPI contracts, and tool definitions.

The evaluation engine must allow compiling schemas ahead of time, validating borrowed or pre-parsed instance structures, and inspecting structured failure diagnostics without pulling external network dependencies.

## Decision

1. **Use `jsonschema` crate.** We select the `jsonschema` crate as the core JSON Schema compiler and validation evaluator for `datafusion-functions-jsonschema`.
2. **Supported drafts.** The engine supports Draft 4, Draft 6, Draft 7, Draft 2019-09, and Draft 2020-12 (answers [FR-010](prd-001-jsonschema-validation.md#functional-requirements)).
3. **Default draft.** When the `$schema` keyword is absent from a schema document, validation defaults to Draft 2020-12 (answers [FR-010](prd-001-jsonschema-validation.md#functional-requirements)).
4. **Format assertion behavior.** In Draft 2019-09 and Draft 2020-12, the `format` keyword acts as an annotation by default and does not fail validation. In Draft 4, Draft 6, and Draft 7, `format` assertions are enforced during validation.

## Consequences

- The package relies on the `jsonschema` crate's implementation of JSON Schema specifications and draft keyword semantics.
- Schemas declaring `$schema` with an unrecognized or unsupported specification URI fail compilation immediately.
- Modern Draft 2020-12 semantics apply to common schemas that omit `$schema`, aligning with modern JSON Schema expectations.

## Checklist

- [ ] The `jsonschema` crate compiles all schema documents
- [ ] Drafts 4, 6, 7, 2019-09, and 2020-12 are supported via `$schema`
- [ ] Schemas omitting `$schema` default to Draft 2020-12 evaluation
- [ ] Format assertions are enforced for Draft 4, 6, and 7, and treated as annotations for Draft 2019-09 and 2020-12

## References

- [prd-001](prd-001-jsonschema-validation.md) - Source: Product requirements for JSON Schema validation
