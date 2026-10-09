# Acting on mutation results

Use this reference only for an authorized test-effectiveness investigation with `cargo-mutants`.
Inspect the installed tool's help and the output directory from the actual run; do not assume Python
mutmut statuses, marker-based tiers, or its file layout.

A caught mutant caused a test to fail. A survivor changed the code while selected tests still passed;
it may indicate a missing assertion, an equivalent change, or code no caller can observe. A build
failure is not evidence that the tests caught the behavioral fault. Treat timeouts and tool failures
separately from successful detection.

For a survivor, reproduce the affected behavior with finite Arrow inputs and determine whether the
mutation changes the public contract. Nulls, zero norms, slices, mismatched dimensions, and a second
row after an error-prone branch often expose a missing assertion.

Add a test only for meaningful caller-visible behavior. Equivalent mutations need an explanation,
not a brittle test or a blanket exclusion. Remove dead code when that is the correct fix. Follow the
installed tool's documented options to rerun the affected mutation or file and confirm the result.

Report the selected scope, tested mutations, survivors, timeouts, and any uncompiled mutations
separately. Coverage says which code ran; mutation testing asks whether assertions noticed a fault.
Do not chase a perfect score by asserting incidental implementation details.
