# Odia performance baseline

```sh
cargo bench --locked --features serde --bench latency
```

The benchmark runs 8000 operations across short Odia/English mixed text, Indian
INR grouping, and temporal notation, using both speech and cleanup. Initialization
is outside the measured loop. Outputs are consumed with black_box. It reports a
host-specific elapsed duration.
Semantic acceptance is tested separately in the shared fixture suite.

Before a performance-sensitive release, collect
uncontended optimized measurements on the target platform and retain machine,
compiler, input, and revision details. Resource-limit and randomized partition
checks remain independent of short-input timing.
