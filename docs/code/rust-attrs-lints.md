---
name: "rust-attrs-lints"
description: "Lint suppression with #[expect] and a mandatory reason, scoped to the narrowest item. Load when silencing a compiler or clippy warning, reviewing an #[allow], or adding lint attributes"
type: "core"
scope: "global"
---

# Lint Attributes

**A lint suppression is a claim about the code beneath it, and the claim is checked.** The suppression is an
`#[expect]`, it carries a `reason`, and it sits on the item that earned it. Derive attributes are owned by
[rust-attrs-derived](rust-attrs-derived.md).

## 1. Fixing Beats Suppressing

A lint is fixed before it is suppressed. Most clippy warnings name a real simplification, and a suppression
trades a two-line edit for a permanent annotation. A suppression is warranted when the lint's premise does
not hold here: the simpler form it suggests is wrong, the complexity it flags is inherent to the design, or a
trait or macro the code does not control requires the pattern. That claim is what the `reason` records.

`just clippy` rejects every warning, so a suppression is the only way a warning survives review. Treat each
one as a design decision, not as a way to get the build green.

Code that guards correctness is never suppressed into compiling.

## 2. A Value Held Only for Its Drop Takes an Underscore

A field or binding that exists for its destructor or its registration side effect is named with a leading
underscore. Rustc's `dead_code` lint skips such names, so the name says what an attribute would have said, in
the place a reader is already looking.

```rust
// ❌ Bad — three lines of attribute to state what the name could state; the reader meets the suppression before the field.
struct SessionFixture {
    ctx: SessionContext,
    #[expect(dead_code, reason = "held only to delete the spill directory on drop")]
    spill_dir: TempDir,
}
```

```rust
// ✅ Good — the underscore marks it write-only at every mention, and the rustdoc says what dropping it does.
struct SessionFixture {
    ctx: SessionContext,
    /// Owns the spill directory; dropping the fixture deletes it.
    _spill_dir: TempDir,
}
```

A `let` binding kept alive for a guard follows the same rule: `let _guard = registry.lock();`. A bare `_` is
a bug here. It drops the value immediately and ends the effect the binding exists for.

The underscore never quiets the lint on an unused item. `dead_code` on a genuinely unused item is the lint
being right, and the fix is deleting the item or `cfg`-gating it ([§6](#6-allow-is-forbidden)). The
underscore is only for a value whose purpose is its `Drop`, and the rustdoc says what that purpose is.

## 3. Suppress With `#[expect]`, Never `#[allow]`

`#[expect]` warns when the lint it names stops firing. `#[allow]` stays behind after the refactor that made
it unnecessary and silently disarms the lint for whatever is written under it later. `#[expect]` is the form
that gets checked.

```rust
// ❌ Bad — this kernel once took nine parameters and now takes four; the attribute still disarms the lint for the next five.
#[allow(clippy::too_many_arguments)]
fn distance_kernel(query: &[f32], candidates: &Float32Array, dimension: Dimension, out: &mut Vec<f32>) {}
```

```rust
// ✅ Good — when the argument count comes down, the attribute becomes a warning and leaves with the cleanup.
#[expect(
    clippy::too_many_arguments,
    reason = "each buffer is borrowed separately so the hot loop holds no struct indirection"
)]
fn distance_kernel(
    query: &[f32],
    candidates: &Float32Array,
    offsets: &[i32],
    nulls: Option<&NullBuffer>,
    dimension: Dimension,
    metric: DistanceMetric,
    out: &mut Vec<f32>,
) {}
```

## 4. Every Suppression Carries a `reason`

The `reason` states why the lint is wrong here, in terms of the design. It never restates the lint's name,
and it is never "clippy false positive". `reason = "the type is complex"` on `clippy::type_complexity`
teaches a reviewer nothing, and the next reader cannot tell whether the claim still holds.

```rust
// ✅ Good — the reason names the design decision the lint argues against, so it is re-evaluated when that decision changes.
#[expect(
    clippy::type_complexity,
    reason = "each argument keeps its array, null mask, and dimension together until the shapes are unified"
)]
fn split_arguments(args: &[ColumnarValue]) -> Result<Vec<(ArrayRef, Option<NullBuffer>, Dimension)>> {}
```

A reason that would read the same on any suppression of that lint is not a reason.

## 5. Scope It to the Narrowest Item

The attribute goes on the item that triggers the lint: the function, the field, the statement. A
module-level `#![allow(clippy::needless_range_loop)]` disarms the lint for code not yet written. Six months
later three more kernels in the file trigger it and nobody knows. At crate level it is disarmed everywhere,
permanently.

```rust
// ✅ Good — the suppression covers exactly the item that earned it.
#[expect(
    clippy::needless_range_loop,
    reason = "one index walks the query, the candidate row, and the weight slice in lockstep"
)]
fn weighted_dot(query: &[f32], candidate: &[f32], weights: &[f32]) -> f32 {}
```

Test code follows the same rule. A suppression a test needs goes on the test, never on the `mod tests` module.

## 6. `#[allow]` Is Forbidden

Outside a macro body, `#[allow]` and `#![allow]` have no accepted use in this workspace. Every suppression is
an `#[expect]`, so one that stops being true becomes a warning. No lint configuration enforces this; review
does.

A lint that fires under one feature combination or under `cfg(test)` only is the tempting case. A bare
`#[expect]` warns in the configurations where the lint does not fire. Scope the expectation to the
configuration with `cfg_attr`.

```rust
// ❌ Bad — the `#[allow]` also covers the build where the lint would have been right.
#[allow(dead_code, reason = "unused when the simd-kernels feature is off")]
struct LaneBuffer { /* ... */ }
```

```rust
// ✅ Good — the expectation is gated to the configuration that produces the lint, and still checked there.
#[cfg_attr(
    not(feature = "simd-kernels"),
    expect(dead_code, reason = "only constructed by the SIMD kernel path")
)]
struct LaneBuffer { /* ... */ }
```

A `cfg_attr` that is hard to write is usually the lint being right. An item dead in one configuration goes
behind the same `cfg` as its users; the warning and the attribute leave together.

The one exception is an attribute inside a `macro_rules!` body. The body emits the same attribute for every
invocation, and the lint fires for only some of them. An `#[expect]` there is unfulfilled wherever the
generated item is used, and no `cfg` can key on what a caller does.

```rust
// ✅ Good — `#[allow]` inside the expansion, with a reason saying the lint depends on the invocation.
macro_rules! option_newtype {
    ($name:ident $inner:ty) => {
        impl $name {
            #[allow(dead_code, reason = "generated accessor; not every option newtype reads back")]
            pub fn get(&self) -> &$inner {
                &self.0
            }
        }
    };
}
```

The exception covers only an attribute inside a macro body whose firing varies by invocation, not the
definition site, the invoking modules, or nearby code. An `#[allow]` anywhere else is a defect. Converting it
to `#[expect]` either succeeds or proves the suppression was stale.

## Checklist

Before committing code, verify:

- [ ] The lint was considered on its merits, and fixing it was rejected for a stated design reason
- [ ] A value held only for its `Drop` or registration effect is underscored, never `dead_code`-suppressed,
      and its rustdoc says what dropping it does
- [ ] The suppression uses `#[expect]`; no `#[allow]` or `#![allow]` was added outside a macro body
- [ ] The attribute carries `reason = "..."` saying why the lint is wrong here, never what the lint is
- [ ] The reason is specific to this site; it would not read identically on any other suppression
- [ ] The attribute is on the item that triggers the lint, never on a module or the crate
- [ ] A conditionally-firing lint is handled with `cfg_attr(..., expect(...))`, never by downgrading to
      `#[allow]`
- [ ] Any `#[allow]` sits inside a macro expansion whose lint varies by invocation, and says so in its
      `reason`
- [ ] No suppression hides an `unwrap`, `expect`, or `panic!` on a production path

## References

- [rust-attrs-derived](rust-attrs-derived.md) - Related: The other attribute family, and its ordering rules
- [rust-docs](rust-docs.md) - Related: A `reason` is documentation and states why, under the same voice
- [principle-least-surprise](principle-least-surprise.md) - Foundation: A suppression that outlives its cause
  makes the code behave differently from what the lint gate promises

## External References

- [The Rust Reference — Lint check attributes](https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-check-attributes)
