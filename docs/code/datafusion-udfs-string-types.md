---
name: "datafusion-udfs-string-types"
description: "String argument types for DataFusion scalar UDFs. Load when defining or reviewing a scalar UDF that accepts strings"
type: "core"
scope: "global"
---

# String Arguments Without Casts

## Rule

A scalar UDF that accepts strings supports `Utf8`, `LargeUtf8`, and `Utf8View` directly. Declare
its signature to accept these logical types without coercing one representation into another. At execution, handle each
physical representation in its matching Arrow array form and borrow text as `&str`. Preserve nulls and
treat scalar inputs consistently with array inputs.

Do not cast every input to one string representation just to simplify row access; that can materialize
data and add work without changing the function's meaning.

~~~rust
// ❌ Bad — normalize every accepted string representation by casting.
let text = cast_to_utf8(argument)?;

// ✅ Good — read text in its accepted representation without casting.
let text: Option<&str> = string_value_at(argument, row)?;
~~~

## Why It Matters

Arrow string types use different offset widths or layouts, but each can expose borrowed text. Supporting
all three lets callers use their existing columns without casts or copies.

## Pragmatism Caveat

Accept these types when the argument is semantically a string. Do not treat another logical type as a
string merely because its values can be rendered as text.

## Checklist

- [ ] The signature accepts `Utf8`, `LargeUtf8`, and `Utf8View` without coercing between them
- [ ] Each representation is read through its matching Arrow array form
- [ ] Row access borrows text and preserves null as `None`
- [ ] Scalar and array inputs follow the same value semantics
