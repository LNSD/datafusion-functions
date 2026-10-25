---
name: "pattern-registry"
description: "Centralized package function registration through a registry entry point. Load when adding or changing package registration"
type: "core"
scope: "global"
---

# Registry Pattern for Package Functions

## Rule

A package with related functions exposes one entry point that accepts the relevant registry. Register
the package's functions there and propagate registry errors. Callers can opt into the whole package
without duplicating its function list. Keep individual handles available when callers need selective
registration; the central entry point remains the complete default.

## Examples

```rust
// ❌ Bad — each caller must know and keep the package function list in sync.
registry.register_udf(a_udf())?;
registry.register_udf(b_udf())?;
```

```rust
// ✅ Good — the package owns its list behind one stable entry point.
pub fn register_all(registry: &mut dyn FunctionRegistry) -> Result<()> {
    registry.register_udf(a_udf())?;
    registry.register_udf(b_udf())?;
    Ok(())
}
```

## Why It Matters

The owning package or component is the authority on which functions it provides. A shared entry point
makes new functions available to callers without requiring each one to update a copied list, and
propagating registry failures lets callers handle registration problems.

## Pragmatism Caveat

A package with one function can still offer `register_all` for a consistent integration surface. A helper
that registers only a caller-selected subset is unnecessary when the UDF handles themselves are already
public and directly registrable.

## Checklist

- [ ] The package entry point accepts DataFusion's function registry.
- [ ] Every package UDF is registered by the entry point, with failures propagated.
- [ ] Individual handles remain available only where selective registration is useful.

## References

- [package-datafusion-functions-jsonschema](package-datafusion-functions-jsonschema.md) - Related: Package registration surface
