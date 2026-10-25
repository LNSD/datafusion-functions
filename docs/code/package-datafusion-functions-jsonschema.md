---
name: "package-datafusion-functions-jsonschema"
description: "Responsibilities and exports of the JSON Schema DataFusion functions package. Load when changing its crate layout, public API, or registration"
type: "crate"
scope: "crate:datafusion-functions-jsonschema"
---

# `datafusion-functions-jsonschema`

## Responsibility

This package provides DataFusion scalar functions that validate JSON text against JSON Schema text and
report validation errors. It owns JSON instance parsing, schema compilation and validation, and conversion
of parse, schema, and validation failures into DataFusion results. Schema references are resolved only
within the supplied schema document: the package does not fetch external references from network or disk.

## Layout

The crate root keeps implementation modules private and exposes two facades. `functions` re-exports
DataFrame expression builders; `udfs` re-exports cached scalar UDF handles for individual registration.
`jsonschema_valid` and `jsonschema_errors` implement the user functions. `schema_argument` and
`string_argument` handle argument forms, `compiled_schema` compiles and caches validators, `jiter_json`
parses instances, and `jsonschema_errors` formats validation failures. Keep these concerns behind the
crate root rather than exposing implementation modules.

## Public Surface

Applications can use `functions::jsonschema_valid` and `functions::jsonschema_errors` to build expressions,
or `udfs::jsonschema_valid_udf` and `udfs::jsonschema_errors_udf` to register a function individually.
`register_all(&mut dyn FunctionRegistry) -> Result<()>` registers both functions and propagates registry
errors. The registered names do not collide with DataFusion built-ins. Keep additions to this surface
intentional: implementation and cache types remain private.

## Checklist

- [ ] The package owns JSON Schema scalar-function behavior and does not perform external reference retrieval.
- [ ] Crate-root implementation modules remain private; user entry points are re-exported through `functions` or `udfs`.
- [ ] `register_all` registers each package UDF and propagates failures.
- [ ] New public API has a clear caller need and does not expose implementation or cache details.

## References

- [code](../__meta__/code.md) - Foundation: Rule document format
