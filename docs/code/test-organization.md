---
name: "test-organization"
description: "Test tiers for DataFusion functions: unit tests in src and SQL-level tests in crate tests. Load when choosing where a test belongs or adding a test tier"
type: "core"
scope: "project"
---

# Test Organization

Test location states the boundary under test. Unit tests exercise a UDF implementation directly; SQL-level tests exercise the behavior users observe through a DataFusion session. Keep each test at the cheapest boundary that can establish its promise.

## Unit and SQL-Level Tests Have Separate Boundaries

Tests inside a crate's `src/` tree exercise one implementation or helper without constructing a SQL query. Tests under that crate's `tests/` directory register and call functions through `SessionContext`, and pin SQL-visible planning, execution, and result behavior. A test does not move to SQL level merely because its subject is a UDF; use SQL when name resolution, coercion, planning, registration, or query output is part of the promise.

This split keeps implementation cases fast and local while preserving a smaller set of tests for the public path a query takes. A SQL test should assert what a query returns or the error a caller receives, not duplicate every internal branch already covered by unit tests.

Use a direct unit test for details such as argument decoding, row iteration, Arrow builder output, and a helper's null handling. Use a SQL-level test when a result depends on registration, expression construction, planner coercion, literal inspection, or SQL's treatment of the returned array or struct. A public behavior may need both: a unit test localizes the implementation contract, while a focused SQL case confirms that the query path exposes it as intended.

The boundary is about what the assertion depends on, not how short the test is. Creating a `SessionContext` just to call a private helper obscures a unit test. Calling `invoke_with_args` to prove SQL registration or type coercion bypasses the behavior the test claims to cover.

```rust
// Unit boundary: pass a scalar and an array directly to the UDF implementation.
let output = implementation.invoke_with_args(args)?;

// SQL boundary: register the public function and observe a query result.
let batches = context.sql("SELECT jsonschema_valid('{}', '{\"type\": \"object\"}')")
    .await?
    .collect()
    .await?;
```

## Place Each Test Beside Its Boundary

Unit tests live in a `#[cfg(test)]` module in the source file they test, or in a child test module declared by that file when it needs separation. SQL-level tests live in `<crate>/tests/` and use a `SessionContext`. The crate integration-test directory is owned by E2E Test Writer; Code Writer adds unit tests under `src/` and hands SQL cases off for that suite.

Do not add an alternate test tier or a shared test framework for a single function. Use the existing Cargo unit and integration test boundaries. A test that reaches the implementation by calling its `ScalarUDFImpl` directly belongs with the unit tests; one that calls the registered SQL function belongs in the crate integration suite.

When the SQL suite already has a file for the function, add a case there rather than making a second file for one edge case. Keep assertions about user output stable and explicit: compare the value or expected error, and include null rows when null visibility is the behavior under test. Avoid pinning internal helper choices in SQL tests.

## Checklist

- [ ] Unit tests stay in the source tree and exercise implementation behavior directly.
- [ ] SQL-level tests live under the crate's `tests/` directory and use `SessionContext`.
- [ ] SQL tests assert user-visible results, planning behavior, or errors rather than duplicating implementation branches.
- [ ] No new test tier or shared framework is introduced for an isolated function.
