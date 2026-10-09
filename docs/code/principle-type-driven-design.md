---
name: "principle-type-driven-design"
description: "Type-Driven Design — make illegal states unrepresentable with enums and newtypes. Load when designing data types such as vector dimensions or metrics, or reviewing optional fields that allow invalid combinations"
type: "principle"
scope: "global"
---

# Type-Driven Design (Make Illegal States Unrepresentable)

## Rule

Types are designed so invalid states cannot be constructed. Input is parsed at the boundary. Every downstream
function receives a type that rules out the cases it does not handle.

- A struct with several `Option` fields where only some combinations are legal is replaced by an enum whose
  variants are exactly the legal shapes.
- "Succeeded, or here is why not" is an enum returned by value or a `Result` with a typed error. It is never
  `Option<T>` plus an out-of-band message, and never a string the caller inspects.
- The compiler enforces it. A `match` on an enum the workspace owns is exhaustive; a catch-all arm silently
  absorbs the variant added next year. A data-derived index uses `.get(i)`; `slice[i]` asserts a bound the
  type does not carry.
- A newtype's invariant is established in its validating constructor: `FromStr` from a string, `TryFrom` from
  any other type. An unchecked constructor asserts the fact the newtype exists to prove, so it carries a
  `// SAFETY:` comment naming why the invariant already holds.

A function that opens with defensive checks for a state that "cannot happen" has a type that lets it happen.

## Examples

1. **Enum over co-optional fields**
   Resolving an argument's vector type answers "here is the layout, or here is why not". Both-set and
   neither-set are meaningless.

```rust
// ❌ Bad — four representable states, two of them nonsense; the shipped version read `dimension` first
// and panicked on a `ListArray` argument.
pub struct VectorArg {
    pub dimension: Option<Dimension>,
    pub unsupported: Option<String>,
}
```

```rust
// ✅ Good — two states; the match hands the caller a non-optional dimension.
pub enum VectorArg {
    Fixed(Dimension),
    Unsupported { data_type: DataType },
}
```

2. **Parse once into a newtype**
   A distance metric is not a `&str`. It is validated where the SQL literal enters, and the type carries
   the proof.

```rust
// ❌ Bad — a raw string for validated data; every kernel call re-matches it, and a typo falls through.
pub fn distance(metric: &str, a: &[f32], b: &[f32]) -> f32 {
    match metric.to_lowercase().as_str() {
        "cosine" => cosine_distance(a, b),
        _ => l2_distance(a, b),
    }
}
```

```rust
// ✅ Good — one validating constructor; the downstream signature states what it requires.
impl std::str::FromStr for Metric {
    type Err = DataFusionError;

    fn from_str(input: &str) -> Result<Self> {
        match input.to_lowercase().as_str() {
            "cosine" => Ok(Metric::Cosine),
            "l2" => Ok(Metric::L2),
            other => plan_err!("unknown distance metric '{other}'"),
        }
    }
}

pub fn distance(metric: Metric, a: &[f32], b: &[f32]) -> f32 {
    match metric {
        Metric::Cosine => cosine_distance(a, b),
        Metric::L2 => l2_distance(a, b),
    }
}
```

## Why It Matters

Every illegal state a type permits becomes a defensive check somewhere, or a missing check and a panic inside
a query. `dimension: Option<Dimension>` puts an absence check on every call site. An enum makes the caller
handle the failure once, at the match, and hands over a non-optional value afterwards.

Types decide what errors can say. A string that might be a metric produces a wrong answer, or "invalid
input", deep in `invoke_with_args`. A `Metric` that failed to parse produces a `plan_err!` naming the
offending value before a single batch is executed.

## Pragmatism Caveat

Types encode structural invariants, not policy. "An argument is a `FixedSizeList` of `Float32`, never a
variable-length list" is structural and goes in the type. "Vectors are at most 4096 dimensions" is policy
that will change and stays a runtime check.

The same test decides when a primitive earns a newtype. A value with an invariant, a unit, or a same-typed
sibling it must never be swapped with is structural: a dimension and a row count, a 0-based argument index
and a 1-based position in an error message. Each gets a newtype, validated in `FromStr` or `TryFrom` and
constructed at the boundary it enters through; [pattern-newtype](pattern-newtype.md) governs them. A value
with no invariant and nothing to confuse it with, such as a function alias or a free-form message, stays a
plain `String`.

Casting into a validated type is the same error under a nominal type. `Dimension(raw_len)` from an
unvalidated array asserts exactly what the newtype exists to prove. An unchecked constructor that is
warranted carries a `// SAFETY:` comment naming why the invariant already holds.

## Checklist

Before committing code, verify:

- [ ] No struct has two or more `Option` fields whose combinations include meaningless states
- [ ] "Succeeded or here is why not" is an enum or a `Result` with a typed error, never `Option` plus a
      message
- [ ] Every `match` on an enum the workspace owns is exhaustive, with no catch-all arm
- [ ] Data-derived indexing uses `.get()` and handles the absence; `slice[i]` appears only where the bound is
      structurally guaranteed
- [ ] A primitive with an invariant, a unit, or a same-typed sibling it must not be swapped with is a newtype;
      one with neither stays plain
- [ ] Newtype invariants are established in `FromStr` or `TryFrom`; every unchecked constructor carries a
      `// SAFETY:` comment
- [ ] No defensive runtime check re-verifies something the type already guarantees

## References

- [principle-validate-at-edge](principle-validate-at-edge.md) - Related: Boundary parsing produces the
  validated types this principle relies on
- [pattern-newtype](pattern-newtype.md) - Related: Governs the newtypes this principle calls for
- [principle-least-surprise](principle-least-surprise.md) - Related: A well-named type whose shape lies is
  worse than no type
- [principle-law-of-demeter](principle-law-of-demeter.md) - Related: Returning an enum lets a collaborator
  answer completely without exposing internals

## External References

- [Designing with Types: Making Illegal States Unrepresentable (F# for Fun and Profit)](https://fsharpforfunandprofit.com/posts/designing-with-types-making-illegal-states-unrepresentable/)
- [Parse, Don't Validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
- [Parse, Don't Validate and Type-Driven Design in Rust](https://www.harudagondi.space/blog/parse-dont-validate-and-type-driven-design-in-rust/#maxims-of-type-driven-design)
- [The Ultimate Guide to Rust Newtypes](https://www.howtocodeit.com/guides/ultimate-guide-rust-newtypes)
- [Using Types To Guarantee Domain Invariants](https://lpalmieri.com/posts/2020-12-11-zero-to-production-6-domain-modelling/)
