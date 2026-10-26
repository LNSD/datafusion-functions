---
name: "prd-011-shell-command-analysis"
description: "Scalar functions to parse, classify, and fingerprint shell commands executed in agent traces"
type: "prd"
status: "draft"
---

# Shell Command Analysis Functions

## Problem

Software engineering agents execute composite shell command lines containing pipelines, sequential operators, heredocs, and working directory shifts. Analytical queries over agent execution traces cannot reliably identify executed programs, tool actions, or command patterns using prefix matching or regular expressions. Without dedicated shell command parsing functions, audit and analytics queries miscount program invocations, misclassify operational actions, and fail to cluster recurring command shapes.

## Goals

- Parse shell command lines as written without side effects, filesystem access, or variable expansion.
- Extract individual command invocations with their associated arguments, wrappers, nesting depth, joining operator, and operational category.
- Classify the primary operational intent of composite command lines using a deterministic precedence taxonomy.
- Generate structural command fingerprints that normalize volatile paths and argument values to enable pattern grouping.
- Adhere to query engine null propagation conventions when inputs are null or unparseable.

## Non-Goals

- Expanding runtime variables, aliases, globs, or environmental state, because the analytical target is what was written rather than runtime execution results.
- Executing commands or accessing local or remote filesystems, because analytical evaluation must remain pure and deterministic.
- Parsing non-POSIX or non-Bash syntax dialects such as PowerShell or Windows cmd.exe, because agent traces predominantly target Unix shell environments.
- Unwrapping container and remote execution wrappers such as container runtimes or remote access tools beyond standard shell command wrappers, to avoid brittle argument heuristic parsing.

## User Stories

### US-1: Decompose composite shell lines (P1)

As a trace analyst, I want to extract each invoked command from a multi-command shell line into structured records, so that I can audit and filter every program executed during an agent session.

**Independent test:** Verify that a composite command line returns an ordered list of structured command records identifying the program, arguments, joining operator, and command category for each invocation.

```gherkin
Scenario: Decompose piped and chained commands
  Given a shell command line containing "cd /workspace && cargo test --lib | tee test.log"
  When the line is decomposed into individual commands
  Then three command records are produced in order
  And each command record identifies its program name, arguments, and individual operational category
```

### US-2: Classify composite operational intent (P1)

As a security auditor, I want to evaluate the predominant operational intent of a shell command line, so that I can detect sensitive actions such as file deletions or network publishing even within composite commands.

**Independent test:** Verify that a composite shell command line produces a single operational classification matching the highest-precedence action in the line.

```gherkin
Scenario: Classify composite line by highest precedence
  Given a shell command line containing "git status && rm -rf build"
  When the operational classification of the line is evaluated
  Then the resulting classification is "delete"
```

### US-3: Cluster recurring command patterns (P2)

As a pipeline engineer, I want to compute a structural fingerprint of command lines with literals and paths normalized, so that I can group failing commands by syntactic shape.

**Independent test:** Verify that distinct command lines sharing the same structure and flags but different path arguments produce identical structural fingerprints.

```gherkin
Scenario: Compute structural fingerprint
  Given a command line "cargo test -p my_crate --lib" and a command line "cargo test -p other_crate --lib"
  When their structural fingerprints are computed
  Then both produce the identical fingerprint "cargo test -p ? --lib"
```

## Requirements

- **FR-001:** THE SYSTEM SHALL provide a scalar function that parses a shell command line and returns an array of structured command objects.
- **FR-002:** THE SYSTEM SHALL include the executable program name, argument list, wrapper list, nesting level, joining operator, and individual operational category within each structured command object.
- **FR-003:** THE SYSTEM SHALL unwrap shell invocation wrappers when inspecting command lines.
- **FR-004:** THE SYSTEM SHALL provide a scalar function that returns a single operational category string representing the highest-precedence action identified in a command line.
- **FR-005:** THE SYSTEM SHALL evaluate operational precedence in descending order across categories: delete, publish, edit, install, vcs, network, test, build, run, search, read, list, shell, and unknown.
- **FR-006:** THE SYSTEM SHALL inspect subcommands and recipes for common build tools and task runners to determine their operational category.
- **FR-007:** THE SYSTEM SHALL provide a scalar function that produces a normalized structural fingerprint string for a shell command line.
- **FR-008:** THE SYSTEM SHALL normalize path arguments and literal values to placeholder markers within structural fingerprints while preserving flags and known subcommands.
- **FR-009:** IF the input command line is null, THEN THE SYSTEM SHALL return null for all analysis functions.
- **FR-010:** IF the input command line fails syntactic parsing or contains syntax errors, THEN THE SYSTEM SHALL return null for all analysis functions.
- **NFR-001:** THE SYSTEM SHALL evaluate command analysis functions as pure computations without reading environmental state, filesystem paths, or network endpoints.
- **NFR-002:** THE SYSTEM SHALL parse and analyze shell command lines in linear time relative to input length without exponential backtracking on ambiguous syntax.

## Edge Cases

- Truncated command strings ending with unclosed quotes or dangling pipes produce null due to parse failure.
- Redirections to non-null files evaluate to an edit category regardless of the preceding reading command.
- Empty or whitespace-only command lines produce an empty command array for structural extraction and null for line classification.

## Key Entities

- **Command Line:** The full shell command string as issued to an execution environment.
- **Command Record:** A structured representation of a single invoked program including its arguments, joining operator, nesting depth, and operational category.
- **Operational Category:** A taxonomic classification describing the intent of a command invocation.
- **Structural Fingerprint:** A canonicalized representation of command syntax with literal arguments and paths masked.

## Success Criteria

- **SC-001:** Over 99% of sampled valid shell commands from agent trace datasets successfully parse and classify into their expected operational category.
- **SC-002:** Composite command lines containing multiple chained programs accurately identify all constituent programs in query output.
- **SC-003:** Queries aggregating by structural fingerprint collapse identical command shapes across varying file arguments into single summary rows.

## Assumptions

- Captured command lines conform to Unix shell and Bash syntax conventions.
- Analytical queries run within standard query engine execution pipelines where null propagation adheres to SQL three-valued logic.

## References

- [prd-001](prd-001-jsonschema-validation.md) - Related: Precedent for scalar function design and null semantics
