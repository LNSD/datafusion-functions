---
name: "principle-symmetry"
description: "Symmetry — express the same idea the same way; split near-duplicates into identical parts and clearly different parts, one altitude per body. Load when writing something that resembles existing code, or reviewing sibling UDFs, functions, branches, or modules"
type: "principle"
scope: "global"
---

# Symmetry (Express the Same Idea the Same Way)

## Rule

The same idea is expressed the same way everywhere it appears. Two pieces of code that are almost the same
are split so the identical parts are literally identical and the differing parts are the only visible
difference.

1. **One idea, one shape.** Two functions answering the same question take parameters in the same order,
   return the same shape, and name their steps the same way.
2. **Near-duplicates keep an identical skeleton.** Same step order, local names, and error handling. Only
   the lines that must diverge differ.
3. **One level of abstraction per function.** Every statement in a body sits at the same altitude.
4. **Sibling branches carry comparable weight.** Arms and branches of one construct all delegate, or all
   inline.
5. **Paired operations stay paired.** Accumulator `state` and `merge_batch`, register and deregister, to and
   from Arrow sit in the same module, at the same level, in the same vocabulary. `principle-least-surprise`
   owns the inverse's name.
6. **Sibling modules in the same role share a layout.** They expose the same entry points in the same file
   positions.

Symmetry is not deduplication. Two symmetric copies with one visible difference are a good outcome.
`principle-dry-wet` decides whether to extract.

## Examples

1. **One idea, one shape**
   Three helpers that read an argument's vector type.

```rust
// ❌ Bad — the index moves between first and last parameter, and "not a vector" is spelled three ways;
// a caller written against one treated `None` as "any dimension" for the other two.
fn vector_dimension(args: &[DataType], index: usize) -> Result<Dimension>;
fn element_type(index: usize, args: &[DataType]) -> Option<DataType>;
fn vector_nullable(args: &[FieldRef], index: usize) -> Result<Option<bool>>;
```

```rust
// ✅ Good — arguments first, index second, a non-vector argument is a planning error.
fn vector_dimension(args: &[DataType], index: usize) -> Result<Dimension>;
fn vector_element_type(args: &[DataType], index: usize) -> Result<DataType>;
fn vector_nullable(args: &[FieldRef], index: usize) -> Result<bool>;
```

2. **Near-duplicates keep an identical skeleton**
   Two distance UDFs. They stay two functions.

```rust
// ❌ Bad — the same steps in a different order under different names; a fix to the dimension check
// landed in one and was missed in the other for two releases.
let (left, right) = as_vector_pair(&args.args)?;
check_same_dimension(&left, &right)?;
let values = left.iter().zip(right.iter()).map(|(a, b)| cosine_distance(a, b));
// ...
let (lhs, rhs) = as_vector_pair(&args.args)?;
let out = lhs.iter().zip(rhs.iter()).map(|(x, y)| l2_distance(x, y));
check_same_dimension(&lhs, &rhs)?;
```

```rust
// ✅ Good — identical skeleton; the one line that differs is the one that must.
let (left, right) = as_vector_pair(&args.args)?;
check_same_dimension(&left, &right)?;
let values = left.iter().zip(right.iter()).map(|(a, b)| cosine_distance(a, b));
// ...
let (left, right) = as_vector_pair(&args.args)?;
check_same_dimension(&left, &right)?;
let values = left.iter().zip(right.iter()).map(|(a, b)| l2_distance(a, b));
```

3. **One altitude, comparable branches**
   A dispatch over the argument's `ColumnarValue`.

```rust
// ❌ Bad — one arm states what happens and the other states how; a second UDF copied the short arm
// and forgot the null check.
match arg {
    ColumnarValue::Array(array) => self.norm_array(array),
    ColumnarValue::Scalar(scalar) => {
        let ScalarValue::FixedSizeList(list) = scalar else { return exec_err!("expected a vector") };
        if list.is_null(0) { return Ok(ColumnarValue::Scalar(ScalarValue::Float32(None))) }
        Ok(ColumnarValue::Scalar(ScalarValue::Float32(Some(l2_norm(list.value(0))))))
    }
}
```

```rust
// ✅ Good — every arm states an intention; the null handling lives with the other conversions.
match arg {
    ColumnarValue::Array(array) => self.norm_array(array),
    ColumnarValue::Scalar(scalar) => self.norm_scalar(scalar),
}
```

## Why It Matters

Asymmetry is paid on every read. A reader who has understood one member of a pair should skip the other. When
the shapes disagree, the reader reads both in full and diffs them by hand. That cost is invisible in a diff
and unbounded over a file's life.

Asymmetry is paid again in bugs. A reviewer's strongest tool is noticing that two things that should match do
not. A fix applied to one UDF and missed in its sibling passes review because nothing looks out of place.
Mixed altitude hides effects from the call site. An unbalanced branch hides a procedure inside a case label.

Symmetry compounds. When every UDF module exposes its struct, `Signature`, kernel, and registration in the
same positions, a reader lands in an unfamiliar function already knowing where to look. That return is
available only while the consistency holds everywhere.

## Pragmatism Caveat

False symmetry is worse than asymmetry. A matching shape claims that the behavior matches, and a reader acts
on the claim. A branch is not padded to balance it. An infallible kernel is not given a `Result` to line up
with its neighbor. A stateless function is not given an `Accumulator` to match an aggregate.

Symmetry is bounded by the seams around it. A DataFusion or Arrow trait dictates its own parameter order and
return shape. The foreign shape is matched at the boundary and the workspace shape everywhere else. Two
variants diverging permanently break their symmetry on purpose, and the cheapest way is a rename, so the
reader stops expecting a pair.

Symmetry broken deliberately carries a comment at the declaration. An undocumented asymmetry is always
wrong: the next reader cannot tell it from the copy nobody updated.

## Checklist

Before committing code, verify:

- [ ] Functions answering the same question take the same parameter order and return the same shape
- [ ] Near-duplicate bodies share step order, local names, and error handling; only the intended lines differ
- [ ] No function body mixes statements that name an intention with statements that perform the mechanism
- [ ] Sibling match arms and branches all delegate or all inline; none hides a procedure
- [ ] Every paired operation (state and merge, register and deregister) sits with its inverse at the same level
- [ ] Modules playing the same role expose the same entry points in the same positions
- [ ] No shape was matched that the behavior does not match, and no branch was padded to balance it
- [ ] Any deliberate asymmetry carries a comment saying why

## References

- [principle-dry-wet](principle-dry-wet.md) - Related: Symmetry makes the difference visible; DRY/WET decides
  whether the pair is one fact to extract
- [principle-least-surprise](principle-least-surprise.md) - Related: Owns the naming contract for paired
  operations; a symmetric shape makes a name's prediction hold
- [principle-single-responsibility](principle-single-responsibility.md) - Related: A body that mixes altitudes
  is usually a function with two responsibilities
- [principle-open-closed](principle-open-closed.md) - Related: A registry stays extensible while every entry
  has the same shape
- [principle-rate-of-change](principle-rate-of-change.md) - Related: Says when a symmetric pair is broken on
  purpose, because the halves move on different schedules

## External References

- [Symmetry, in Kent Beck's Implementation Patterns](https://blog.iterate.no/2012/06/20/programming-like-kent-beck/)
- [Mastering Programming — Kent Beck](https://tidyfirst.substack.com/p/mastering-programming)
- [The Value of Symmetry — Scott Allen](https://odetocode.com/blogs/scott/archive/2011/02/07/the-value-of-symmetry.aspx)
- [Consistency creates cognitive leverage — A Philosophy of Software Design](https://danlebrero.com/2021/02/24/philosophy-of-software-design-summary/)
- [Single Level of Abstraction Principle](https://principles-wiki.net/principles:single_level_of_abstraction)
