---
name: "datafusion-udfs-resource-boundaries"
description: "Resource bounds and deterministic evaluation for DataFusion scalar UDFs. Load when parsing structured input, producing diagnostics, or considering external I/O"
type: "core"
scope: "global"
---

# Bounded and Pure UDF Evaluation

## Rule

A scalar UDF is a function of its arguments. Do not read files, access the network, or consult mutable
external state during planning or evaluation. The same arguments should produce the same result.

Bound work on structured or recursive input using a documented size or depth limit where input shape
could cause excessive processing. When a UDF returns a collection of diagnostics, cap the emitted items
and make truncation visible. Inspect at most one item beyond the cap when only the truncation state is
needed.

~~~rust
let (items, truncated) = take_with_truncation(diagnostics, MAX_DIAGNOSTICS);
~~~

## Why It Matters

Unbounded input can consume excessive CPU or memory, while external I/O makes results depend on the
environment instead of query arguments. Visible diagnostic truncation tells callers when a result is
incomplete.

## Pragmatism Caveat

Choose limits from the function's contract and document any cap callers can observe. Do not add arbitrary
limits where the work and input are already bounded.

## Checklist

- [ ] Evaluation is a pure function of its arguments
- [ ] UDF evaluation performs no network or filesystem access
- [ ] Structured input has a documented bound when needed
- [ ] Diagnostic output has a cap and visible truncation state
- [ ] Truncation detection does not traverse the entire remaining result
