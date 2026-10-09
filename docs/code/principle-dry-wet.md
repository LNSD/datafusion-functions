---
name: "principle-dry-wet"
description: "DRY vs WET — deduplicate knowledge, tolerate coincidental similarity. Load when extracting shared helpers, creating abstractions, or reviewing duplicated-looking code"
type: "principle"
scope: "global"
---

# DRY/WET Balance (Don't Repeat Yourself vs. Write Everything Twice)

## Rule

Every piece of knowledge has one authoritative representation: a formula, a serialized encoding, a type
mapping, a policy. Deduplicate knowledge. Alike-looking code serving separate concerns stays apart.

Four checks precede a shared helper:

1. **Same knowledge, not same shape.** Two loops for a scalar and an array argument are two facts.
2. **Rule of Three.** Extract on the third occurrence, when the varying parts are visible.
3. **Inline test.** A parameter or conditional whose only job is to pick a caller's behavior marks the
   wrong abstraction. Inline it.
4. **Serialized-boundary test.** Across a serialized boundary the shared fact is the encoding. Each side keeps
   its own type and implements only the conversion direction it performs. The conversion is declared in the
   consuming package, beside the local type.

Write out the copy each side needs before centralizing. A copy of fields and serde derives alone is a shape,
and a shape earns no dependency edge.

Duplication is cheaper than the wrong abstraction.

A shared fact no package can own gets a new package. That package owns the concept by charter, holds no
caller's type, and pulls in no dependency. Neither a re-export chain nor the package everything already
depends on is a home.

## Examples

1. **Same knowledge, one representation**
   The Arrow type a vector argument is coerced to is one fact. The signature, the return type, and the
   kernel all need it.

```rust
// ❌ Bad — three places re-derive the item field; one missed copy makes coercion and the kernel disagree.
let item = Arc::new(Field::new("item", DataType::Float32, true));
let item = Arc::new(Field::new_list_field(DataType::Float32, true));
let item = Arc::new(Field::new("element", DataType::Float32, false));
```

```rust
// ✅ Good — one function owns the vector type; a change of item type is a one-line change.
pub fn vector_type(dimension: Dimension) -> DataType {
    DataType::FixedSizeList(Arc::new(Field::new_list_field(DataType::Float32, true)), dimension.get())
}
```

2. **Coincidental similarity, two homes**
   Two lists of metric names with the same shape encode two policies.

```rust
// ❌ Bad — one list for two policies; adding a metric that is not a true distance also makes it indexable.
pub const METRICS: &[&str] = &["l2", "cosine", "inner_product", "l1"];
```

```rust
// ✅ Good — two facts, two homes; an edit to one cannot change the other's behavior.
const ACCEPTED_METRICS: &[&str] = &["l2", "cosine", "inner_product", "l1"];
const TRUE_DISTANCES: &[&str] = &["l2", "l1"];
```

3. **Wrong abstraction, inlined**
   A scalar UDF and an aggregate UDF both walk vector rows. The bodies look alike. The aggregate keeps state
   across batches and merges partials; the scalar emits one value per row.

```rust
// ❌ Bad — a mode flag selects the caller's behavior; every aggregate change risks the scalar function.
fn walk_vectors(array: &FixedSizeListArray, mode: Mode, state: Option<&mut Accumulated>) -> ArrayRef {}
```

```rust
// ✅ Good — one kernel per function kind; an aggregate change cannot reach the scalar tests.
pub fn norms(array: &FixedSizeListArray) -> Float32Array {}
pub fn accumulate_centroid(state: &mut CentroidState, array: &FixedSizeListArray) {}
```

4. **Across a serialized boundary, share the encoding**
   An aggregate serializes its partial state with `state()` and another partition reads it back in
   `merge_batch`. The bytes couple them. The type owning the in-memory state never learns the stored form
   exists. An inverse impl written for symmetry is dead code.

```rust
// ❌ Bad — the in-memory struct is the wire form; a field rename compiles and breaks merging old partials.
#[derive(Serialize, Deserialize)]
pub struct CentroidState {
    pub sum: Vec<f64>,
    pub count: u64,
}
```

```rust
// ✅ Good — the writer declares its encoded form and its one direction beside the local type.
pub struct EncodedCentroid {
    sum: ScalarValue,
    count: ScalarValue,
}

impl From<&CentroidState> for EncodedCentroid {
    fn from(state: &CentroidState) -> Self {
        let sum = ScalarValue::List(ScalarValue::new_list_from_iter(state.sum_values(), &DataType::Float64, true));
        Self { sum, count: ScalarValue::UInt64(Some(state.count())) }
    }
}
```

## Why It Matters

Duplicated knowledge needs coordinated edits. One missed copy of a vector's item field makes the declared
return type disagree with the array the kernel builds. DataFusion reports it only at execution, if at all.

A shared abstraction over two shapes couples them. A change to the aggregate touches code the scalar
function's tests cover. The flags grow until the function's behavior is the sum of its branches. Undoing that
costs more than never building it.

A type shared across a serialized boundary asserts that both sides always agree on its layout. Partial
states and persisted values outlive a release. The compiler checked field names; the skew is in the bytes.
Two types leave each side free to handle the skew.

## Pragmatism Caveat

The Rule of Three is a heuristic. Two occurrences of an unmistakable fact (a function name registered with
SQL, a constant from a spec) can be extracted at once. Three occurrences serving three function kinds stay
apart.

A small helper duplicated across a module or package boundary is usually correct. Promoting a four-line
conversion helper to `pub(crate)` or a shared package widens an API surface and ties the two modules
together. Prefer the private copy.

Deliberate duplication carries a comment saying the similarity is coincidental. An extracted helper is named
for the shared concept (`vector_type`), never the shared shape (`walk_vectors`, `handle_thing`). An
undocumented decision either way is always wrong.

## Checklist

Before committing code, verify:

- [ ] Extracted code encodes one fact, not one syntax shape
- [ ] No shared helper takes a flag, mode, or `kind` parameter that exists only to select a caller's behavior
- [ ] Function names, Arrow type mappings, serialized encodings, and spec values have exactly one definition
- [ ] Similar-looking code that serves two function kinds or two policies stays in two places
- [ ] Deliberate duplication carries a comment saying the similarity is coincidental
- [ ] Cross-package hoisting is justified by shared knowledge, never by line count
- [ ] A value exchanged across a serialized boundary couples the two sides through its encoding, with no
      struct both sides depend on
- [ ] Each side of a boundary implements only the conversion direction it performs; no inverse impl exists
      for symmetry alone
- [ ] The local copy each side would need was written out before centralizing was chosen
- [ ] A shared fact no package owns got a new package, and no home in the package everything already
      depends on

## References

- [principle-single-responsibility](principle-single-responsibility.md) - Related: A helper serving two
  concerns is the wrong abstraction by definition
- [principle-open-closed](principle-open-closed.md) - Related: Registries and extension points hold shared
  behavior; flags do not
- [principle-least-surprise](principle-least-surprise.md) - Related: An abstraction named for its shape
  surprises every caller
- [principle-symmetry](principle-symmetry.md) - Related: Make near-duplicates symmetric first; then it is
  visible whether they are one fact or two
- [principle-rate-of-change](principle-rate-of-change.md) - Related: Two copies that change on different
  schedules are two facts, whatever their shape says

## External References

- [The Wrong Abstraction — Sandi Metz](https://sandimetz.com/blog/2016/1/20/the-wrong-abstraction)
- [DRY is about Knowledge (Verraes)](https://verraes.net/2014/08/dry-is-about-knowledge/)
- [Caught in a Bad Abstraction — Israeli Tech Radar](https://medium.com/israeli-tech-radar/caught-in-a-bad-abstraction-55bfe6634b83)
- [DRY: Most Over-rated Programming Principle — Gordon C](https://gordonc.bearblog.dev/dry-most-over-rated-programming-principle/)
