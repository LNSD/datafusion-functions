---
name: "prd-001-jsonschema-validation"
description: "Validate JSON text columns against JSON Schema inside DataFusion SQL queries"
type: "prd"
status: "shipped"
---

# JSON Schema Validation for DataFusion SQL

## Problem

Analytical query engines frequently query semi-structured data where JSON payloads are stored in text columns, such as API response logs, incoming webhooks, agent tool-call arguments, and LLM structured outputs. Users need to verify that these JSON records conform to a declared JSON Schema contract directly within SQL queries without pulling entire datasets out of the database into external application workers.

Existing SQL approaches require manually writing fragile path extractions or brittle regular expressions to verify required fields and types. When records fail, analysts cannot easily determine which specific fields broke schema constraints across millions of rows. Furthermore, downstream queries risk reading unvalidated payloads, propagating bad data into reporting and analytics pipelines.

## Goals

- Enable users to filter, count, and audit stored JSON records against a declared JSON Schema contract directly within DataFusion SQL and DataFrame queries.
- Support evaluating conforming rows efficiently so that validation can run at scan speed in filter predicates.
- Diagnose and report the exact locations and constraint keywords where an invalid JSON document fails schema validation, without exposing payload values.
- Handle both constant literal schemas shared across all rows and per-row schemas stored in adjacent columns.
- Prevent external system access or denial-of-service vulnerabilities originating from untrusted schemas during query execution.

## Non-Goals

- Coercing or casting invalid JSON values into conforming types.
- Modifying documents or injecting schema default values into stored instances.
- Resolving external references (`$ref`) over the network or reading schema files from the local filesystem.
- Performing business-rule validations beyond what the JSON Schema specifications define.

## User Stories

### US-1: Filter Conforming Rows (Priority: P1)

As a data analyst querying event logs, I want to filter and count rows whose JSON payload conforms to a declared contract, so that downstream models process only valid records.

Verifiable independently by running a SQL `WHERE` filter with a literal schema and counting surviving rows.

```gherkin
Scenario: Filter rows matching a schema contract
  Given a table containing JSON text records in a column
  When the user executes a query filtering with a valid JSON Schema
  Then only records conforming to the schema are returned
  And records that do not conform are excluded from the result
```

### US-2: Validate Against Per-Row Schemas (Priority: P1)

As an ML platform engineer auditing agent interactions, I want to validate tool arguments against each tool's distinct input schema stored beside each invocation, so that model performance can be measured across diverse tools in a single query.

Verifiable independently by joining execution traces with tool definitions and evaluating validation row-by-row.

```gherkin
Scenario: Validate records with varying schemas per row
  Given a table of invocations joined with tool definitions providing distinct schemas per row
  When the user checks validation against the schema column
  Then each invocation is validated against its corresponding tool schema
  And valid invocations evaluate to true while invalid ones evaluate to false
```

### US-3: Distinguish Malformed JSON from Invalid Instances (Priority: P2)

As a data quality engineer inspecting incoming data streams, I want to distinguish completely unparseable or corrupt JSON strings from well-formed JSON that simply fails schema constraints, so that pipeline ingest bugs can be triaged separately from contract violations.

Verifiable independently by querying non-JSON strings with an empty schema `{}` that accepts any well-formed JSON.

```gherkin
Scenario: Querying malformed JSON produces null
  Given an unparseable or malformed JSON text instance
  When the user tests the instance against any JSON Schema
  Then the result produces null rather than false
```

### US-4: Diagnose Schema Violations (Priority: P2)

As an application developer debugging client payload rejections, I want to inspect the exact failing paths and schema keywords for non-conforming rows, so that I can report actionable validation errors back to API clients.

Verifiable independently by calling error reporting on an invalid instance and inspecting the resulting error records.

```gherkin
Scenario: Inspect detailed schema failure locations
  Given a well-formed JSON text instance that violates schema constraints
  When the user requests detailed error diagnostics for the instance
  Then the output lists each failing instance path, schema path, and constraint keyword
  And the output does not echo the rejected data values
```

## Requirements

### Functional Requirements

- **FR-001:** THE SYSTEM SHALL evaluate whether a JSON text instance conforms to a JSON Schema contract via a scalar function returning boolean.
- **FR-002:** THE SYSTEM SHALL list validation errors as structured records containing instance path, schema path, and failing keyword via a scalar error diagnostics function.
- **FR-003:** IF an instance argument is null, THEN THE SYSTEM SHALL produce null.
- **FR-004:** IF a schema argument is null, THEN THE SYSTEM SHALL produce null.
- **FR-005:** IF an instance argument is not valid JSON text, THEN THE SYSTEM SHALL produce null.
- **FR-006:** IF an instance argument exceeds 200 levels of nesting, THEN THE SYSTEM SHALL produce null.
- **FR-007:** WHERE a schema is provided as a constant literal, THE SYSTEM SHALL compile the schema during query planning and fail query planning if compilation fails.
- **FR-008:** WHERE a schema is provided as a column expression, THE SYSTEM SHALL compile schemas during query execution and fail query execution if compilation fails.
- **FR-009:** THE SYSTEM SHALL accept instance and schema arguments in `Utf8`, `LargeUtf8`, and `Utf8View` string representations without requiring explicit casts to a common type.
- **FR-010:** THE SYSTEM SHALL support JSON Schema drafts 4, 6, 7, 2019-09, and 2020-12, defaulting to draft 2020-12 when `$schema` is omitted.
- **FR-011:** THE SYSTEM SHALL resolve local schema references (`$ref`) within the schema document and reject any references requiring network or filesystem access.
- **FR-012:** THE SYSTEM SHALL reject schemas containing regular expression patterns with lookaround or backreference constructs.
- **FR-013:** WHERE an instance conforms to the schema, THE SYSTEM SHALL return an empty list of errors and indicate that results are not truncated.
- **FR-014:** WHERE an instance violates a schema, THE SYSTEM SHALL output at most 100 errors per row and set a truncation indicator when additional errors exist.
- **FR-015:** THE SYSTEM SHALL omit rejected instance values from diagnostic error outputs.

### Non-Functional Requirements

- **NFR-001:** THE SYSTEM SHALL evaluate boolean validation without allocating unnecessary intermediate syntax tree data structures for strings in conforming instances.
- **NFR-002:** THE SYSTEM SHALL write diagnostic error records directly into columnar array builders without allocating intermediate collection buffers per error.
- **NFR-003:** THE SYSTEM SHALL cache compiled schemas across rows when evaluating schema column expressions.
- **NFR-004:** THE SYSTEM SHALL guarantee linear-time evaluation of regular expression patterns during schema validation.

## Edge Cases

- Empty schema `{}`: Conforms to any valid JSON instance, while malformed text instances produce null.
- False schema `false`: Rejects every JSON instance while producing an error for any valid JSON input.
- Instances with duplicate keys: Lookups read the last value, while member counting keywords observe all occurrences.
- High-nesting instances: JSON documents exceeding 200 nesting levels produce null rather than crashing the execution process with stack overflows.
- Excessively large error lists: Documents triggering more than 100 errors are capped at 100 and marked with `truncated = true`.

## Success Criteria

- **SC-001:** Filtering 1,000,000 JSON records against a literal schema in a SQL query completes in under 2 seconds on standard hardware.
- **SC-002:** 100% of the JSON Schema specification draft test suite test cases supported by the underlying schema engine pass for supported drafts.
- **SC-003:** No external network socket or filesystem handle is opened during query execution regardless of schema or instance content.
- **SC-004:** Zero payload data leaks appear in diagnostic error output schemas or logs.

## References

- [JSON Schema Specification](https://json-schema.org/specification) - Dependency: Core JSON Schema standard
- [datafusion-functions-json](https://github.com/datafusion-contrib/datafusion-functions-json) - Related: JSON extraction functions in DataFusion
