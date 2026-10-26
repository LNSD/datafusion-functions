---
name: "code-datafusion-udfs"
description: "Format for generic DataFusion UDF rules, `docs/code/datafusion-udfs-*.md`. Load when creating or editing a DataFusion UDF rule"
type: "meta"
scope: "global"
---

# DataFusion UDF Rule Format

## Namespace

Each document matching `docs/code/datafusion-udfs-*.md` describes one recommendation for authoring
DataFusion scalar UDFs that applies across projects. This specification adds to code, which still applies
in full; nothing here relaxes it.

The prose is authoritative. `code-datafusion-udfs.structure.json` checks the frontmatter and section
outline below.

## Frontmatter

The name starts with `datafusion-udfs-`. The type is `core` and the scope is `global` because these
recommendations apply to DataFusion UDFs across projects.

## Structure

The H1 title names the recommendation. Sections follow it in this order before the Checklist required
by `code`:

- Rule: the recommendation and when it applies.
- Examples: optional invented, minimal examples that do not transcribe project implementation code.
- Why It Matters: the reason and consequences.
- Pragmatism Caveat: optional limits or cases where the recommendation does not apply.

Omit Examples when they add no useful distinction. References and External References may follow the
checklist in the order required by `code`.

## Checklist

- [ ] The document describes one general DataFusion scalar UDF recommendation.
- [ ] Its name starts with `datafusion-udfs-`, type is `core`, and scope is `global`.
- [ ] Rule, optional Examples, Why It Matters, and optional Pragmatism Caveat appear in that order.
- [ ] Examples are invented, minimal, and contain no project-specific paths or implementation symbols.

## References

- [`code`](code.md) - Extends: Base code rule format
