---
name: "rust-crates"
description: "Cargo.toml section ordering, feature flag rules, kebab-case naming. Load when editing a Cargo.toml, adding a dependency, or adding features to a package"
type: "core"
scope: "global"
---

# Package Configuration (`Cargo.toml`)

**A `Cargo.toml` reads in one fixed order: what the package is, what it builds, what it needs, and how it
is checked.** A reviewer then knows where to look, and two branches adding a dependency each conflict on
different lines. Every package `Cargo.toml` in the workspace follows it.

## 1. Section Ordering

Sections appear in this order, with no other section between them:

1. `[package]`: package metadata
2. Target definitions, `[lib]`, `[[bench]]`, `[[example]]`: what this package builds
3. `[features]`: feature flags and their dependencies
4. `[dependencies]`: runtime dependencies
5. `[dev-dependencies]`: development and test dependencies
6. `[build-dependencies]`: build-time dependencies. `[target.'cfg(…)'.build-dependencies]` takes the same
   slot, since it is the same section under a gate
7. `[lints.<tool>]`: lint configuration

Every section except `[package]` is optional. Dependencies within a section are alphabetical. Sections are
separated by a blank line.

`[package]` inherits shared metadata from the root `[workspace.package]` with `.workspace = true`, so the
version, edition, and license are set once. Target definitions sit high because they answer whether this is
a plain library or also carries benchmarks and examples. A reader asks that before any dependency matters.
Lints sit last because they configure the build and do not compose it.

```toml
# ✅ Good — a reviewer knows where to look for a new dependency.
[package]
name = "datafusion-functions-vector"
version.workspace = true
edition.workspace = true

[[bench]]
name = "distance"
harness = false

[features]
# Portable SIMD distance kernels in place of the scalar loops
simd-kernels = []

[dependencies]
arrow = "59"
datafusion = "55"

[dev-dependencies]
approx = "0.5"
criterion = "0.7"
```

Named dependency groups are allowed when a set of dependencies shares external update ownership or a
lockstep compatibility requirement. `arrow` and `datafusion` are the usual case: each DataFusion release
pins one Arrow major, so the two move together. Each group carries a short comment saying why it is grouped,
and dependencies stay alphabetical inside it.

## 2. Features Section Rules

A `[features]` section is optional. It is added only when a feature is required, such as one gating an
optional dependency. The `default` feature is implicit and is omitted when empty.

When a `[features]` section exists:

- `default` is listed first. Every other feature is alphabetical.
- Feature names are kebab-case: lowercase letters and hyphens only.
- Names are descriptive. `float16-support` says what it enables; `f16` does not say whether it adds a type,
  a kernel, or an Arrow cast. The same holds for `simd-kernels` over `simd` and `serde-support` over `serde`.
- Every feature has a `#` comment above it stating its purpose.

```toml
# ❌ Bad — unordered, undocumented, and named so nobody can tell what enabling one pulls in; `f16` shipped without casts.
[features]
simd = []
serde_Support = ["dep:serde"]
default = ["simd"]
# stuff
f16 = ["dep:half"]
```

```toml
# ✅ Good — `default` first, the rest alphabetical, kebab-case, and each line says what enabling it buys.
[features]
# Default features, enabled unless default-features = false
default = ["simd-kernels"]
# Float16 vector columns, with casts to and from Float32
float16-support = ["dep:half"]
# Serialize and deserialize UDF options with serde
serde-support = ["dep:serde"]
# Portable SIMD distance kernels in place of the scalar loops
simd-kernels = []
```

## Checklist

Before committing code, verify:

- [ ] Sections appear in the correct order: `[package]` → target definitions → `[features]` → `[dependencies]`
      → `[dev-dependencies]` → `[build-dependencies]` → `[lints.<tool>]`
- [ ] Shared metadata is inherited from `[workspace.package]`, not repeated
- [ ] All dependencies within each section are alphabetical, or split into commented named groups that are
      alphabetical inside each group
- [ ] Features use kebab-case naming
- [ ] `default` feature is listed first (if present)
- [ ] All remaining features are alphabetically ordered
- [ ] Every feature has a descriptive `#` comment above it
- [ ] No `[features]` section added unnecessarily

## References

- [principle-least-surprise](principle-least-surprise.md) - Foundation: A `Cargo.toml` in a predictable order is
  read by scanning, never by searching

## External References

- [The Cargo Book — The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)
- [The Cargo Book — Features](https://doc.rust-lang.org/cargo/reference/features.html)
