---
name: "test-files"
description: "Rust test placement and file organization for DataFusion function crates. Load when adding or moving a unit test module or SQL integration test"
type: "core"
scope: "project"
---

# Test File Organization

Keep unit tests close to the Rust module they exercise, and keep tests that drive DataFusion through SQL in the crate integration-test directory. The file boundary should make it clear which contract the test protects without creating a parallel test layout.

## Keep Unit Tests in the Source Tree

Put unit tests in a `#[cfg(test)] mod tests` in the source module under test. When a test module grows enough to merit a separate file, declare a child test module from that source file and keep the file under the module's directory. This preserves access to private implementation details while making the subject clear.

Avoid one central `src/tests.rs` for unrelated modules: tests should move with the code and sit near the behavior whose change should update them. Do not place unit tests under `<crate>/tests/`, where Cargo builds each file as an external integration-test crate and private implementation is unavailable.

A module-local test block can import its parent with `use super::*;` when that makes the subject clear and avoids needless visibility changes. A child test file is useful when the test body is substantial, but the production module still declares it under `#[cfg(test)]`; the file is not a free-floating sibling that Cargo discovers independently.

Keep source fixtures small and local to their test module unless several test modules share a genuine contract. Do not add a helper module solely to wrap one assertion or one array construction. Tests should remain readable from the module declaration to the assertion that pins the behavior.

## Keep SQL Tests in the Crate Integration Directory

Tests that construct a `SessionContext`, register a function, and run SQL belong under `<crate>/tests/`. Use a file named for the behavior or function being exercised. This boundary validates the public query path while leaving implementation-level cases beside their source.

Files in `<crate>/tests/` are owned by the E2E Test Writer. A code change that needs a SQL-level regression case records the SQL call, input case, and expected result or error in the E2E handoff rather than editing those files.

An integration-test file may use only the crate's public surface, as a downstream caller would. That restriction is useful: an import that requires `pub(crate)` or a path into `src/` is a sign the test belongs in the unit-test tree. Keep one file focused on a cohesive function or behavior, and avoid duplicating private setup already covered by unit tests.

## Checklist

- [ ] Unit tests are declared beside the module they exercise under `src/`.
- [ ] A separate unit-test file is a child of its subject module, not a central unrelated test file.
- [ ] Tests using `SessionContext` and SQL live under `<crate>/tests/`.
- [ ] No Code Writer change edits crate integration-test files; SQL cases are handed off to E2E Test Writer.
