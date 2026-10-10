---
name: "feat-datafusion-functions"
description: "Format for package feature documents, docs/feat/datafusion-functions-*.md. Load when creating or editing the feature document of a datafusion-functions-* package"
type: "meta"
scope: "global"
---

# Package Feature Format

## Namespace

`docs/feat/datafusion-functions-*.md` documents each describe one Cargo package of the workspace, for the
users who add it to a DataFusion application: what it provides and how to register its functions. This
specification adds to [feat](feat.md), which still applies in full; nothing here relaxes it.

`feat-datafusion-functions.structure.json` checks the frontmatter values, that Usage and References are
present, and that no Table of Contents or Key Concepts appears. The prose is authoritative; change both when a
machine-checkable rule changes.

## Frontmatter and Naming

The filename is the package name: `datafusion-functions-<name>.md`. One package per document.

`type` is `"feature"`. `components` includes `crate:<package>`, naming the package the filename names.

## Structure

The H1 title is the package name in code format, such as `` `datafusion-functions-my-crate` ``. Sections
appear in this order:

| Section | Required | Content |
|---|---|---|
| Summary | Yes | What the package provides and for whom, ending with the one call that registers its functions, `datafusion_functions_<name>::register_all(&mut ctx)?;`, in a `rust` block |
| Semantics | No | Rules every function of the package shares, such as null handling or network access |
| Usage | Yes | Rust registration on a `SessionContext` imported from the module that declares it, `datafusion::execution::context`, never from `datafusion::prelude`; whether registration replaces any DataFusion built-in; then a SQL query calling a registered function |
| References | Yes | The standards or projects the package depends on or relates to |

A Table of Contents and Key Concepts are forbidden: a package document is short enough to read whole,
and its terms belong to its functions' documents.

Summary points to its functions' documents by their filename pattern, `udf-<function>.md`, and their
`crate` field; the document never lists them, since a list goes stale with the next function. A rule specific
to one function belongs in that function's document, not here.

## Checklist

- [ ] The filename and `name` are the package name, and `components` includes `crate:<package>`.
- [ ] The title is the package name in code format.
- [ ] Summary ends with the `register_all` call in a `rust` block.
- [ ] Semantics, when present, states only rules every function of the package shares.
- [ ] Usage shows Rust registration with `SessionContext` imported from `datafusion::execution::context`,
      says whether it replaces a built-in, and runs a SQL query.
- [ ] The document points to its function documents by pattern and never lists them.
- [ ] No Table of Contents and no Key Concepts.
