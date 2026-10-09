---
name: "principle-least-surprise"
description: "Principle of Least Surprise — code behaves as its name and shape predict; deviations are documented. Load when naming UDFs or their methods, designing constructors, or reviewing a public API surface"
type: "principle"
scope: "global"
---

# Principle of Least Surprise (Follow Rust Idioms and Conventions)

## Rule

Code behaves as a reader predicts from its name, signature, and shape. What a function returns, what it
touches, and whether it can fail are guessable without opening it. Where `std` or DataFusion sets a pattern
for naming, traits, or method semantics, follow it.

1. **Names follow the standard library.** `new`, `with_*`, `as_*`, `to_*`, `into_*`, `try_*`, and `is_*`
   keep their `std` meaning for cost and ownership. A `ScalarUDFImpl::name()` is the SQL name users type.
2. **Construction reveals its cost.** A value is never half-initialized. A constructor that validates says
   so with `try_new` and returns a `Result`.
3. **Fallibility is visible.** A function that can fail returns `Result`; it does not panic on bad input,
   and it does not silently return null for an error case.
4. **Pairs share a vocabulary.** Where the codebase uses `register`/`deregister`, do not introduce
   `add`/`drop`. [principle-symmetry](principle-symmetry.md) owns the shape a pair shares; this rule owns
   the words.
5. **More than two or three related parameters go in a config struct or a builder.** A positional `bool` is
   never one of them: `normalize(array, 0.0, true, false)` cannot be reviewed.
6. **No hidden effects.** A function named for a computation does not register functions, mutate session
   state, or log. A lookup and an effect are two functions.

## Examples

1. **`new` does not validate; fallible construction is `try_new`**
   A dimension of zero is invalid, so a constructor that accepts any `usize` cannot be `new`.

```rust
// ❌ Bad — `new` panics on zero; a caller reading `Dimension::new(n)` cannot tell it may abort.
pub fn new(value: usize) -> Self {
    assert!(value > 0, "dimension must be positive");
    Self(value)
}
```

```rust
// ✅ Good — the name and the return type both say construction can fail.
pub fn try_new(value: usize) -> Result<Self, DataFusionError> {
    if value == 0 {
        return plan_err!("vector dimension must be positive");
    }
    Ok(Self(value))
}
```

2. **Separate the lookup from the effect**
   Only one of "which function handles this name" and "register it" touches the session.

```rust
// ❌ Bad — a name that reads as a query, a body that registers; a caller probing names mutates the session.
pub fn function_for(ctx: &SessionContext, name: &str) -> Option<ScalarUDF> {
    let udf = all_functions().into_iter().find(|f| f.name() == name)?;
    ctx.register_udf(udf.clone());
    Some(udf)
}
```

```rust
// ✅ Good — the query is pure, and the effect names itself.
pub fn function_for(name: &str) -> Option<ScalarUDF> {
    all_functions().into_iter().find(|f| f.name() == name)
}

pub fn register_all(registry: &mut dyn FunctionRegistry) -> Result<(), DataFusionError> {}
```

## Why It Matters

Every broken convention forces a reader to open the implementation. Across a codebase that cost is paid in
bugs: a caller who assumes `as_slice()` is a borrow calls it once per row, and a caller who assumes `new`
cannot panic passes a user-supplied dimension straight through.

Consistency compounds. If every validating constructor is `try_new`, then `Metric::new(..)` tells a reviewer
the value cannot be invalid. Standard traits buy ecosystem integration: `FromStr`, `From`, `TryFrom`, and
`?` compose with DataFusion's error conversions. A custom constructor needs custom glue at every boundary.

## Pragmatism Caveat

A domain term beats a convention when it is clearer. `cosine_distance` beats `to_cosine_distance` even
though it allocates a new array, because distance is the established word. Prefer the domain word only when
it is more predictable. A DataFusion trait dictates its own method names (`invoke_with_args`,
`return_field_from_args`); match the foreign convention at the boundary and the workspace convention
everywhere else.

A deliberate deviation (a method that never fails, a null result for a documented case, a name a trait
forced) says why in a doc comment at the declaration. An undocumented deviation is always wrong; the next
reader cannot tell it from a mistake.

## Checklist

Before committing code, verify:

- [ ] Names follow the standard library's vocabulary for cost and ownership
- [ ] No value is observable half-initialized; a validating constructor is `try_new` and returns `Result`
- [ ] Failure is a `Result` with a DataFusion error, not a panic or an undocumented null
- [ ] No function named for a query performs an effect; lookup and effect are separate functions
- [ ] More than two or three related parameters are a config struct or a builder; no positional `bool`
- [ ] Every intentional deviation (a domain verb, a trait-imposed shape) is documented at the declaration

## References

- [principle-type-driven-design](principle-type-driven-design.md) - Related: The same discipline, for types
- [principle-validate-at-edge](principle-validate-at-edge.md) - Related: `FromStr`/`TryFrom` parse at the edge
- [principle-single-responsibility](principle-single-responsibility.md) - Related: A type that cannot be
  named in one sentence cannot have a predictable API
- [principle-symmetry](principle-symmetry.md) - Related: A prediction is available only where the same idea
  keeps the same shape

## External References

- [Rust API Guidelines — Naming](https://rust-lang.github.io/api-guidelines/naming.html)
- [Principle of Least Surprise (principles-wiki.net)](https://principles-wiki.net/principles:principle_of_least_surprise)
- [The Principle of Least Astonishment](https://dev.to/notmattlucas/the-principle-of-least-astonishment-3f9k)
- [What is the Principle of Least Astonishment?](https://softwareengineering.stackexchange.com/a/187462)
