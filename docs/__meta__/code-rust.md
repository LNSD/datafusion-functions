---
name: "code-rust"
description: "Format for Rust convention documents, docs/code/rust-*.md. Load when creating or editing a Rust convention document in docs/code/"
type: "meta"
scope: "global"
---

# Rust Convention Format

## Namespace

`docs/code/rust-*.md` documents each govern one Rust construct: what it is named, where it is
declared, how it is spelled, and what a reader may assume on meeting it. This specification adds
to [code](code.md), which still applies in full; nothing here relaxes it.

`code-rust.structure.json` checks the frontmatter values and that References is present. The prose
is authoritative; change both when a machine-checkable rule changes.

## Frontmatter and Naming

`name` is `rust-<aspect>` or `rust-<aspect>-<facet>`. `type` is `"core"` and `scope` is `"global"`.

A `rust-<aspect>-<facet>` document specializes `rust-<aspect>`. A facet exists because the parent
had two reasons to change, not because it grew long. The parent carries rule content and is never
a router that only lists its children. A facet group with no parent is legal.

## Structure

The H1 title is a noun phrase naming the construct, such as "Module Reference Graph", never the
filename respelled. A one- or two-paragraph doctrine follows without a heading: the governing idea
a reader could derive the sections from, and pointers to what neighbouring documents own.

Rule sections are titled `## N. Title`, numbered from 1, with at least two. The title states the
rule, not the topic, so a reader scanning the headings has read the document. Sections have no
sub-headings. Each opens with the rule, then the reason it exists, then its examples.

Examples use invented types and functions in DataFusion function terms, and real crate names.
Bad comes before Good, and each marker comment names the cost concretely. Good-only is correct
where there is no instructive mistake. A section about placement or naming may have no example.

The Checklist opens with `Before committing code, verify:` and has one item per rule, in section
order. References is required. A facet lists its parent first as `Extends`; siblings are
`Related`, and principle documents are `Foundation`. Each entry says what the reader gets there.

## Ownership

Every rule has one home. A document points at a neighbour's rule instead of restating it, in a
fixed, greppable form: `{{Subject}} is owned by [{{doc}}]({{doc}}.md).` in prose, and
`- [{{doc}}]({{doc}}.md) - Related: Owns {{subject}}` in References.

## Template

````markdown
---
name: "rust-aspect-facet"
description: "The construct and its convention. Load when writing or reviewing it"
type: "core"
scope: "global"
---

# Construct Name

The governing idea. Lint suppression is owned by [rust-attrs-lints](rust-attrs-lints.md).

## 1. The Rule, Stated as a Title

The rule, then why it exists.

```rust
// ❌ Bad — what went wrong, and what it cost
```

```rust
// ✅ Good — why this is right
```

## 2. The Next Rule

The rule, then why it exists.

## Checklist

Before committing code, verify:

- [ ] A check a reviewer can run against a diff, for section 1.
- [ ] A check for section 2.

## References

- [rust-aspect](rust-aspect.md) - Extends: What the parent owns
````

## Checklist

- [ ] The document governs one Rust construct, and its frontmatter matches this namespace.
- [ ] The title names the construct; a doctrine paragraph precedes section 1.
- [ ] Rule sections are numbered from 1, at least two, flat, and titled by their rule.
- [ ] Each section states the rule, then its reason, then any examples.
- [ ] Examples are invented, use DataFusion function terms, and put Bad before Good.
- [ ] The Checklist opens with the fixed lead-in and follows section order.
- [ ] References lists a facet's parent first as Extends.
- [ ] Neighbouring rules are pointed at, not restated.

## References

- [code](code.md) - Extends: Base code rule format
