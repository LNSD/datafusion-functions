---
name: "principle-rate-of-change"
description: "Rate of Change — one lifetime per type; keep volatile policy out of stable mechanism and resolve invocation-constant facts once. Load when a type holds both configuration and per-call state, when deciding where a resolved fact is computed, or when splitting a module edited on two schedules"
type: "principle"
scope: "global"
---

# Rate of Change (Group by Lifetime, Split What Changes Apart)

## Rule

Things that change at the same rate belong together. Things that change at different rates belong apart, even
when they share a subject. Rate is measured two ways: how often a value is replaced at runtime, and how often
a line of code is edited across releases.

1. **One lifetime per type.** Values fixed when a function is registered, values fixed at planning time, and
   per-batch values are three lifetimes and three types.
2. **Resolve a fact once, at the rate it changes.** A decision fixed for an invocation (the argument types,
   the vector dimension) is made at the start and carried as a value. Re-deriving it per row is wasted work
   and lets rows disagree.
3. **Volatile policy stays out of stable mechanism.** An Arrow array kernel and the rules over its output
   are edited for different reasons. The rules take the computed value.
4. **Rate decides cohesion.** Two things about one subject on different schedules are two concerns. Two
   near-identical things on different schedules are two facts.
5. **History is the evidence.** A file half touched this month and half two years ago names its own seam.
   "This might change one day" is not evidence.

## Examples

1. **One lifetime per type**
   A UDF holds its registration-time signature and per-batch scratch state.

```rust
// ❌ Bad — two lifetimes in one type; the scratch buffer forces a `Mutex` into a type DataFusion
// shares across threads, and every call contends on it.
pub struct L2Distance {
    signature: Signature,
    scratch: Mutex<Vec<f32>>,
}
```

```rust
// ✅ Good — the UDF holds only what is fixed at registration; scratch space is a local per batch.
pub struct L2Distance { signature: Signature }

fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
    let mut scratch = Vec::with_capacity(args.number_rows);
    // ...
}
```

2. **Resolve once, at the rate the fact changes**
   The vector dimension is fixed for the whole array.

```rust
// ❌ Bad — the dimension is read per row; a mismatch is found halfway through, after work was done.
for row in 0..left.len() {
    let dimension = left.value_length() as usize;
    if right.value_length() as usize != dimension {
        return exec_err!("vector dimensions differ");
    }
    distances.append_value(l2(&left_values[row * dimension..], &right_values[row * dimension..]));
}
```

```rust
// ✅ Good — the dimension is checked once, where the arrays are unpacked, and passed in as a value.
fn l2_rows(left: &[f32], right: &[f32], dimension: Dimension) -> Float32Array {
    left.chunks_exact(dimension.get())
        .zip(right.chunks_exact(dimension.get()))
        .map(|(a, b)| Some(l2(a, b)))
        .collect()
}
```

3. **Volatile policy out of stable mechanism**
   A distance kernel and the rule deciding which results to keep.

```rust
// ❌ Bad — every threshold change edits the kernel; a cutoff tweak shipped a division by zero.
let distance = dot(a, b) / (norm(a) * norm(b));
if distance < 0.2 {
    results.append_null();
}
```

```rust
// ✅ Good — the kernel computes; the policy is a separate function over its output with its own tests.
let distances = cosine_rows(left, right, dimension);
let kept = filter_below_threshold(&distances, threshold);
```

## Why It Matters

A type is as easy to reason about as its fastest-changing field. One mutable buffer beside an immutable
`Signature` puts the whole UDF behind a lock and makes every concurrent query contend on it. An `Option` field
that is absent outside one phase is a lifetime that wanted its own type, and its invariant is re-checked at
every use.

Re-deriving a slow fact at a fast rate costs time and consistency. A check done per row runs millions of times
and can fail after part of the output is built.

Mixed rates cost review. When volatile rules sit inside stable kernels, every routine policy edit is a diff
against numeric code, and the reviewer re-reads the kernel or waves the change through.

## Pragmatism Caveat

Rates are estimates. Splitting on a predicted rate is a premature abstraction. Split when the two rates are
structural (per-batch against registration time, planning time against execution) or when history already
shows them. Two settings that have never moved independently are one concern.

Splitting costs threading. When separating a value means passing it through five layers that never read it,
the split may cost more than the coupling. A small struct built and dropped in one function keeps its phases
together.

Two rates kept together on purpose carry a comment at the declaration saying why. An undocumented mix is
always wrong, because the next reader cannot tell a choice from an accretion.

## Checklist

Before committing code, verify:

- [ ] No type mixes registration-time values, planning-time values, and per-batch state
- [ ] No field is `Option` only because it is absent outside one phase
- [ ] Nothing is locked because a minority of its fields mutate
- [ ] A fact constant for an invocation is resolved once at its start and carried as a value
- [ ] No per-row loop re-derives a value that cannot change while the loop runs
- [ ] Rules that change per release are not edited inside kernels that compute, parse, or convert arrays
- [ ] Values with different sources (build time, registration, planning, execution) have different homes
- [ ] Any deliberate mixing of rates carries a comment saying why

## References

- [principle-single-responsibility](principle-single-responsibility.md) - Related: Two rates of change are two
  reasons to change; the same split reached from the other side
- [principle-dry-wet](principle-dry-wet.md) - Related: Two copies that change on different schedules are two
  facts, however alike they look
- [principle-symmetry](principle-symmetry.md) - Related: Divergence on different schedules is when a symmetric
  pair is broken on purpose
- [principle-type-driven-design](principle-type-driven-design.md) - Related: A per-phase type removes the
  `Option` fields a mixed lifetime forces
- [principle-information-hiding](principle-information-hiding.md) - Related: Splitting by rate lets the
  volatile half change without widening the stable half's surface

## External References

- [Rate of Change, in Kent Beck's Implementation Patterns](https://zxuanhong.medium.com/kent-beck-implementation-pattern-principles-6-rate-of-change-4c63354cc84)
- [Tune Software Development for Rate of Change — Kent Beck](https://medium.com/@kentbeck_7670/tune-software-development-for-rate-of-change-not-rate-of-progress-56f93c15a769)
- [Shearing layers](https://en.wikipedia.org/wiki/Shearing_layers)
- [On the Criteria To Be Used in Decomposing Systems into Modules — Parnas](https://dl.acm.org/doi/10.1145/361598.361623)
