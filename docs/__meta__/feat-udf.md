---
name: "feat-udf"
description: "Format for SQL function feature documents, docs/feat/udf-*.md. Load when creating or editing the feature document of a SQL function in any datafusion-functions-* package"
type: "meta"
scope: "global"
---

# UDF Feature Format

## Namespace

`docs/feat/udf-*.md` documents each describe one SQL function, for the users who call it. This
specification adds to [feat](feat.md), which still applies in full; nothing here relaxes it.

`feat-udf.structure.json` checks the frontmatter values and the sections and order below. The prose is
authoritative; change both when a machine-checkable rule changes.

## Frontmatter and Naming

The filename is `udf-<function>.md`, where `<function>` is the SQL function name with underscores
replaced by hyphens. It does not name the package: SQL function names are unique across the workspace, so
the function alone identifies the document. One function per document; its DataFrame builder and cached
UDF get no documents of their own.

`crate` is required and names the Cargo package that registers the function, such as
`datafusion-functions-jsonschema`. `type` is `"feature"`. `components` includes `crate:<crate>`.

## Structure

The H1 title is the SQL function name in code format, such as `` `my_function` ``. Sections appear in
this order; all are required, and the base's Table of Contents and Key Concepts are omitted:

| Section | Place in the base outline | Content |
|---|---|---|
| Summary | Base | What the function computes and when to call it |
| Signature | Added after Summary | A `sql` block with the call form and result type, `my_function(arg, ...) -> Type`, then a table with one row per argument: its name, the Arrow types it accepts, and whether it may be a literal, a column, or either |
| Semantics | Added after Summary | What a call means beyond its types: what is rejected at planning, null handling, which values fail execution, and how constant arguments are treated |
| Usage | Base | SQL queries only, with any cast the arguments need |
| Use Cases | Added after Limitations | One H3 per problem solved: a brief overview of why the function fits, then a SQL query |
| References | Base | The package's feature document, as a `Dependency`, and related functions |

Signature is the one place an argument's types appear; other sections refer to arguments by name.
Registration belongs in the package's feature document, and the DataFrame builder in rustdoc.

## Checklist

- [ ] The filename is `udf-<function>.md`, with the SQL function name in kebab case.
- [ ] The document covers exactly one function and is titled with its SQL name.
- [ ] `crate` names the package that registers the function, and `components` includes it.
- [ ] Signature follows Summary, with the call form and result type, then one table row per argument.
- [ ] No section outside Signature restates an argument's types.
- [ ] Semantics states planning errors, nulls, execution errors, and constants.
- [ ] Usage shows SQL queries only, with any casts their arguments need.
- [ ] Use Cases has one H3 per use case, each with a brief overview and a SQL query.
- [ ] References link the package's feature document as a `Dependency`.
