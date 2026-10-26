---
name: "code-module"
description: "Format for module-scoped code rules in docs/code/module-*.md. Load when creating or editing a module rule"
type: "meta"
scope: "global"
---

# Module Rule Format

## Namespace

`docs/code/module-*.md` documents conventions shared by a named Rust module or a cohesive set of modules.
This specification adds to [code](code.md), which still applies in full; nothing here relaxes it. Prefer a
package rule when the convention describes crate-wide responsibilities or exports.

`code-module.structure.json` checks the module name and requires the sections below. The prose is
authoritative; change both when a machine-checkable rule changes.

## Frontmatter

`name` starts with `module-`. `type` is `"crate"`; `scope` names the owning Cargo package as
`"crate:<package-name>"`.

## Structure

The H1 title identifies the module. `Responsibility` states its single owned concern and boundary;
`Dependencies` describes the allowed direction of collaboration with sibling modules. Then the required
`Checklist` follows, with optional references inherited from [code](code.md).

## Checklist

- [ ] The rule describes a real module boundary in the named package.
- [ ] Responsibility and Dependencies appear in that order.
- [ ] Dependencies do not create a cycle or duplicate another module's ownership.

## References

- [code](code.md) - Extends: Base code rule format
