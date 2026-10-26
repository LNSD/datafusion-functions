---
name: "datafusion-errors-reporting"
description: "DataFusion error types, planning and execution failures, and returned error context. Load when returning or reporting an error from a UDF"
type: "core"
scope: "global"
---

# DataFusion UDF Errors

UDF failures cross the DataFusion boundary as `DataFusionError`, classified by when they become known
and reported with enough context to locate the failure. Logging boundaries are owned by
[logging](logging.md).

## 1. Return DataFusion Errors

UDF-facing fallible functions return `datafusion::common::Result` and therefore `DataFusionError`.
Use a local private error type only when code needs to distinguish dependency failures before they reach
that boundary; convert it there. This keeps package details internal while allowing DataFusion to report
the failure to the query caller.

```rust
// ✅ Good — a private preparation failure is translated at the query boundary.
match prepare(input) {
    Ok(value) => Ok(value),
    Err(error) => plan_err!("{function}: {error}"),
}
```

## 2. Classify Failures by When They Are Known

A failure determined from query literals is a planning error, reported with `plan_err!` from a planning
hook. A failure discovered while evaluating arguments or rows is an execution error, reported with
`exec_err!` from the execution path. A non-literal input cannot be validated at planning time, so the
execution path handles it. Reserve `internal_err!` for states the implementation makes unreachable.

This distinction lets invalid constant configuration fail before rows run and gives data-dependent
failures the appropriate execution context.

```rust
// ✅ Good — the failure is discovered while evaluating this row.
Err(error) => exec_err!("{function}: row {row}: {error}"),
```

## 3. Include Available Context

An error message gives enough context to locate the failure and describes its cause. Include the UDF
name when the failure site knows it and the row index for row-specific failures. Shared conversion
helpers may not know which UDF called them; do not invent a function name in that case.

Context lets callers identify the failing operation and input position without needing an implementation
backtrace.

```rust
// ✅ Good — reports function, row and cause.
exec_err!("{function}: row {row}: value is not a string")
```

## 4. Do Not Include Row Values

Returned errors should describe invalid input without copying the row's contents. Row values can be
sensitive, and the error may be surfaced or recorded beyond the code that produced it. Include a row
index and a concise cause instead.

```rust
// ❌ Bad — includes the rejected cell in a returned error.
exec_err!("invalid value {value}")
```

```rust
// ✅ Good — reports location and cause without copying the value.
exec_err!("{function}: row {row}: value is not a string")
```

## Checklist

Before committing code, verify:

- [ ] UDF-facing fallible functions return `datafusion::common::Result`
- [ ] Any private helper error is converted at the first DataFusion `Result` boundary
- [ ] Query-literal failures use `plan_err!` in a planning hook
- [ ] Failures discovered during argument or row evaluation use `exec_err!`
- [ ] `internal_err!` is reserved for impossible internal states
- [ ] Include the UDF name when the failure site knows it
- [ ] Include the row index for a row-specific failure
- [ ] Describe the cause clearly
- [ ] Do not copy row values into returned errors

## References

- [principle-validate-at-edge](principle-validate-at-edge.md) - Foundation: Validate literals during planning
- [rust-fn-parse](rust-fn-parse.md) - Related: Parsing and fallible conversions
