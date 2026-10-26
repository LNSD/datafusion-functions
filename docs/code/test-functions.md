---
name: "test-functions"
description: "Direct UDF unit tests and their scalar-versus-array argument matrix. Load when testing or changing a scalar function implementation"
type: "core"
scope: "project"
---

# UDF Function Tests

Direct unit tests pin the contract of a UDF implementation at its invocation boundary. They call `invoke_with_args` with explicit DataFusion arguments and examine the returned `ColumnarValue` or `DataFusionError`; SQL behavior belongs to [test-organization](test-organization.md).

## Exercise the Scalar and Array Forms

For each argument, cover its scalar and array forms where the signature accepts both. A scalar is broadcast across the batch; an array supplies one value per row. Include the combinations that expose interactions between arguments, not only the all-scalar and all-array cases. For two arguments, the useful matrix is scalar/scalar, scalar/array, array/scalar, and array/array. Include null values in both forms when null handling is part of the contract.

This matrix catches shape errors that a single representation hides: reading a scalar at the current row, mismatching array lengths, or failing to broadcast a scalar beside a column. Do not test unsupported types by bypassing the function signature unless the invocation helper itself owns that defensive behavior.

For a two-argument function whose signature accepts scalar and array values, think of the cases explicitly:

| First argument | Second argument | What the case exercises |
|---|---|---|
| Scalar | Scalar | Both inputs are constant for the batch |
| Scalar | Array | The first input broadcasts while the second varies by row |
| Array | Scalar | The second input broadcasts while the first varies by row |
| Array | Array | Both inputs advance together row by row |

Choose distinct values across rows so a mistaken scalar/row index is observable. If the function accepts only a subset of these shapes, document that in its signature and cover only accepted shapes. For three or more arguments, cover each argument's scalar and array access path and the mixed combinations that exercise distinct code paths; avoid an exponential set of permutations when combinations are behaviorally equivalent.

## Invoke the Implementation Directly

Build `ScalarFunctionArgs` with the argument values, row count, and the other required DataFusion context, then call `invoke_with_args`. Assert the output type and each row's value or null status. For an execution failure, assert the `DataFusionError` variant and the useful message portion. Keep each test focused on one behavior such as broadcast, null propagation, or row-specific failure.

Use the smallest arrays and scalars that show the case. Do not involve `SessionContext` here: query parsing, function registration, and SQL coercion are the SQL-level suite's responsibilities.

Keep the constructed values and row count visible at the call site. A test helper may reduce repetitive DataFusion boilerplate, but its arguments should still reveal which values are scalar, which are arrays, and how many rows are expected. Do not hide the case in a broad fixture or a table-driven loop that makes a failing combination hard to identify.

When a function returns an array for a scalar invocation, assert the defined shape rather than assuming DataFusion will wrap or unwrap it. Read the function's `invoke_with_args` contract and pin the observed `ColumnarValue` variant and row values. For errors, distinguish planning-time validation from execution-time failures in SQL-level coverage; a direct invocation can only establish the execution path it calls.

## Checklist

- [ ] Direct implementation tests call `invoke_with_args` with explicit `ScalarFunctionArgs`.
- [ ] The accepted scalar and array forms are covered, including mixed forms for multi-argument UDFs.
- [ ] Null behavior is checked in scalar and array form when the function defines it.
- [ ] Assertions inspect result type, row values, and nulls, or the relevant error variant and message.
- [ ] Query parsing, registration, coercion, and SQL-visible behavior remain in SQL-level tests.
