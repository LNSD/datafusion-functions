---
name: "code-package"
description: "Format for package-scoped code rules in docs/code/package-*.md. Load when creating or editing a package rule"
type: "meta"
scope: "global"
---

# Package Rule Format

## Namespace

`docs/code/package-*.md` documents conventions owned by one Cargo package. This specification adds to
[code](code.md), which still applies in full; nothing here relaxes it. The package rule's `scope` names
the exact package. Keep responsibilities and public surface grounded in the package's manifest and source.

`code-package.structure.json` checks the package scope and requires the sections below. The prose is
authoritative; change both when a machine-checkable rule changes.

## Frontmatter

`name` starts with `package-`. `type` is `"crate"`, and `scope` is `"crate:<package-name>"` using the
workspace Cargo package name.

## Structure

The H1 title names the package. After it, `Responsibility` describes the package's purpose and boundaries;
`Layout` describes crate-root modules and their ownership; `Public Surface` describes exported modules,
functions, types, and registration entry points. `Checklist` follows those sections, then the optional
references inherited from [code](code.md).

## Checklist

- [ ] The rule is grounded in the named package's manifest and source.
- [ ] Responsibility, Layout, and Public Surface appear in that order.
- [ ] The document does not prescribe conventions outside its package.

## References

- [code](code.md) - Extends: Base code rule format
