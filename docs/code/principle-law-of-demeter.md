---
name: "principle-law-of-demeter"
description: "Law of Demeter — a unit talks to its immediate collaborators, never through them to reach something further. Load when reviewing call chains, field access patterns, or coupling concerns"
type: "principle"
scope: "global"
---

# Law of Demeter (Principle of Least Knowledge)

## Rule

A method `m` on a type `T` calls methods only on `self`, on `m`'s arguments, on values `m` created, and on
`T`'s own fields. It does not reach through a chain of values to something further along the graph.

`a.b().c().do_something()` is a violation, and so is `a.b.c.do_something()` through public fields. Stop at
`a.b()`. If something from `c` is needed, ask `a` or `b` to hand over the value, or accept the value as a
parameter.

A chain where every link is the same logical value is not a violation. Builder chains
(`SessionConfig::new().with_target_partitions(4)`), iterator adapters
(`array.iter().flatten().map(..).collect()`), `Result` and `Option` combinators
(`.map_err(..).ok_or_else(..)`), downcasts such as `array.as_fixed_size_list()`, and a match on an enum a
direct collaborator returned (`match VectorArg::try_from(&args.args[0])? { .. }`) all stay on one value.

## Examples

1. **Ask the collaborator**
   A UDF needs the dimension of its vector argument. The argument's field owns the type layout.

```rust
// ❌ Bad — the caller depends on the field's type nesting; a `LargeList` or extension type breaks it.
let DataType::FixedSizeList(_, size) = args.arg_fields[0].data_type() else { return plan_err!("...") };
let dimension = *size as usize;
```

```rust
// ✅ Good — one call to a function that owns the question answers it completely.
match VectorType::try_from(args.arg_fields[0].as_ref())? {
    VectorType::Fixed(dimension) => Ok(dimension),
    VectorType::Variable => plan_err!("vector_norm requires a fixed-size vector"),
}
```

2. **Receive the value, not the graph that contains it**
   A kernel needs a metric and two arrays. It takes exactly those.

```rust
// ❌ Bad — three levels of reach-through; a unit test has to stand up a whole session.
fn distances(state: &SessionState, args: &[ArrayRef]) -> Result<Float32Array> {
    let metric = &state.config().options().extensions.get::<VectorOptions>().unwrap().metric;
}
```

```rust
// ✅ Good — the collaborators are parameters; the caller that holds the session navigates once, at the seam.
fn distances(metric: Metric, left: &FixedSizeListArray, right: &FixedSizeListArray) -> Result<Float32Array> {}
```

## Why It Matters

A reach-through chain turns a private detail into a public contract. When the vector type changes from a
`FixedSizeList` to an Arrow extension type, every caller that matched its nesting breaks, and the type system
never said those callers existed. A module whose callers stay on its functions can restructure its internals
freely.

The second cost is testability. A kernel that navigates `state.config().options().extensions` is exercised
only by building a session, so it is covered by an end-to-end SQL test or not at all. A kernel that takes a
`Metric` and two arrays is tested with two hand-built arrays in three lines.

## Pragmatism Caveat

Navigating a plain data structure you own is reading data, not coupling: an options struct, arguments just
validated. Reading `args.args` and `args.number_rows` from `ScalarFunctionArgs` is talking to a direct
argument. The rule targets navigation through behavioral values that could hide their internals. A
deliberate reach-through into one of those carries a comment saying why a delegating method on the direct
collaborator, or passing the value in, was rejected. An undocumented violation is always wrong.

## Checklist

Before committing code, verify:

- [ ] No expression navigates two or more levels into another type's fields to reach behavior
- [ ] Functions accept the values they use (a metric, an array, a dimension), never a container to dig
      through
- [ ] Cross-crate access goes through public functions and methods, never through another crate's internal
      collections or state
- [ ] Fluent chains on one logical value (builders, iterators, `Result`/`Option` combinators, downcasts,
      matched enums) are not mistaken for violations
- [ ] Every deliberate reach-through is local and carries a comment with its rationale

## References

- [principle-single-responsibility](principle-single-responsibility.md) - Related: A type that must be
  navigated deeply usually owns too much
- [principle-inversion-of-control](principle-inversion-of-control.md) - Related: Injecting the value you need
  is the standard cure for reach-through
- [principle-type-driven-design](principle-type-driven-design.md) - Related: Returning an enum lets a
  collaborator answer a question completely without exposing its internals

## External References

- [Law of Demeter — Principle of Least Knowledge](https://dev.to/dazevedo/law-of-demeter-principle-of-least-knowledge-35l2)
