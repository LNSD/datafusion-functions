---
name: "datafusion-udfs-null-errors"
description: "Null and error behavior for DataFusion scalar UDFs. Load when defining or reviewing handling of null, invalid configuration, or unassessable data"
type: "core"
scope: "global"
---

# Nulls and Errors in Scalar UDFs

## Rule

A null input produces a null output. A null result for non-null data that cannot be assessed is allowed
only when the function's documented contract defines it. State this behavior clearly so callers can
distinguish absent input, unassessable data, and invalid configuration. For the error kind and message
format used for invalid configuration, follow
[datafusion-errors-reporting](datafusion-errors-reporting.md).

~~~rust
let Some(text) = values.value(row) else {
    results.append_null();
    continue;
};

// Null is the documented result when this input cannot be assessed.
let Ok(value) = parse_input(text) else {
    results.append_null();
    continue;
};
~~~

~~~rust
// Invalid configuration remains an explicit execution error.
let config = check_config(config)
    .map_err(|error| exec_err!("{function}: row {row}: invalid configuration: {error}"))?;
~~~

## Why It Matters

Null input and data that cannot be assessed under the documented contract produce null. Invalid
configuration is an error; see [datafusion-errors-reporting](datafusion-errors-reporting.md) for its
error kind and message format. This distinction helps callers interpret results and correct configuration.

## Pragmatism Caveat

Document which non-null inputs cannot be assessed and whether they produce null. Do not silently turn
invalid configuration or an undocumented failure into null, false, or an empty result.

## Checklist

- [ ] Null inputs produce null outputs
- [ ] Invalid literal configuration fails during planning
- [ ] Invalid configuration discovered during evaluation returns an execution error
- [ ] A null result for non-null unassessable data is part of the documented function contract
- [ ] No invalid configuration or undocumented failure is translated into null, false, or an empty result

## References

- [datafusion-errors-reporting](datafusion-errors-reporting.md) - Error kind and message format
