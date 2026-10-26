---
name: "logging"
description: "Logging boundaries for UDF packages: keep planning and evaluation silent; registration may use debug logging. Load when adding logging, tracing, or printing to a UDF package"
type: "core"
scope: "project"
---

# Logging

UDF planning and evaluation do not emit log records. They run as part of a query, potentially once
per batch or row, and the package does not own the caller's logging policy. Return failures through
DataFusion errors ([datafusion-errors-reporting](datafusion-errors-reporting.md)) instead of logging them.

## 1. Keep Planning and Evaluation Silent

Planning hooks, `invoke_with_args`, and their helpers do not use `log`, `tracing`, `println!`, or
`eprintln!`. A row-level message scales with input size and can expose data. Success is silent; a
failure is returned to the query engine.

```rust
// ❌ Bad — row evaluation emits a record for each input.
log::debug!("checking row {row}");
```

```rust
// ✅ Good — return a failure to the query engine.
let schema = CompiledSchema::cached(text)?;
```

## 2. Registration May Log at Debug

Registration may emit `log::debug!` records containing function names. Registration is a setup action,
not row evaluation. Use the `log` facade and keep this output at debug level.

```rust
// ✅ Good — registration identifies the function without logging input data.
log::debug!("registering UDF {}", udf.name());
```

## Checklist

- [ ] Planning and evaluation paths do not log or print
- [ ] Failures are returned rather than logged
- [ ] Registration logs, if added, use `log::debug!` and contain no row data

## References

- [datafusion-errors-reporting](datafusion-errors-reporting.md) - Related: Return failures through DataFusion instead of logging them
