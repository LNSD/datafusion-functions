---
name: code-review
description: Review Rust UDF changes for bugs, regressions, security, soundness, and rule compliance. Use when asked for a review or when a change needs scrutiny beyond compilation and linting.
compatibility: Requires access to the changeset and repository files; Cargo checks are optional evidence.
allowed-tools: Bash(git diff *) Bash(git status *) Bash(git merge-base *) Bash(rg *) Bash(just check *) Bash(just clippy *)
---

# Code Review

Review the requested changeset. For uncommitted work use `git status --short` and `git diff HEAD`; for a
branch, compare it with its actual base. If `HEAD` does not exist, inspect the untracked scaffold directly.
Do not stage files to obtain a diff. Include untracked files in the review subject.

Run `/code-rules-check` for declared rules. Where `docs/code/` is absent, use `AGENTS.md`, the documented
input contract, and current public API without claiming a corpus-based check.

## Review dimensions

- **Arrow correctness:** validate nested types and dimensions; respect null bitmaps, offsets, slices,
  empty batches, and scalar/array broadcasting. A `zip` must not silently truncate mismatched vectors.
- **Numerical behavior:** check accumulation precision, finite values, zero norms, cancellation, and
  bounds. Preserve the documented distinction between null output and execution errors.
- **Errors and safety:** external inputs must not trigger unchecked indexing, invalid downcasts,
  `unwrap`, or panics. Examine any unsafe code and state the invariant proving it sound.
- **Compatibility:** inspect SQL names, signatures, return types, `functions` builders, `udfs` factories,
  registration behavior, MSRV, and DataFusion/Arrow compatibility.
- **Performance:** detect unnecessary buffer copies, allocations inside element loops, and repeated
  setup. Do not claim SIMD or allocation-free execution without evidence.
- **Security:** check credential handling, logging, input-driven resource growth, and command execution.
- **Tests and documentation:** determine whether tests pin the observable behavior and whether the README
  and rustdoc remain true. Clippy passing does not prove those contracts.

Report actionable findings in severity order, with file, line, concrete trigger, consequence, and a
proposed correction. Distinguish demonstrated defects from questions and residual risks. Avoid style
opinions the repository has not adopted. Fix within the requested review scope only.

Review inline by default. Use delegation only when the user requests it or an applicable instruction
requires it. This skill does not authorize commits, pull requests, messages, or publishing.
