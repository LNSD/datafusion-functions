---
name: "principle-inversion-of-control"
description: "Inversion of Control — accept dependencies as parameters instead of constructing them. Load when designing UDFs and kernels, wiring collaborators, or making code testable without a full SessionContext"
type: "principle"
scope: "global"
---

# Inversion of Control (Dependency Injection)

## Rule

A unit declares what it needs. Collaborators arrive as function parameters, as struct fields set by the
constructor, or as generic parameters. The unit does not construct them.

Inject when either signal fires:

1. **It performs I/O or reads ambient state.** It reads files, the environment, the clock, or a global
   session. Tests must be able to substitute it.
2. **It varies by context.** Production and tests, or two configurations, need different instances.

When neither fires, construct inline. A `Vec`, a `Float32Builder`, and a pure helper in the same module are
never injected.

Rust gives two shapes for the seam. A generic parameter (`M: Metric`) monomorphizes and is the default for a
collaborator fixed at construction. A trait object (`Arc<dyn ScalarUDFImpl>`) is for a set assembled at
runtime or stored heterogeneously, such as the list of functions a package registers.

## Examples

1. **Inject the effect, not the machinery that performs it**
   A UDF needs configuration options such as a default metric. It takes them from DataFusion's config,
   which arrives in `ScalarFunctionArgs`, instead of reading them itself.

```rust
// ❌ Bad — the env var is read inside the kernel call, so every test must mutate process state.
fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
    let metric = std::env::var("VECTOR_DEFAULT_METRIC").unwrap_or_else(|_| "l2".to_owned());
    distance(&args.args, metric.parse()?)
}
```

```rust
// ✅ Good — the metric is a constructor argument; tests build the UDF with the metric they assert on.
pub struct VectorDistance { signature: Signature, metric: Metric }

impl VectorDistance {
    pub fn new(metric: Metric) -> Self {
        Self { signature: Signature::any(2, Volatility::Immutable), metric }
    }
}
```

2. **Register into the registry you are given**
   Registration takes the registry as a parameter. The package never builds its own `SessionContext`.

```rust
// ❌ Bad — the session is built inside, so callers cannot add functions to the context they already have.
pub fn vector_session() -> SessionContext {
    let ctx = SessionContext::new();
    ctx.register_udf(ScalarUDF::from(VectorDistance::new(Metric::L2)));
    ctx
}
```

```rust
// ✅ Good — any `FunctionRegistry` works; tests pass a fresh context, applications pass theirs.
pub fn register_all(registry: &mut dyn FunctionRegistry) -> Result<()> {
    registry.register_udf(Arc::new(ScalarUDF::from(VectorDistance::new(Metric::L2))))?;
    Ok(())
}
```

## Why It Matters

Without injection a unit's dependencies are invisible. Nothing in `invoke_with_args(args)` says it reads the
environment. With injection the signature is the dependency list, and a reviewer sees the blast radius
without reading the body.

Injection is the only seam Rust gives. There is no runtime patching: a `SessionContext::new()` inside a body
cannot be replaced by a test. A collaborator that is not in the signature is reachable only through an
end-to-end SQL test, and the awkward cases go uncovered.

Injection keeps the dependency graph acyclic. A kernel module never depends on the UDF wrappers or on
registration, because implementations arrive as values the caller composes.

## Pragmatism Caveat

Do not inject for its own sake. A UDF may build its own `Signature` in `new`: it lives as long as the UDF and
no other implementation exists. Injecting a value one level up (a metric, a dimension, an option) is often
better than injecting an object.

Do not use `Arc<dyn Trait>` where a generic parameter fits. Do not introduce a trait with one implementation
and no test double.

When an ambient dependency is deliberately read inside, the seam that replaces it exists and is named in a
comment. An ambient dependency with no seam is always wrong.

## Checklist

Before committing code, verify:

- [ ] Every I/O or ambient collaborator is a parameter, a constructor argument, or a generic parameter, never
      constructed inline
- [ ] Effects a test needs to control are parameters (an option, a metric, a registry), with the real value
      supplied by the caller
- [ ] `std::env`, the current directory, and the clock are never read inside a UDF or kernel
- [ ] Registration takes a `FunctionRegistry` or `SessionContext` from the caller, never builds its own
- [ ] The unit can be exercised directly with hand-built Arrow arrays, without running SQL
- [ ] Generic parameters are used for fixed collaborators; trait objects only where the set is
      runtime-assembled
- [ ] Where a dependency is constructed internally, the injectable seam is documented
- [ ] No trait exists solely to wrap a single implementation that nothing substitutes

## References

- [principle-law-of-demeter](principle-law-of-demeter.md) - Related: Injecting the exact value needed removes
  the reason to navigate an object graph
- [principle-single-responsibility](principle-single-responsibility.md) - Related: A unit with one
  responsibility has few enough collaborators to inject them all
- [principle-open-closed](principle-open-closed.md) - Related: The injection seam and the extension point are
  the same trait

## External References

- [Inversion of Control (Kent C. Dodds)](https://kentcdodds.com/blog/inversion-of-control)
- [Inversion of Control Containers and the Dependency Injection pattern (Martin Fowler)](https://martinfowler.com/articles/injection.html)
- [Beginner's Guide to Inversion of Control (HackerNoon)](https://hackernoon.com/beginners-guide-to-inversion-of-control)
