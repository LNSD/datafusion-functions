---
name: code-rules-check
description: Check a code changeset against the repository's code rules in docs/code/ - select the rule documents the diff falls under and walk each one's checklist against the changed lines, citing the rule behind every finding. Use after implementing and before committing, when reviewing a pull request for compliance with the repository's conventions, or when asked whether code follows the guidelines. A compliance check, not a bug or security review
compatibility: Reads files only. Requires a git checkout and the code rules kept as Markdown documents in docs/code/ with name and description frontmatter
allowed-tools: Bash(git diff *) Bash(git status *) Bash(git merge-base *) Bash(rg *) Bash(ls docs/*) Read
---

# Code Rules Check

Verifies that a changeset follows the rule documents in `docs/code/`. This is a **compliance check, not a
review**: it does not hunt bugs, question the design, or assess security.

One question only: **does this code follow the rules that govern it?**

If `docs/code/` does not exist, state that no code-rule corpus is available. Check the applicable
requirements in `AGENTS.md` without claiming a document-based compliance pass.

## 1. The changeset

```bash
git diff --stat HEAD                                  # uncommitted work, the default
git diff "$(git merge-base HEAD main)"...HEAD         # a whole branch
```

Given explicit paths or commits, check those instead. Check only what the diff touches: an unchanged line
that breaks a rule is not this changeset's finding.

## 2. The rules that govern it

```bash
rg --max-count 3 '^(description|type|scope):' docs/code/*.md
rg --max-count 4 '^(description|status):' docs/arch/adr-*.md   # where the repository keeps decision records
```

An architecture decision record in `docs/arch/` whose `status` is `accepted` governs code as a rule document
does, and its checklist is checked the same way; one in any other status is not a rule.

Select by what the diff **contains**, not by what the task was about. Walk the hunks and, for each construct
in them — a signature, a Result boundary, an import, rustdoc, a test, a log line — find the documents
whose trigger clause names it. Prefer the most specific document in a group, and one scoped to the changed
package or directory over a global one. Read every match. Principle documents, whose descriptions state their
rules, are in scope without being read in full.

## 3. The check

**Each rule document's checklist is the check surface** — its rules restated as verifiable statements. Walk
every item against every changed hunk it applies to. Where a document has no checklist, walk its rules.

Do this inline when the selection is small, which it usually is: one piece of work touches a few rule groups,
and the documents are often already in context from `/code-rules`.

**Fan out only when the selection exceeds about four rule groups**, or the diff spans several packages, and
only if you can run subagents. Then give each agent one group's document paths, the diff command, the
instruction to apply those documents' checklists, and the report format below. Deduplicate what comes back:
two groups flagging one line are reported once, citing both.

## 4. Report

Clean:

> Rules check clean. Applied: `rust-types`, `error-handling`, `rustdoc`.

Violations, most severe first, one per line, with the fix:

> `src/common.rs:118` — **error-handling**: this unchecked downcast can panic on caller-provided
> Arrow input. Return a typed execution error instead.

- **Every finding cites the document that states the rule.** A finding with no document behind it is a style
  opinion — drop it.
- Quote the checklist item when the violation is not self-evident.
- Do not report what the repository's linter or formatter already enforces. A rule document naming a lint rule
  does not mean the rule is enabled: check the linter's configuration before assuming it caught something.
- A rule that seems wrong, or contradicts another document, is a finding against the documents, not the code.

## Initial checkout

If `HEAD` does not exist, use `git status --short` and read untracked files directly. A diff against
`HEAD` cannot show the initial scaffold. Do not stage files merely to make a review command work.
