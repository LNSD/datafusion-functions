# Before and after

Use these examples with the three gates in [SKILL.md](SKILL.md). Examples illustrate message shape;
they are not claims about an existing defect in this repository.

## Titles

| Narrated edit | Observable consequence |
|---|---|
| `fix: change the list offset calculation` | `fix(datafusion-functions-vector): score sliced vector columns correctly` |
| `feat: add an expression helper` | `feat(datafusion-functions-vector): score vectors through the DataFrame API` |
| `chore: copy agent files` | `chore(skills): use Rust checks for vector crate development` |

The right side states what a caller or contributor can now do. Naming the edited key or file is the
diff's job.

## A whole message

Narrated:

```text
fix(datafusion-functions-vector): change the cosine branch

Update the zero-norm condition and add a test.

- Add an if statement before division
- Push None when the accumulator is zero
- Add a zero-vector case
```

Intent-focused:

```text
fix(datafusion-functions-vector): return null for undefined cosine scores

Comparing an embedding with a zero vector produced a nonfinite score that callers could mistake for a usable distance. Undefined cosine scores now propagate as SQL nulls.

- Zero vectors produce null instead of a nonfinite distance, so filters do not treat an undefined score as a valid match
- Finite nonzero vectors retain their distance, preserving ranking for usable embeddings
```

Every bullet states an observable consequence. The title alone cannot replace the concrete trigger
in the summary.

## Mechanical changes

```text
chore(deps): update DataFusion to the next compatible patch
```

A dependency maintenance change without another decision can be title-only. Do not pad it with an
invented performance or correctness claim.
