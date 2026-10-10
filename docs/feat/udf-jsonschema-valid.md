---
name: "udf-jsonschema-valid"
description: "The jsonschema_valid scalar UDF, which checks JSON text against a JSON Schema. Load when filtering or counting rows by whether their JSON conforms to a schema"
type: "feature"
status: "development"
components: "crate:datafusion-functions-jsonschema"
crate: "datafusion-functions-jsonschema"
---

# `jsonschema_valid`

## Summary

`jsonschema_valid(instance, schema)` tells whether the JSON text `instance` conforms to the JSON
Schema `schema`: true when it does, false when it does not, and null when it cannot be assessed. Call
it to filter, count, and audit stored JSON against a contract inside a SQL query.

## Signature

```sql
jsonschema_valid(instance, schema) -> Boolean
```

| Argument | Types | Literal or column |
|---|---|---|
| `instance` | `Utf8`, `LargeUtf8`, `Utf8View` | Either |
| `schema` | `Utf8`, `LargeUtf8`, `Utf8View` | Either |

Each argument is read in its own type, without a cast to a common one. The result is nullable.

## Semantics

- The draft is read from the schema's `$schema` keyword: 4, 6, 7, 2019-09, or 2020-12, the default
  when the keyword is absent. Under 2019-09 and 2020-12, `format` (such as `email`) never fails
  validation; under drafts 4, 6, and 7 it is asserted.
- A literal schema is compiled when the query is planned. A schema that is not JSON, is not a valid
  schema for its draft, names an unknown draft in `$schema`, or has a `$ref` that does not resolve
  fails planning.
- A schema column is compiled per row through a process-wide cache that keeps the 64 most recently
  used schemas, keyed by their text. The same faults fail execution with an error naming the row's
  index within its batch.
- `$ref` is never fetched: a reference to a URL or a file is unresolvable, so no query touches the
  network or the filesystem. Local references, such as `#/$defs/item`, resolve.
- A null instance or schema produces null. An instance that is not JSON, including one nested deeper
  than 200 levels, also produces null, never false.
- `pattern` uses a linear-time regular expression engine; a pattern with lookaround or
  backreferences makes the schema invalid.
- Numbers are compared as `f64`. A duplicated object key is looked up by its last value, but keywords
  that count or list members, such as `maxProperties` or `additionalProperties`, see every occurrence.

## Usage

Check each row against a literal schema:

```sql
SELECT id, jsonschema_valid(body, '{"type": "object", "required": ["amount"]}') AS valid
FROM requests;
```

Check each row against the schema stored beside it:

```sql
SELECT c.id, jsonschema_valid(c.arguments, t.input_schema) AS valid
FROM tool_calls c
JOIN tools t ON c.tool_name = t.name;
```

Tell malformed JSON apart from a missing payload. The empty schema `{}` accepts any JSON, so a null
result on a non-null body means the body is not JSON:

```sql
SELECT id
FROM requests
WHERE body IS NOT NULL AND jsonschema_valid(body, '{}') IS NULL;
```

The function is available to SQL once the application registers the package's functions; see
[`datafusion-functions-jsonschema`](datafusion-functions-jsonschema.md).

## Use Cases

### Checking Tool Arguments

Measure, per model, how often a tool is called with arguments that break the tool's declared input
schema: the first signal of how reliably an agent uses its tools.

```sql
SELECT c.model,
       avg(CASE WHEN jsonschema_valid(c.arguments, t.input_schema) THEN 1.0 ELSE 0.0 END) AS valid_rate
FROM tool_calls c
JOIN tools t ON c.tool_name = t.name
GROUP BY c.model;
```

### Gating Structured Model Outputs

Keep only the model outputs that match the requested response schema before downstream code reads
their fields. Conformance makes the fields safe to extract, not their values true.

```sql
SELECT id, output
FROM generations
WHERE jsonschema_valid(output, output_schema);
```

### Replaying a New Contract on Past Data

Before tightening a schema, by adding a required field or narrowing an enum, run the new version
over stored payloads to count what it would have rejected that the current one accepts.

```sql
SELECT count(*) AS newly_rejected
FROM generations
WHERE jsonschema_valid(output, output_schema)
  AND NOT jsonschema_valid(output, '{"type": "object", "required": ["invoice_id", "currency"]}');
```

### Separating Capture Faults From Contract Faults

Null and false are different failures: null means the payload could not be assessed, such as a
truncated capture, and false means it broke the contract. Tracking both rates tells an
instrumentation problem from a model or client problem.

```sql
SELECT date_trunc('day', created_at) AS day,
       avg(CASE WHEN jsonschema_valid(output, output_schema) IS NULL THEN 1.0 ELSE 0.0 END) AS unassessable_rate,
       avg(CASE WHEN jsonschema_valid(output, output_schema) = false THEN 1.0 ELSE 0.0 END) AS invalid_rate
FROM generations
GROUP BY date_trunc('day', created_at)
ORDER BY day;
```

### Auditing API Request Bodies

Validate logged request bodies against the schema each endpoint publishes, joined on route. Grouping
by client version shows which release broke the contract.

```sql
SELECT r.route, r.client_version, count(*) AS rejected
FROM requests r
JOIN routes s ON r.route = s.route
WHERE NOT jsonschema_valid(r.body, s.request_schema)
GROUP BY r.route, r.client_version
ORDER BY rejected DESC;
```

## References

- [datafusion-functions-jsonschema](datafusion-functions-jsonschema.md) - Dependency: registering the package's functions
- [udf-jsonschema-errors](udf-jsonschema-errors.md) - Related: where an instance breaks a schema
- [JSON Schema specification](https://json-schema.org/specification) - Dependency: the standard the function implements
