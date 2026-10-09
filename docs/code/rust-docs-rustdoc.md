---
name: "rust-docs-rustdoc"
description: "Rustdoc: crate, module and item levels; mandatory # Panics and # Errors; never # Returns, # Arguments or usage # Examples; SQL-facing UDF documentation. Load when writing a /// or //! comment"
type: "core"
scope: "global"
---

# Rustdoc

Rustdoc is the caller's channel: the hover text at every call site and the rendered API page. It carries
contracts and domain, in the timeless present. Channel choice, voice, and length are owned by
[rust-docs](rust-docs.md). History and justification are owned by
[rust-docs-comments](rust-docs-comments.md). This document owns what each rustdoc level says and which
sections it carries.

## 1. Three Levels, Three Questions

Each level answers one question. A doc comment answering the wrong one is in the wrong place.

| Level      | Written as                   | Answers                                                      |
|------------|------------------------------|--------------------------------------------------------------|
| **Crate**  | `//!` in `src/lib.rs`        | What is this package for, and why would I depend on it?      |
| **Module** | `//!` at the top of a module | Why does this module exist, and what does it defend against? |
| **Item**   | `///` on the item            | What does calling this give me, and what must I uphold?      |

```rust
// ✅ Good — the crate root, for someone deciding whether to depend on it.
//! Vector functions for Apache DataFusion: distance and normalization UDFs over `FixedSizeList` columns.

// ✅ Good — the module says what it protects, which no signature carries.
//! Vector decoding. Kernels accept only a `FixedSizeList` of `Float32`; every other encoding is converted
//! here, before a kernel sees it.
```

A module `//!` states the invariant the module relies on or upholds. That is a fact about this code, so it
belongs here and not in a rule document.

A test module carries one line or nothing. A `#[cfg(test)] mod tests` module and a file under `tests/` are read
only by someone already inside them, and the test names say what they cover. A header that re-explains the
feature under test is a copy of the feature doc; link the doc instead.

## 2. Say the Contract, Briefly

Every public item carries a description: the contract in one sentence, and one more for a behavior a caller
must schedule around. Nothing else.

```rust
// ❌ Bad — the signature in prose. It says nothing the reader cannot see, and rots when an argument moves.
/// Registers the functions in the registry
///
/// # Arguments
/// * `registry` - The function registry into which the functions are registered
pub fn register(registry: &mut dyn FunctionRegistry) -> Result<()> {}

// ✅ Good — the contract, then the one behavior a caller has to plan for.
/// Register every vector UDF with `registry`. A function already registered under the same name is replaced.
pub fn register_all(registry: &mut dyn FunctionRegistry) -> Result<()> {}
```

`# Arguments` is never written. A parameter that needs explaining needs a better name or a type that carries
the meaning ([pattern-newtype](pattern-newtype.md)).

## 3. Sections That Are Never Written

The reader is holding the signature. A section that restates it is a second place for the same fact to rot.

- **`# Returns`**: the return type says it.
- **`# Arguments`**: see [§2](#2-say-the-contract-briefly).
- **`# Examples`**: usage examples are what tests are for. The one exception is a doctest that pins a
  contract, such as the exact rendering of a formatting impl, which [rust-fn-fmt](rust-fn-fmt.md) requires. A
  doctest that only demonstrates typical usage is not written.

```rust
// ❌ Bad — a usage demo duplicating a test. `/// Parse a metric name from a literal option.` is the whole doc.
/// Parses a distance metric
///
/// # Examples
/// ```
/// assert!("cosine".parse::<DistanceMetric>().is_ok());
/// ```
impl FromStr for DistanceMetric {}
```

The bans hold when the reader can see the signature. A UDF called from SQL, such as
`cosine_distance(embedding, [0.1, 0.2])`, has a reader with no signature. Its arguments, return type, and an
example query belong in the `Documentation` that `ScalarUDFImpl::documentation` returns, which DataFusion
shows to SQL users. The rustdoc on the implementing struct still follows the bans.

## 4. Sections That Are Mandatory

**`# Panics`**: any function that can panic names the condition. That includes a body that reaches an
`unwrap`, an `expect`, a `panic!`, an index from data, or a call to something that panics.

```rust
// ✅ Good — the condition, not the mechanism.
/// The dimension shared by `vectors`.
///
/// # Panics
///
/// Panics if `vectors` is empty.
pub fn common_dimension(vectors: &[&[f32]]) -> Dimension {}
```

An `unwrap` or `expect` that an invariant makes unreachable gets a `// SAFETY:` comment on its statement and
no `# Panics` section. Documenting a panic that cannot occur misleads the caller.

**`# Errors`**: any fallible public function says what its failures mean here. One sentence, or one clause
per kind when the meaning differs at this call: a plan error for arguments rejected at planning, an
execution error for a value rejected while a batch runs. It names the condition, never the
`DataFusionError` variant list.

```rust
// ✅ Good — what each failure means to the caller.
/// The vector pair two distance arguments describe.
///
/// # Errors
///
/// Returns a plan error if either argument is not a `FixedSizeList` of floats, or their dimensions differ.
pub fn vector_pair(left: &DataType, right: &DataType) -> Result<VectorPair> {}
```

**`# Safety`**: required on every constructor that skips validation. It is owned by
[rust-fn-unchecked](rust-fn-unchecked.md), with the `// SAFETY:` comment each call site carries.

The `Cargo.toml` feature comment is owned by [rust-crates](rust-crates.md).

## Checklist

Before committing code, verify:

- [ ] The crate root `//!` says what the package is for, in the terms of someone deciding whether to depend
      on it
- [ ] Every non-test module has a `//!` comment saying why it exists and stating any invariant it relies on or
      upholds; a test module has one line or none
- [ ] Every public item has a one-or-two-sentence description, contract first
- [ ] No `# Returns`, `# Arguments`, or usage-demo `# Examples` section was added
- [ ] Any doctest present pins a contract rather than demonstrating typical usage
- [ ] SQL-facing arguments, return type, and example query of a UDF sit in its `Documentation`, not in rustdoc
- [ ] Every function that can panic has a `# Panics` section naming the condition
- [ ] A provably-unreachable `unwrap`/`expect` has a `// SAFETY:` comment on its statement and no `# Panics`
      section
- [ ] Every fallible public function has an `# Errors` section saying what its failures mean, in one sentence
      or one clause per kind
- [ ] Every validation-skipping constructor has a `# Safety` section

## References

- [rust-docs](rust-docs.md) - Extends: The intent rule, the audience routing, the shared voice, and the
  sentence budgets, applied to the caller's channel
- [rust-docs-comments](rust-docs-comments.md) - Related: The editor's channel, and the leading comment that
  sits after a doc comment
- [rust-fn-unchecked](rust-fn-unchecked.md) - Related: Owns `# Safety` sections and their call-site comments
- [rust-fn-fmt](rust-fn-fmt.md) - Related: Owns the one place a doctest is mandatory, because the rendering is the
  contract
- [rust-crates](rust-crates.md) - Related: Owns `Cargo.toml` feature documentation
- [pattern-newtype](pattern-newtype.md) - Related: The type that carries a parameter's meaning in place of an
  `# Arguments` entry

## External References

- [The rustdoc book: How to write documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
- [Rust API Guidelines: Documentation](https://rust-lang.github.io/api-guidelines/documentation.html)
