---
name: "rust-mods"
description: "Module doctrine: a module is named by its path, a parent declares its children, references flow one way. Load when adding a module, moving one, or deciding which file an item belongs in"
type: "core"
scope: "global"
---

# Module Tree

**A package's module tree is its structure.** Three invariants hold everywhere in the workspace, and each rule
in the `rust-mods-*` group is one of them made operational. File layout is owned by
[rust-mods-files](rust-mods-files.md), the reference graph by [rust-mods-graph](rust-mods-graph.md), and
member order by [rust-mods-members](rust-mods-members.md).

## 1. A Module Is Found by Its Name

A module's name and its location are the same fact. File and directory names match the module name exactly.
A reader who sees `distance::metric` in a path, an error message, or a backtrace knows which file to open.

## 2. A Parent Declares Its Children and Owns What Escapes Them

The unit of encapsulation is the module. A parent declares which children exist and re-exports what callers
may reach. A child's internals stay a decision the parent can revise without any caller noticing: the Arrow
kernel behind a distance UDF can split or merge while callers keep naming the same UDF type. A module's public
surface is what its parent re-exports, never the union of everything its descendants marked `pub`.

## 3. References Flow One Way

A module refers to what it owns and to what sits beside it, never back to the file that declared it. The
reference graph over a package's modules is acyclic: down to a child, or across to a sibling, with no path
returning to where it started. A cycle costs a reader the ability to understand a subtree without first
reading its parent. When a child needs something its parent holds, the item is in the wrong file. Move it.

## Checklist

Before committing code, verify:

- [ ] The module's file and directory names match its name exactly
- [ ] The parent declares the child and re-exports what callers outside the module need
- [ ] Nothing the module needs is declared in the file that declares the module
- [ ] No sequence of references leads from a module back to that same module

## References

- [rust-mods-files](rust-mods-files.md) - Related: Where module files sit and what the named module file carries
- [rust-mods-graph](rust-mods-graph.md) - Related: Which references between those files are legal
- [rust-mods-members](rust-mods-members.md) - Related: Order of the items inside a single module file
- [principle-information-hiding](principle-information-hiding.md) - Foundation: The module is the privacy boundary, and visibility is gated at the declaration
