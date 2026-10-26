---
name: "datafusion-udfs-planning"
description: "Planning-time validation of DataFusion scalar UDF literals. Load when validating literal arguments or deriving a result field from them"
type: "core"
scope: "global"
---

# Planning-Time Literal Validation

## Rule

Validate a scalar literal during planning when it determines whether a UDF call is valid. Use
`return_field_from_args` to inspect literal arguments and validate them during planning. Follow
[datafusion-errors-reporting](datafusion-errors-reporting.md) for the error kind and message format.
This gives callers a prompt error before execution processes rows.

Keep `return_type` as the general output type contract. Use `return_field_from_args` when literal
arguments affect field metadata or need validation. Declare nullable output when null inputs can produce
null.
Column values are unknown during planning and must remain validated during execution with row context.

If execution also needs a value prepared from a literal, pass it through supported execution arguments
or retrieve it from a bounded cache keyed by all inputs that determine its meaning. Do not assume the
planning hook retains arbitrary prepared values for execution.

~~~rust
fn validate_literal(value: &ScalarValue) -> Result<()> {
    if is_invalid(value) {
        return plan_err!("invalid constant argument");
    }
    Ok(())
}
~~~

## Why It Matters

Planning-time validation gives stable errors for query constants before a query starts reading data. It
also keeps planning separate from row work: a column value is not known until execution.

## Pragmatism Caveat

Validate only facts available to the planner. A column expression or null literal is not a valid
non-null configuration merely because the planning hook cannot inspect a runtime value.

## Checklist

- [ ] Literal-dependent validity is checked during planning
- [ ] Planning failures use `plan_err!`
- [ ] `return_type` remains the general type contract
- [ ] Output nullability matches the UDF's null behavior
- [ ] Column values remain validated during execution with row context

## References

- [datafusion-errors-reporting](datafusion-errors-reporting.md) - Error kind and message format
