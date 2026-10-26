---
name: "prd-001-shell-udfs"
description: "Scalar SQL functions parsing agent shell commands into structured commands, kinds, and fingerprints"
type: "prd"
status: "draft"
---

# Shell Command UDFs for Agent Traces

## Problem

An agent's shell command is an executable program. In trace datasets of tool invocations, command lines
frequently contain operators such as pipes, conjunctions, semicolons, command substitutions, or heredocs,
and many open by navigating directories. Plain text matching and regular expressions cannot accurately inspect
which programs ran, distinguish subcommands from arguments or flags, or identify common failure patterns.

SQL analysts and audit pipelines need scalar functions that parse command lines directly inside queries,
without executing commands or touching the environment.

## Goals

- Parse shell command lines into structured commands, arguments, and chaining operators.
- Classify lines into operational categories such as read, edit, test, build, or delete using standard precedence.
- Generate normalized command fingerprints with literal arguments abstracted to identify recurring execution patterns.

## Non-Goals

- Command execution: parsing only; commands are never run, evaluated, or expanded.
- Environment awareness: no filesystem, variable, or network resolution.
- Multi-dialect parsing: standard shell syntax (Bash and POSIX sh) only; custom language runtimes are excluded.

## User Stories

### US-1: Extract executed programs and arguments (P1)

As an audit analyst, I want to extract programs and arguments from shell commands, so that I can report which
tools agents run.

**Independent test:** Parse a pipeline command string and verify extracted program records.

```gherkin
Scenario: Extract commands from pipeline
  Given a shell command string "cat data.csv | grep error"
  When parsed by the command extraction function
  Then two command items "cat" and "grep" are returned with their respective arguments
```

### US-2: Classify operational kind of command lines (P1)

As a security reviewer, I want each command line classified by its highest-precedence operation, so that I can
flag potentially destructive actions like file deletion or publishing.

**Independent test:** Classify command lines containing chained operations against the precedence order.

```gherkin
Scenario: High-precedence operation takes priority
  Given a shell command string "git status && rm -rf build"
  When classified by the command kind function
  Then the reported kind is "delete"
```

### US-3: Group similar commands by fingerprint (P2)

As a pipeline engineer, I want commands normalized into shape fingerprints with parameters abstracted, so that I
can aggregate failures across dynamic paths and parameters.

**Independent test:** Generate identical fingerprints from commands varying only in arguments.

```gherkin
Scenario: Normalize command shape
  Given a command "cargo test -p crate_a" and a command "cargo test -p crate_b"
  When fingerprinted by the shape normalization function
  Then both produce the fingerprint "cargo test -p ?"
```

## Requirements

- **FR-001:** WHEN a command line is provided, THE SYSTEM SHALL parse it into an array of commands preserving
  order, arguments, and chaining operators.
- **FR-002:** WHEN a command line is provided, THE SYSTEM SHALL return an array of distinct program names
  executed by the line.
- **FR-003:** WHEN a command line is provided, THE SYSTEM SHALL classify the line into exactly one operational
  kind according to fixed precedence.
- **FR-004:** WHEN a command line is provided, THE SYSTEM SHALL compute a shape fingerprint replacing paths
  and variable arguments with placeholder markers.
- **FR-005:** IF a command line is null, THEN THE SYSTEM SHALL return null.
- **FR-006:** IF a command line fails to parse, THEN THE SYSTEM SHALL return null [NEEDS CLARIFICATION: should unparseable lines return null or a best-effort fallback?].
- **FR-007:** WHERE a command contains task runner invocations, THE SYSTEM SHALL classify them according to the
  recipe or subcommand [NEEDS CLARIFICATION: should task runners like make or npm be classified by recipe or remain run?].
- **NFR-001:** THE SYSTEM SHALL evaluate scalar expressions synchronously without filesystem or network access.
- **NFR-002:** THE SYSTEM SHALL support standard string data types without requiring explicit type casts.

## Edge Cases

- What happens when a command string is truncated midway through a token?
- What happens when nested command substitutions contain syntax errors?

## Key Entities

- **Command Record:** A structured representation of a single command containing program, arguments, and chaining operator.
- **Command Kind:** A categorical label denoting the operational category of a command.
- **Fingerprint:** A canonical string representation of command structure with specific parameter values replaced.

## Success Criteria

- **SC-001:** SQL queries can aggregate agent actions by operational category across millions of rows.
- **SC-002:** Common command patterns can be grouped using fingerprint equality in GROUP BY clauses.

## Assumptions

- Agent trace records capture shell commands as single UTF-8 text expressions.
- The majority of recorded agent commands adhere to Bash or POSIX shell syntax.

## Open Questions

- **FR-006:** Should unparseable lines return null or a best-effort fallback?
- **FR-007:** Should task runners like make or npm be classified by recipe or remain run?

## References

- [arch](../__meta__/arch.md) - Base: Architecture documentation corpus spec
