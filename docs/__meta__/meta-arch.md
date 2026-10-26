---
name: "meta-arch"
description: "Responsibility of the `arch` specs in docs/__meta__: `arch.md` and `arch-*.md`, the specs of the arch corpus, and what belongs in a neighbouring family instead. Load when creating or editing an `arch` spec"
type: "meta"
scope: "global"
---

# Arch Spec Family

**Applies to `arch.md` and every `arch-*.md` in `docs/__meta__/`.** It governs those specs and nothing else: not
the documents of `docs/arch/`, which `arch.md` and its namespaces govern.

## Responsibility

The family has one responsibility: the rules for writing the architecture documents of `docs/arch/`. `arch` is the root, with the document numbering, frontmatter and closing references every kind shares; each `arch-<kind>` namespace states one kind of document.

## Belongs Here

- **It is a spec whose name is `arch` or continues it with a hyphen**, such as `arch` and `arch-adr`.
- **It states how the `arch` corpus is written**, not what one document of it says.
- **It stays on its layer.** The root states what holds for the whole corpus, and a namespace states only what it
  adds for its group.

## Belongs Elsewhere

| When the rule is about | It belongs in |
|---|---|
| A single document of `docs/arch/` | that document, not a spec |
| A family other than `arch` | the spec of that family |

## Checklist

- [ ] Every spec it covers is named `arch` or `arch-<namespace>`
- [ ] No rule is about one document of `docs/arch/`

## References

- [arch](arch.md) - Related: Base architecture corpus spec
