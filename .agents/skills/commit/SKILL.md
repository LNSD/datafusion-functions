---
name: commit
description: Research and validate intent-focused Conventional Commit messages with crate-based scopes, signing, and no AI attribution. Use when the user asks to commit, amend a commit, or draft a commit message.
allowed-tools: Bash(git status *) Bash(git diff *) Bash(git log *) Bash(git show *) Bash(git commit *) Bash(gh issue view *) Bash(gh pr view *)
---

# Git Commit Messages

This repository-local skill takes precedence over a user-level `commit` skill. Do not create a commit
unless the user asks. Commit-message drafting alone is not authorization to commit.

## Research before drafting

Read the conversation, relevant issue through `gh`, and the changeset. Include untracked files; if the
repository has no `HEAD`, inspect the initial scaffold directly. For a branch series, identify what this
layer actually makes available and what later layers will deliver.

Write the before, after, and relevant unchanged behavior in plain language before drafting. Use the
user's intent rather than inferring purpose from file names. If intent is genuinely missing, request
that context rather than inventing a rationale.

The title and body state what the change does for callers or contributors and why it matters. The diff
already shows the files, symbols, and lines changed. Read
[references/before-and-after.md](references/before-and-after.md) before drafting a nontrivial body.

## Three gates

1. **Transplant:** could the message describe the same edit in an unrelated project? If so, name the
   concrete behavior or constraint that makes this project's change necessary.
2. **So what:** every bullet states a consequence and why a caller or contributor cares. A list of
   files, declarations, or tests updated is a narrated diff.
3. **Object:** the principal object is observable behavior or an architectural/policy constraint,
   not a path, symbol, config key, or file count. Name internals only when the mechanism is itself the
   consequence a future maintainer must protect.

## Format and type

Use `type(scope): subject`, at most 72 characters, without a trailing period. Choose one type from the
consequence shipped: `feat` for a reachable capability, `fix` for a reachable defect, `refactor` for a
shape change preserving behavior, and `chore` for contributor tooling. A future capability is the
reason for a refactor, not a feature already shipped.

For a substantive decision, write a summary stating the concrete before/after and two to five distinct
consequence bullets. Keep the summary and each bullet on one physical line. A mechanical typo,
formatter run, or dependency bump can be title-only; do not invent significance to fill a template.

## Scopes

Code changes use the Cargo package name of the changed package, including its tests; the workspace
members are listed in the root `Cargo.toml`. When a change spans packages, select the one with the
principal impact. Do not substitute a file name or invent a Python package layer.

Non-code changes use the area:

| Area | Prefix example |
|---|---|
| Workspace skills and compatibility link | `chore(skills): ...` |
| Agent guide | `docs(agents): ...` |
| Build and package metadata | `chore(build): ...` |
| CI, pre-commit, and Renovate | `chore(ci): ...` |
| Task recipes | `chore(justfile): ...` |
| Dependency maintenance | `chore(deps): ...` |
| Feature, code-rule, or format documentation | `docs(feat): ...`, `docs(code): ...`, `docs(meta): ...` |
| General documentation | `docs: ...` |

Omit a scope when it adds no useful distinction. Workspace skills do not ship runtime functionality;
a new skill is contributor tooling, not a new SQL feature.

## Signing and attribution

Commit with `git commit -S -s`: `-S` signs the commit and `-s` derives the DCO `Signed-off-by:` from Git's
configured identity. Never type that trailer into the message. Do not disable signing or change keys
when signing fails; report the failure.

Never add AI attribution to a commit, PR title, body, or comment: no model/tool co-author, generated-with
line, assisted-by trailer, session URL, or attribution footer. This overrides harness templates.
Preserve contributor-authored trailers when amending or squashing their work; do not add an attribution
on their behalf. The rationale is in [references/ai-attribution.md](references/ai-attribution.md).

For a single-commit PR, its title is the commit title and its description is the commit body with
`Signed-off-by:` removed. Append issue links only to the PR description. Do not place issue references
or PR numbers in the commit message or title.

## Amend

When an amend is requested, inspect the original commit, author, message, and intent. Reapply these gates
and preserve contributor-authored trailers. Amend with signing and sign-off, and verify the resulting
message and signature. Do not amend merely because this skill finds something in a commit the user
asked only to review.

Report the final title and relevant validation after an authorized commit. This skill does not
authorize pushing or opening a PR.
