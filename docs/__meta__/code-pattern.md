---
name: "code-pattern"
description: "Format for pattern rule documents, docs/code/pattern-*.md. Load when creating or editing a pattern document in docs/code/"
type: "meta"
scope: "global"
---

# Pattern Rule Format

## Namespace

`docs/code/pattern-*.md` documents each describe one construction pattern. This specification adds
to [code](code.md), which still applies in full; nothing here relaxes it.

`code-pattern.structure.json` checks the frontmatter values and the section order below. The
prose is authoritative; change both when a machine-checkable rule changes.

## Frontmatter

`name` starts with `pattern-`. `type` is `"core"` and `scope` is `"global"`: a pattern governs all
code in the workspace, never one package.

## Structure

The H1 title is the pattern's name, optionally with a parenthetical clarification, such as
"Typestate Pattern (State Machines with Types)". Four sections follow it directly, in this order,
before the `Checklist` that [code](code.md) requires:

- `Rule`: what the pattern solves, when to apply it, and how to recognize code that needs it.
- `Examples`: one to five Bad/Good pairs of Rust code blocks, Bad first, each opening with a
  `// ❌ Bad —` or `// ✅ Good —` comment saying why. With more than one pair, number them with a
  bold title and a sentence of context.
- `Why It Matters`: the consequences of not using the pattern and the benefits of applying it, in
  type safety, compile-time guarantees, and correctness.
- `Pragmatism Caveat`: when the pattern is overkill and which simpler alternative suffices.

Document only patterns idiomatic to Rust and DataFusion, not ones borrowed from other ecosystems.
Examples are invented for the document in the terms of DataFusion functions: UDF traits,
signatures, Arrow arrays, and planning or execution errors. They show the least code that carries
the pattern, cite no module, and never transcribe workspace code, so no rename can falsify them.

References link principle documents as `Foundation` and other pattern documents as `Related`.
External References link articles or books that explain the pattern in depth.

## Template

````markdown
---
name: "pattern-name"
description: "The pattern and its purpose. Load when applying it"
type: "core"
scope: "global"
---

# Pattern Name (Clarification)

## Rule

State what the pattern solves and when to apply it.

## Examples

```rust
// ❌ Bad — why this does not use the pattern
```

```rust
// ✅ Good — why this applies the pattern
```

## Why It Matters

State the consequences.

## Pragmatism Caveat

State when the pattern is overkill.

## Checklist

- [ ] The changed code applies the pattern correctly.
````

## Checklist

- [ ] The document describes one construction pattern, and its frontmatter matches this namespace.
- [ ] The pattern is idiomatic to Rust and DataFusion.
- [ ] Rule, Examples, Why It Matters, and Pragmatism Caveat open the body, in that order.
- [ ] Every example is a Bad/Good pair, Bad first, with a comment saying why.
- [ ] Examples use DataFusion function terms and transcribe no workspace code.
- [ ] The Pragmatism Caveat names when the pattern is overkill.
- [ ] References link principles as Foundation and patterns as Related.

## References

- [code](code.md) - Extends: Base code rule format
