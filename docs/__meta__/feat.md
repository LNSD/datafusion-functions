---
name: "feat"
description: "Format for implemented feature documentation. Load when creating or editing documents in docs/feat/"
type: "meta"
scope: "global"
---

# Feature Document Format

## Corpus

`docs/feat/` describes implemented workspace capabilities for their users and maintainers. Document
one subject per file. Add a feature document in the change that implements the behavior; do not
seed the corpus with planned features or describe implementation details that belong in code.

The prose specification is authoritative. `feat.structure.json` checks its frontmatter, outline,
word caps, and token budget. Change both when a machine-checkable rule changes.

## Frontmatter

Every document begins with YAML frontmatter containing exactly five required string fields:

- `name`: lowercase kebab-case, matching the filename without `.md`.
- `description`: a nonempty description with a `Load when` trigger and no trailing period.
- `type`: `meta`, `feature`, or `component`.
- `status`: `stable`, `experimental`, `unstable`, or `development`.
- `components`: one or more comma-separated entries prefixed by `crate:`, `module:`, `skill:`,
  or `spec:`. Each name uses letters, digits, underscores, colons, or hyphens; omit spaces.

Quote all values. `meta` explains a concept, `feature` a user-facing capability, and `component`
an internal building block. Status states current maturity. Component entries identify the
packages, modules, skills, or specifications a behavior change affects.

## Naming and References

Use lowercase kebab-case filenames. Shared leading segments group related subjects; namespace
specifications add constraints to matching names without relaxing the base. A base specification
never inventories its extensions.

Use relative Markdown links for repository documents and published URLs for external sources.
References name dependencies or related behavior, rather than listing neighboring files. Code
References name source paths and their responsibilities, without duplicating implementation logic.

## Structure and Content

Open with one H1 title. H2 sections appear in this order:

| Section | Required | Word cap | Content |
|---|---|---|---|
| Overview | Yes | 250 | Purpose, audience, and the capability provided |
| Behavior | No | 350 | Input contracts, results, limits, and errors |
| Usage | For feature documents | 500 | Working examples of the implemented capability |
| References | No | 200 | Related behavior and external authorities |
| Code References | For component documents | 200 | Source paths and their responsibilities |

Meta documents omit Usage and Code References. These requirements by document type are checked
by review; the structure specification treats those sections as optional. Empty sections are
forbidden, so omit unused optional sections. The whole document has a 4,000-token budget.
These caps leave room for a focused contract and examples without making one document an inventory.
Cut or split overflowing content rather than raising limits to silence a finding.

## Template

````markdown
---
name: "feature-name"
description: "The implemented capability. Load when using this capability"
type: "feature"
status: "development"
components: "crate:datafusion-functions-vector"
---

# Feature Name

## Overview

Explain the capability and who uses it.

## Usage

Show a working example.
````

## Checklist

- [ ] The document describes implemented behavior and owns one subject.
- [ ] Frontmatter is quoted, its trigger aids discovery, and component names resolve.
- [ ] Status reflects the current maturity of the capability.
- [ ] A feature document includes working Usage examples.
- [ ] A component document includes Code References.
- [ ] A meta document omits Usage and Code References.
- [ ] References do not inventory neighboring documents or duplicate implementation logic.
- [ ] Prose and the structure specification enforce the same requirements and limits.
