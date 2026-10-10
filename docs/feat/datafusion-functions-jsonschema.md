---
name: "datafusion-functions-jsonschema"
description: "The datafusion-functions-jsonschema package: JSON Schema validation functions for DataFusion SQL. Load when registering the package's functions or choosing which of them to call"
type: "feature"
status: "development"
components: "crate:datafusion-functions-jsonschema"
---

# `datafusion-functions-jsonschema`

## Summary

`datafusion-functions-jsonschema` adds JSON Schema validation to DataFusion SQL: it checks JSON text
stored in a column against a JSON Schema, and reports where a document breaks it: API bodies,
configuration rows, tool arguments, or structured model outputs.

To use these functions, call:

```rust
datafusion_functions_jsonschema::register_all(&mut ctx)?;
```

This registers every function of the package in your `SessionContext`. Each function has its own
document beside this one, `udf-<function>.md`, whose `crate` field names this package.

## Semantics

- Every function takes JSON text as `Utf8`, `LargeUtf8`, or `Utf8View`, with the schema as a literal
  or a column.
- No function fetches a `$ref` from the network or the filesystem; references resolve only inside
  the schema document.
- A null argument, or an instance that is not JSON, produces null, never a negative answer.
- A schema fault fails planning when the schema is a literal, and execution when it is a column.

## Usage

Register the functions on a `SessionContext`. No name or alias collides with a DataFusion built-in,
so registration replaces none.

```rust
use datafusion::execution::context::SessionContext;

let mut ctx = SessionContext::new();
datafusion_functions_jsonschema::register_all(&mut ctx)?;
```

Then call them from SQL:

```sql
SELECT id, jsonschema_valid(body, '{"type": "object", "required": ["amount"]}') AS valid
FROM requests;
```

The DataFrame API builds the same calls with `datafusion_functions_jsonschema::functions`, and
`datafusion_functions_jsonschema::udfs` returns each UDF for registering functions one at a time.

## References

- [JSON Schema specification](https://json-schema.org/specification) - Dependency: the standard the functions implement
- [datafusion-functions-json](https://github.com/datafusion-contrib/datafusion-functions-json) - Related: extracting values from JSON text, such as a nested document to validate with `json_get_json`
