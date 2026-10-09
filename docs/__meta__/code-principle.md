---
name: "code-principle"
description: "Format for principle rule documents, docs/code/principle-*.md. Load when creating or editing a principle document in docs/code/"
type: "meta"
scope: "global"
---

# Principle Rule Format

## Namespace

`docs/code/principle-*.md` documents each describe one design principle. This specification adds
to [code](code.md), which still applies in full; nothing here relaxes it.

`code-principle.structure.json` checks the frontmatter values and the section order below. The
prose is authoritative; change both when a machine-checkable rule changes.

## Frontmatter

`name` starts with `principle-`. `type` is `"principle"` and `scope` is `"global"`: a principle
governs all code in the workspace, never one package.

## Structure

The H1 title is the principle's name with a parenthetical clarification, such as "Law of Demeter
(Principle of Least Knowledge)". Four sections follow it directly, in this order, before the
`Checklist` that [code](code.md) requires:

- `Rule`: what to do and what not to do, and how to recognize a violation.
- `Examples`: one to five Bad/Good pairs of Rust code blocks, Bad first, each opening with a
  `// ❌ Bad —` or `// ✅ Good —` comment saying why. With more than one pair, number them with a
  bold title and a sentence of context.
- `Why It Matters`: the consequences of violating the principle and the benefits of following
  it, in coupling, maintenance burden, bug risk, and testability.
- `Pragmatism Caveat`: when deviating is acceptable and how the deviation is documented. It states
  that an undocumented violation is always wrong.

Examples are invented for the document in the terms of DataFusion functions: UDF traits,
signatures, Arrow arrays, and planning or execution errors. They show the least code that carries
the principle, cite no module, and never transcribe workspace code, so no rename can falsify them.

References link other principle documents as `Related`. External References link articles or
books that explain the principle in depth.

## Template

````markdown
---
name: "principle-name"
description: "The principle and its purpose. Load when applying it"
type: "principle"
scope: "global"
---

# Principle Name (Clarification)

## Rule

State what to do and how to recognize a violation.

## Examples

```rust
// ❌ Bad — why this violates the principle
```

```rust
// ✅ Good — why this follows the principle
```

## Why It Matters

State the consequences.

## Pragmatism Caveat

State when deviating is acceptable.

## Checklist

- [ ] The changed code follows the principle.
````

## Checklist

- [ ] The document describes one design principle, and its frontmatter matches this namespace.
- [ ] Rule, Examples, Why It Matters, and Pragmatism Caveat open the body, in that order.
- [ ] Every example is a Bad/Good pair, Bad first, with a comment saying why.
- [ ] Examples use DataFusion function terms and transcribe no workspace code.
- [ ] The Pragmatism Caveat says an undocumented violation is always wrong.
- [ ] References link only other principle documents.

## References

- [code](code.md) - Extends: Base code rule format
