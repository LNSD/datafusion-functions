---
name: "code"
description: "Format for repository code rules. Load when creating or editing documents in docs/code/"
type: "meta"
scope: "global"
---

# Code Rule Format

## Corpus

`docs/code/` holds binding conventions for code in this workspace. Each document owns one rule or
closely related topic. Describe conventions the repository uses, with readable Rust examples;
keep contracts specific to one module beside that module's code.

The prose specification is authoritative. `code.structure.json` checks its frontmatter, outline,
word caps, and token budget. Change both when a machine-checkable rule changes.

## Frontmatter

Every document begins with YAML frontmatter containing exactly four required string fields:

- `name`: lowercase kebab-case, matching the filename without `.md`.
- `description`: a nonempty description with a `Load when` trigger and no trailing period.
- `type`: one of `principle`, `core`, `arch`, `crate`, or `meta`.
- `scope`: `global` or `crate:<package-name>`, with a lowercase kebab-case package name.

Quote the values. `principle` describes a design principle, `core` a coding convention, `arch`
workspace structure, and `crate` a package convention. `meta` is reserved for format specifications.

## Naming and References

Use lowercase kebab-case filenames. A shared leading segment groups a subject; a namespace
specification may add constraints to matching names. Each layer applies independently and cannot
relax the base. A base specification never inventories its extensions.

Use relative Markdown links for repository documents. A References section names only documents
the rule depends on; an External References section links external authorities. Avoid inventories
of neighboring documents. Use package names from this workspace, without another project's prefix.

## Structure and Content

Open with one H1 title. Add H2 sections named for the rule, followed by a required `Checklist`,
then optional `References` and `External References` sections in that order. No section follows
the references. Omit empty optional sections; all empty sections are forbidden.

State the convention in the present tense and explain its reason where it affects a decision.
Examples illustrate the rule without transcribing implementation code. The checklist contains
concrete checks a reader can apply to changed code.

Each rule section has a 350-word cap, and Checklist has a 250-word cap. References have no word
cap. The whole document has a 20,000-token budget, allowing Rust examples while bounding agent
context. Cut or move overflowing content rather than raising limits to silence a finding.

## Template

````markdown
---
name: "rule-name"
description: "The convention and its purpose. Load when applying this convention"
type: "core"
scope: "global"
---

# Rule Name

## Rule

State the convention and its reason.

## Checklist

- [ ] The changed code follows the convention.
````

## Checklist

- [ ] The document describes an existing convention and owns one subject.
- [ ] Frontmatter values are quoted, and the trigger identifies when to load the rule.
- [ ] Scope and package names match this workspace.
- [ ] Examples are readable, relevant Rust and do not duplicate implementation code.
- [ ] The checklist provides concrete checks.
- [ ] References express dependencies rather than listing neighboring files.
- [ ] Prose and the structure specification enforce the same requirements and limits.
