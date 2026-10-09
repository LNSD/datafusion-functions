---
name: "rust-fn-unchecked"
description: "Unchecked constructors that skip FromStr/TryFrom: # Safety docs and a // SAFETY: comment at every call site. Load when adding or reviewing a *_unchecked call"
type: "core"
scope: "global"
---

# Unchecked Constructors

**An unchecked constructor moves the proof of an invariant from the type to the caller, and the caller
writes that proof down at the call.** Where the invariant is established is owned by
[principle-validate-at-edge](principle-validate-at-edge.md). The rustdoc sections a `# Safety` section sits
among are owned by [rust-docs-rustdoc](rust-docs-rustdoc.md).

## 1. Validation Lives in `FromStr` and `TryFrom`

A newtype carries a proof: a value of this type has been checked. The check has one home, `FromStr` for
string input and `TryFrom` for everything else. Every other constructor delegates to it or bypasses it.

An unchecked constructor does not weaken the invariant. It transfers the obligation to prove it to the
caller. The transfer is sound only when the caller holds the proof, and visible only when the call site says
so.

These functions are not `unsafe fn`; no undefined behavior is at stake. The `# Safety` and `// SAFETY:`
conventions are borrowed because the discipline is the same: an obligation the compiler cannot check,
discharged in writing where it is assumed.

## 2. Bypass Validation Only Where the Proof Already Exists

Three reasons warrant an unchecked constructor. Each is a value whose invariant was established somewhere the
type cannot see.

1. Reading a property of an Arrow array whose `DataType` was validated at planning time. Execution receives
   the argument types `return_type` accepted; a re-check repeats the planning check on every batch and adds
   an error path that cannot be taken.
2. Re-wrapping a value taken from an already-valid instance. Borrowing from an existing newtype, or converting
   between its borrowed and owned forms, cannot produce an invalid value.
3. A literal in a test, where the value is visible in the same expression.

Anything else is validation avoidance. A hot kernel is measured first; parsing an already-correct value is
rarely what a profile blames.

## 3. Name the Bypass `_unchecked` and Keep It Narrow

The name states the input and the bypass, so a reader never opens the definition to learn a check was
skipped: `from_ref_unchecked` and `from_owned_unchecked` for the borrowed and owned forms of a `Cow`-backed
newtype, `from_i32_unchecked` or `from_usize_unchecked` for a typed source.

Visibility is as narrow as the callers allow. A `pub(crate)` unchecked constructor cannot be reached by a
consumer without the proof. A `pub` one is API, and every downstream crate inherits the obligation.

## 4. The Declaration Carries a `# Safety` Section

The `# Safety` section states what the caller must guarantee, in terms of the invariant. "The value must be
valid" says nothing. A plain `new` hides the bypass twice: the name does not admit it, and a caller assumes it
validates, because every other constructor does.

```rust
// ✅ Good — the name admits the bypass, and the docs name the obligation being transferred.
/// Create a function name from a borrowed str.
///
/// # Safety
///
/// The caller must ensure the name is non-empty and lowercase. This constructor performs no validation.
pub(crate) fn from_ref_unchecked(name: &'a str) -> Self {
    Self(Cow::Borrowed(name))
}
```

A type with an unchecked constructor also says so at module level: a `//!` comment naming the invariants the
type maintains and where validation happens.

## 5. Every Call Site Carries a `// SAFETY:` Comment

The comment sits immediately above the call and names why the invariant already holds here. "This is fine",
"we know it's valid", and a restatement of the function's own docs record that someone thought about it, and
nothing of what they concluded. Without the comment, a later change to the invariant has no searchable list
of sites to re-examine.

```rust
// ✅ Good — the comment names the reason the proof exists, so a reviewer checks the claim.
let list = as_fixed_size_list_array(&array)?;
// SAFETY: `return_type` accepted this argument only after `Dimension::try_from` checked its list size.
let dimension = Dimension::from_i32_unchecked(list.value_length());
```

Test code is the one exception. A call in a `#[cfg(test)]` module needs no `// SAFETY:` comment; the value is
a literal in the same expression.

A `From`/`Into` impl that re-wraps an already-valid value is not an exception. It writes
`// SAFETY: The input already upholds the invariants` at the call.

## 6. Never Wrap Untrusted Input

Anything that arrives from outside is parsed. A SQL literal, a session config option, a `Utf8` or `Binary`
column holding an encoded vector, and a name passed to a `FunctionRegistry` lookup are never wrapped.
Wrapping asserts the fact the boundary exists to establish, with no error path, so the invalid value
surfaces later in a kernel with nothing tying it to the input that caused it
([principle-validate-at-edge](principle-validate-at-edge.md)).

```rust
// ❌ Bad — a zero dimension enters the kernel, and the first sign is `chunks_exact(0)` panicking the query.
let dimension = Dimension::from_usize_unchecked(options.dimension);

// ✅ Good — the boundary converts, and the user gets a planning error to act on.
let dimension = Dimension::try_from(options.dimension)?;
```

## Checklist

Before committing code, verify:

- [ ] The type's validating constructor is `FromStr` or `TryFrom`, and it is the only place the invariant is
      checked
- [ ] Every constructor that skips validation has `_unchecked` in its name
- [ ] The declaration carries a `# Safety` doc section naming the invariant the caller must uphold
- [ ] The constructor's visibility is as narrow as its callers allow
- [ ] Every call site outside `#[cfg(test)]` has a `// SAFETY:` comment immediately above it, naming why the
      invariant already holds
- [ ] No `// SAFETY:` comment merely restates the function's docs or asserts that the value is valid
- [ ] No value from a SQL literal, config option, string or binary column, or registry lookup is wrapped
      instead of parsed
- [ ] The module documents which invariants the type maintains and where validation occurs

## References

- [rust-fn](rust-fn.md) - Extends: What a constructor is named, and why the bypass takes the `_unchecked` suffix
- [principle-validate-at-edge](principle-validate-at-edge.md) - Foundation: Where the invariant is established,
  and why a type checked at planning is a different case from untrusted input
- [principle-type-driven-design](principle-type-driven-design.md) - Foundation: The newtype carries the proof
  that an unchecked constructor asserts
- [principle-least-surprise](principle-least-surprise.md) - Foundation: `FromStr`/`TryFrom` are the constructors a
  reader expects; a bypass must announce itself in its name
- [rust-docs-rustdoc](rust-docs-rustdoc.md) - Related: The rustdoc sections a `# Safety` section sits among, and
  module-level invariant docs
