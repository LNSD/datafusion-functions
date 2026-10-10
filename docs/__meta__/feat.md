---
name: "feat"
description: "Feature documentation format specification. Load when creating or editing feature docs in docs/feat/"
type: "meta"
scope: "global"
---

# Feature Documentation Format

**MANDATORY for ALL feature documents in `docs/feat/`**

## Table of Contents

1. [Core Principles](#1-core-principles)
2. [Frontmatter Requirements](#2-frontmatter-requirements)
3. [Naming Schema](#3-naming-schema)
4. [Document Structure](#4-document-structure)
5. [Content Guidelines](#5-content-guidelines)
6. [Word Caps and Token Budget](#6-word-caps-and-token-budget)
7. [Template](#7-template)
8. [Checklist](#8-checklist)

---

## 1. Core Principles

The corpus in `docs/feat/` is **the feature documentation**; a single member of it is a **feature document**.
A feature document describes something this workspace provides, such as a package or a SQL function, at the
level a reader needs to use it, not at the level a reader needs to modify it.

### The Corpus Grows With Features

Each implemented feature is documented in the change that ships it. The corpus describes existing behavior;
it does not reserve names for planned features. A feature document describing unimplemented behavior is
indistinguishable from one describing broken behavior.

### Feature Docs Are Authoritative

Feature documentation is the **ground truth** for what a feature should do.

- If a feature document exists, the implementation **MUST** align with it
- If code behaves differently than documented, the code is wrong OR the document must be updated
- When implementation changes, update the feature document in the same change

### Describe Behavior, Not Implementation

Feature documents describe **what** a feature does and **why** it exists. They do not document
implementation internals; they reference source files in Code References and let the code speak for itself.
The dividing line is who the reader is: someone calling a function reads its feature document; someone
changing how it walks an array reads the module.

### One Document, One Subject

**A feature document has exactly one reason to change.** Split by subject, not by size. Siblings **link**;
they do not restate. A behavior documented twice has two places to rot and no authority when they disagree.

### A Document's Path Selects Its Specifications

Nothing registers a feature document with a specification: its own path resolves them.

**This specification is the base layer.** It governs every document at `docs/feat/<name>.md` through two
files in `docs/__meta__/`:

| File | Governs | Read by |
|---|---|---|
| `feat.md` | Everything. This document is the authority | A person, and an agent before it writes |
| `feat.structure.json` | The frontmatter fields; the section outline, its order, the caps, the token budget | `lorecraft check` |

A **namespace layer** adds to that base for a group of documents: `feat-<namespace>.md` states its rules in
prose, and `feat-<namespace>.structure.json` holds the parts a check can decide. A layer applies when the
namespace equals the document's name or is a hyphen-delimited prefix of it. Every layer is applied on its own,
broad to narrow, so a layer states only what it adds: it can require a section the base leaves optional,
require a field the base allows, narrow a field, or tighten a cap, and it cannot release a document from
anything this base says. A rule that holds for one group stays out of this file.

The per-type section rules in [§4](#4-document-structure) have no machine-checkable form: a specification is
selected by path, never by `type`. They are checked by review against the [checklist](#8-checklist).

**Prose and schema are one rule set in two forms.** When a rule here changes, change the JSON companion in
the same change.

### Discoverability Through Frontmatter

Feature documents use YAML frontmatter for lazy loading: agents read frontmatter to decide which documents
to load, rather than reading the corpus up front. Agent entrypoint documents do not hardcode feature lists.

---

## 2. Frontmatter Requirements

The rules in this section are held in machine-checkable form under the `frontmatter` key of
[feat.structure.json](feat.structure.json).

```yaml
---
name: "feature-name-kebab-case"
description: "What it explains + when to load it"
type: "meta|feature|component"
status: "stable|experimental|unstable|development"
components: "prefix:name,prefix:name"
---
```

**All values are double-quoted.** The schema cannot see quoting, so this is verified by reading.

### Field Requirements

| Field | Required | Format | Description |
|---|---|---|---|
| `name` | YES | `^[a-z0-9]+(-[a-z0-9]+)*$` | Unique identifier matching the filename (minus .md) |
| `description` | YES | Single line, succinct | Discovery-optimized description |
| `type` | YES | `meta`, `feature`, or `component` | Document classification |
| `status` | YES | enum | Maturity: `stable`, `experimental`, `unstable`, `development` |
| `components` | YES | Prefixed, comma-separated | Packages, modules, skills and specifications a change would touch |
| `crate` | NO | kebab-case | The Cargo package a document belongs to when its name does not say; a namespace may require it |

### Type Definitions

| Type | Purpose | Sections |
|---|---|---|
| `meta` | Groups related features or concepts | No Usage, no Code References |
| `feature` | Documents a user-facing capability | Usage with working examples is required |
| `component` | Documents an internal building block | Code References is required |

### Status Definitions

| Status | Quality | Notes |
|---|---|---|
| `stable` | GA | Breaking changes require a deprecation cycle |
| `experimental` | Preview | Functional, but the interface may change between releases |
| `unstable` | Alpha | Implemented with sharp edges or incomplete areas |
| `development` | N/A | Under active design; may change or be removed |

`status` states where the feature stands today; it never names a version.

### Component Prefixes

Every `components` entry carries one prefix: `crate:` a Cargo package, `module:` a module path inside one,
`skill:` a skill directory, or `spec:` a specification name under `docs/__meta__/` (`spec:feat` stands for
`feat.md` and every `feat.*.json` beside it). Names use letters, digits, underscores, colons, or hyphens.

### Description Guidelines

A description answers what the document explains and when an agent should load it, through a `Load when`
clause. Third person, specific, no ending period.

---

## 3. Naming Schema

Feature names run from broad domain to specific feature: `<domain>-<subdomain>-<variant>`. The first segment
is the **domain**; a namespace layer matches a domain or a longer leading run of segments. A domain with a
namespace layer takes its naming from that layer, so read `docs/__meta__/feat-<domain>.md` before naming a
document in it.

1. **Use kebab-case**, and make `name` match the filename (minus .md)
2. **Domain first**, then progressively specific segments, so related features sort together
3. **Flat directory**: every document lives at the root of `docs/feat/`; the checks ignore subdirectories

---

## 4. Document Structure

The section order below is held in [feat.structure.json](feat.structure.json). Which sections each type
requires or forbids is checked by review.

| Section | meta | feature | component |
|---|:-:|:-:|:-:|
| H1 Title | ✓ | ✓ | ✓ |
| Summary | ✓ | ✓ | ✓ |
| Table of Contents | optional | optional | optional |
| Key Concepts | optional | optional | optional |
| Architecture | optional | optional | optional |
| Configuration | optional | optional | optional |
| Usage | ✗ | ✓ | optional |
| Limitations | optional | optional | optional |
| References | optional | optional | optional |
| Code References | ✗ | optional | ✓ |

1. **H1 Title**: the feature's name, and the only H1
2. **Summary**: 2-4 sentences expanding on the frontmatter description
3. **Table of Contents**: links to the sections below it
4. **Key Concepts**: the terms the document uses, defined once
5. **Architecture**: how the feature fits together, when usage does not show it
6. **Configuration**: options, defaults, and where they are read from
7. **Usage**: how to invoke the feature, with examples that run
8. **Limitations**: known constraints
9. **References**: cross-references to other documents
10. **Code References**: source files behind the feature, one line each, never wrapped

### Section Order

**The order in the table is the order on the page.** Summary opens every document, followed by Table of
Contents and Key Concepts when present. References and Code References close it, in that order. Optional sections keep their relative
order whether or not each is present.

A document may add sections of its own in two places: directly after Summary, Table of Contents and Key
Concepts, for what a reader needs
before Usage, such as an interface; and after Limitations, before References, for everything else. A
namespace layer names the sections its documents add and fixes their place.

**No empty sections.** Omit an optional section rather than leaving a bare heading.

### References

One link per line, the relationship named first: `Dependency`, `Alternative`, `Related`, `Extended by`, or
`Base`. A document's **base** is the document whose name is the longest hyphen-delimited prefix of its own.
Every document except the top of a domain links up to its base; no document links down to its extensions,
except that a component may link to the components it contains. References name dependencies, never an
inventory of neighboring files.

---

## 5. Content Guidelines

**Do:** keep descriptions focused; name source files in Code References; include examples that run; use the
terminology Key Concepts defines.

**Don't:** describe how the workspace is developed (no `just` recipe, CI job or contributor skill); duplicate
`docs/code/`; explain code logic; restate a sibling's behavior; document planned features; record dependency
versions, release status or benchmark figures; leave an optional section empty.

---

## 6. Word Caps and Token Budget

**Word caps keep each section concise** for the person reading it. A word is whitespace-delimited text
outside fenced code blocks and table rows. Table of Contents, References and Code References carry no cap.
Each section a document adds is capped at 300 words.

**A token budget keeps the document cheap to load** for the agent reading it: 4,000 `o200k_base` tokens for
the whole file, frontmatter, code and tables included.

A section over its cap or a document over its budget is a signal about structure: split the subject, move the
detail into the module it describes, or replace a paragraph with a table. A namespace layer can only tighten
these numbers.

---

## 7. Template

````markdown
---
name: "{{feature-name-kebab-case}}"
description: "{{What it explains. Load when [trigger conditions]}}"
type: "{{meta|feature|component}}"
status: "{{stable|experimental|unstable|development}}"
components: "{{prefix:name - use crate:, module:, skill:, or spec:}}"
---

# {{Feature Title}}

## Summary

{{2-4 sentences: what this feature does, why it exists, and its primary use.}}

## Table of Contents

1. [Key Concepts](#key-concepts)
2. [Usage](#usage)
3. [References](#references)

## Key Concepts

- **Term**: What this term means here, in one line

## Usage

{{A working example.}}

## References

- [base-feature](base-feature.md) - Base: Brief description
````

---

## 8. Checklist

- [ ] Frontmatter is valid, every value double-quoted, `name` matches the filename
- [ ] `description` says what it covers and includes a `Load when` clause (no ending period)
- [ ] `status` reflects where the feature stands today, and names no version
- [ ] `components` entries use `crate:`, `module:`, `skill:` or `spec:`, and resolve
- [ ] One H1 title; Summary (2-4 sentences), then Table of Contents and Key Concepts when present
- [ ] **If type=feature**: a Usage section with examples that run
- [ ] **If type=component**: a Code References section naming source files
- [ ] **If type=meta**: no Usage and no Code References section
- [ ] Added sections sit directly after the opening sections or after Limitations, as the namespace fixes
- [ ] References and Code References, when present, close the document in that order
- [ ] References name the relationship, link up to the base, and never down to an extension
- [ ] Every behavior documented exists today; the document has one subject and restates no sibling
- [ ] No development workflow, dependency version, release status, or benchmark figure appears
- [ ] The document is within the word caps and the token budget
