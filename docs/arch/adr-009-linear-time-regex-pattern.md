---
name: "adr-009-linear-time-regex-pattern"
description: "Require linear-time regex execution for the pattern keyword and reject lookaround or backreferences. Load when modifying regex compilation, pattern validation, or schema parsing safety"
type: "adr"
status: "accepted"
---

# Linear-Time Regular Expressions for Schema `pattern` Keywords

## Context

The JSON Schema specification uses regular expressions in the `pattern` keyword for strings and `patternProperties` for object property names. Standard regex engines with backtracking or advanced features (such as lookaround assertions and backreferences) are susceptible to catastrophic backtracking (Regular Expression Denial of Service - ReDoS).

In analytical engines executing user-supplied or tenant-provided schemas over large datasets, a catastrophic backtracking regex could freeze query workers, starving engine resources and creating availability vulnerabilities.

## Decision

1. **Linear-time regex engine.** We enforce linear-time regular expression evaluation for the `pattern` and `patternProperties` keywords using Rust's `regex` crate semantics (answers [FR-012](prd-001-jsonschema-validation.md#functional-requirements), [NFR-004](prd-001-jsonschema-validation.md#non-functional-requirements)).
2. **Reject lookaround and backreferences.** Any schema containing regular expressions with lookahead, lookbehind, or backreference constructs is deemed invalid and fails compilation (answers [FR-012](prd-001-jsonschema-validation.md#functional-requirements)).

## Consequences

- Schema pattern evaluation is guaranteed to run in $O(n)$ time proportional to instance string length.
- ReDoS vulnerabilities from malicious or poorly written patterns are completely eliminated.
- Schemas relying on non-linear regex constructs must be simplified or adjusted before use.

## Checklist

- [ ] Regular expression evaluation runs in linear time
- [ ] Patterns with lookahead or lookbehind assertions fail schema compilation
- [ ] Patterns with backreferences fail schema compilation

## References

- [prd-001](prd-001-jsonschema-validation.md) - Source: Product requirements for JSON Schema validation
