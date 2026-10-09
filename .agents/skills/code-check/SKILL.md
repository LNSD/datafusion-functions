---
name: code-check
description: Compile Rust targets and check Clippy findings. Use after changing Rust code or Cargo metadata, when compiler or lint errors appear, or before commits and reviews.
compatibility: Requires Rust with Clippy and the just task runner. IDE diagnostics are optional.
allowed-tools: Bash(just check *) Bash(just clippy *) Bash(just fmt *)
---

# Code Checking

Run `/code-format` first, then:

```bash
just check
just clippy
```

`check` compiles all workspace targets. `clippy` checks all targets and rejects warnings; Package and workspace manifests define any
additional lint policy; inspect them rather than assuming a lint is enabled. Compiler errors are failed gates,
not lint preferences.

Both recipes accept Cargo flags, for example `just check --examples`. Use their full default scope
for the final check when shared UDF code or Cargo metadata changed.

If semantic IDE tools are available, use diagnostics for a fast local check. A connection refusal means
the IDE is unavailable; continue with Cargo. IDE diagnostics never replace these gates.

Fix findings in the affected code. Do not automatically run `cargo clippy --fix --allow-dirty`, weaken
lint levels, or silence an entire group to obtain a pass. A necessary exception names the precise lint
and explains the reason at the affected construct.

Once compilation and lint pass, choose the appropriate tests through `/code-test`.
