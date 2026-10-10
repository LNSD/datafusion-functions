---
name: "udf-jsonschema-errors"
description: "The jsonschema_errors scalar UDF, which lists where JSON text breaks a JSON Schema. Load when diagnosing which fields and keywords fail validation"
type: "feature"
status: "development"
components: "crate:datafusion-functions-jsonschema"
crate: "datafusion-functions-jsonschema"
---

# `jsonschema_errors`

## Summary

`jsonschema_errors(instance, schema)` names each place the JSON text `instance` breaks the JSON
Schema `schema`. Where [`jsonschema_valid`](udf-jsonschema-valid.md) answers whether an instance
conforms, this function answers where it does not, as rows SQL can group, count, and report.

## Signature

```sql
jsonschema_errors(instance, schema) -> Struct<
  errors List<Struct<instance_path Utf8, schema_path Utf8, keyword Utf8>>,
  truncated Boolean
>
```

| Argument | Types | Literal or column |
|---|---|---|
| `instance` | `Utf8`, `LargeUtf8`, `Utf8View` | Either |
| `schema` | `Utf8`, `LargeUtf8`, `Utf8View` | Either |

Each argument is read in its own type, without a cast to a common one. The struct is nullable; its
fields, the list, and the list's entries are not.

## Semantics

- `instance_path` is a JSON Pointer to the failing value, such as `/items/2/price`; `schema_path` a
  JSON Pointer to the rule it broke, such as `/properties/price/minimum`; `keyword` that rule's name,
  such as `required` or `type`.
- Drafts, schema compilation, planning and execution errors, `$ref` handling, and
  nulls follow [`jsonschema_valid`](udf-jsonschema-valid.md): a null instance, a
  null schema, or an instance that is not JSON produces a null row, never an empty list.
- A conforming instance produces an empty `errors` list and `truncated` false.
- Errors never echo the rejected value, so the output cannot leak payload contents. Paths do contain
  the instance's object keys and array indices.
- At most 100 errors are listed per row; `truncated` is true when more exist.
- Errors follow the schema's evaluation order and are not sorted.
- Struct field access such as `['errors']` needs DataFusion's nested expression functions, which its
  default features include.

## Usage

List the errors of each row:

```sql
SELECT id, jsonschema_errors(body, '{"required": ["currency"]}')['errors'] AS errors
FROM requests;
```

Expand them to one row per error:

```sql
SELECT id, unnest(jsonschema_errors(body, schema)['errors']) AS error
FROM requests;
```

Find rows whose error list was cut short:

```sql
SELECT id
FROM requests
WHERE jsonschema_errors(body, schema)['truncated'];
```

## Use Cases

### Ranking Models by How Their Outputs Fail

Group structured-output failures by model and keyword. One model drops required fields, another
sends a string where a number belongs; each needs a different prompt fix.

```sql
SELECT e.model, e.error['keyword'] AS keyword, count(*) AS failures
FROM (
  SELECT model, unnest(jsonschema_errors(output, output_schema)['errors']) AS error
  FROM generations
) e
GROUP BY e.model, e.error['keyword']
ORDER BY failures DESC;
```

### Explaining Rejected Tool Arguments

Find the argument a model most often gets wrong for each tool, to know which parameter to describe
better in the tool's definition.

```sql
SELECT e.tool_name, e.error['instance_path'] AS path, count(*) AS failures
FROM (
  SELECT c.tool_name, unnest(jsonschema_errors(c.arguments, t.input_schema)['errors']) AS error
  FROM tool_calls c
  JOIN tools t ON c.tool_name = t.name
) e
GROUP BY e.tool_name, e.error['instance_path']
ORDER BY failures DESC;
```

### Checking Whether Retries Fix the Reported Field

Join a rejected tool call to its retry and compare their first failing paths. A null retry path means
the retry conformed; the same path means the validation feedback did not land.

```sql
SELECT o.id,
       jsonschema_errors(o.arguments, t.input_schema)['errors'][1]['instance_path'] AS first_failure,
       jsonschema_errors(r.arguments, t.input_schema)['errors'][1]['instance_path'] AS retry_failure
FROM tool_calls o
JOIN tool_calls r ON r.retry_of = o.id
JOIN tools t ON o.tool_name = t.name
WHERE NOT jsonschema_valid(o.arguments, t.input_schema);
```

### Finding Schemas That Are Too Strict

A rule that fails across every model points at the schema, such as an over-tight constraint or an
enum missing a value, rather than at the models.

```sql
SELECT e.error['schema_path'] AS rule, count(DISTINCT e.model) AS models, count(*) AS failures
FROM (
  SELECT model, unnest(jsonschema_errors(output, output_schema)['errors']) AS error
  FROM generations
) e
GROUP BY e.error['schema_path']
ORDER BY models DESC, failures DESC;
```

### Reporting Invalid Settings Without Exposing Secrets

Show operators which settings fail validation and why. Errors carry paths and keywords, never values,
so secrets stay out of the report.

```sql
SELECT key, jsonschema_errors(value, schema)['errors'] AS errors
FROM settings
WHERE NOT jsonschema_valid(value, schema);
```

## References

- [datafusion-functions-jsonschema](datafusion-functions-jsonschema.md) - Dependency: registering the package's functions
- [udf-jsonschema-valid](udf-jsonschema-valid.md) - Related: whether an instance conforms, and the shared schema rules
- [JSON Pointer, RFC 6901](https://www.rfc-editor.org/rfc/rfc6901) - Dependency: the path format
- [JSON Schema specification](https://json-schema.org/specification) - Dependency: the standard the function implements
