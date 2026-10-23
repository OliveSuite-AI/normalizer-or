# Robustness testing and replay

The test layers have different jobs:

- Shared fixtures freeze reviewed text, source spans, and diagnostic behavior.
- Independent oracles exercise finite numeric/calendar domains and notation variants.
- Rust/Python conformance compares every result field and mapped error record.
- Properties generate/shrink mutations and check source/policy invariants.
- Limits, cancellation, concurrency, locales, and bounded stress check engineering behavior.

These are text-contract and engineering checks. The Odia word oracle remains an
alpha contract requiring native-speaker/audio acceptance before production.

## PR profile

From the checkout root, after installing the exact built wheel into the chosen
Python environment:

```sh
cargo test --locked --all-features
cargo test --locked --no-default-features
.venv/bin/python -m pytest -q
cargo build --locked --features serde --example contract-probe
.venv/bin/python -m scripts.robustness --seed 1729
```

Properties default to 512 cases and seed 1729. CI explicitly sets those values.
The conformance runner includes all shared inputs across three policies, 0–99
number readings in three digit scripts, scale/sign boundaries, all 1440 clocks,
Gregorian month/leap-century boundaries, pairwise variations, full three-way
money/wrapper/list cases, paise/precision/units, invalid mutations, hints, cleanup,
invalid input, and cancellation. The finite oracle uses authored fixtures and
Python calendar arithmetic; it never imports production vocabulary or renderers.

The pairwise axes are digit script, wrapper, spacing, prefix prose, suffix prose,
and notation family. A separate test verifies that every pair of axis values
is covered and that a seed replays exactly. Cases with unclear/deferred grammar
stay unresolved; adding a variation is not permission to expand language support.

## Deep profile and failure replay

POSIX example (PowerShell: set `$env:PROPTEST_CASES` and `$env:PROPTEST_RNG_SEED`):

```sh
PROPTEST_CASES=10000 PROPTEST_RNG_SEED=8675309 cargo test --locked --all-features
.venv/bin/python -m scripts.robustness --seed 8675309 --deep --stress --output target/robustness-deep
```

A scheduled/manual workflow runs four property seeds: 1729, 8675309, 20261023,
and 4294967291. Deep conformance exercises the full variation product. The stress
worker runs in a child process with a 30-second hang timeout and records timings
without speed pass/fail thresholds. Patterns include nested wrappers, repeated
markers, numeric runs, combining marks, and electronic paths at increasing sizes
up to the public 32 KiB input limit. Reaching a documented output/work limit is
an accepted outcome; invalid-input/internal errors on valid stress inputs are not.

On failure retain:

- `target/robustness/metadata.json`: seed, platform, Python/Rust versions, tested binary hashes.
- `requests.jsonl`, `rust-results.jsonl`, `rust-stderr.txt`, `failures.json`: exact requests/results and failed case IDs.
- Proptest's `tests/*.proptest-regressions` and `src/**/*.proptest-regressions`: minimized regression seeds, plus the failing source/seed in the test log.
- `stress.json` and `summary.json` on successful stress/conformance completion.

Use the same seed/profile/toolchain to replay. For one shrunk property, filter by
its test name: `cargo test --test properties single_mutations_keep_complete_owners`.
The runner accepts `--probe /absolute/path/to/contract-probe` for alternate build
locations. Preserve the original failure artifacts before rerunning into the same
output directory. Logs contain synthetic/public test inputs only.

## Shared fixture contract

`tests/fixtures/odia.json` keeps its existing shorthand entries. Every entry has
a stable unique `id`. `issue_spans` is a list and may contain multiple original
UTF-8 byte ranges; absent ranges retain the legacy whole-input default.

New detailed cases use `expect.preserve`, `expect.reject`, and `expect.fallback`.
Each expected object selects required canonical result fields. Object fields
are checked recursively; arrays require exact length and order. For diagnostics,
include every expected range/category and fallback class/reason/strategy. For
speech goldens include exact normalized_text. Cases may also freeze segment
ranges, text, kind, and rule ID. Reject diagnostics must equal Preserve diagnostics.
The canonical exporter also independently checks original grapheme partitions.

Author expectations before running the engine. Never capture actual output as a
new golden or weaken a failure to make a run green. Investigate mismatches against
the documented contract; fix violations and retain a minimized shared regression.

## Portability and limits

CI retains Linux/macOS/Windows and CPython 3.11–3.14 wheel testing. Locale tests
spawn a child with UTF-8 mode and locale coercion disabled. Conformance checks
hints and errors through the installed native extension, not a source-tree mock.

Input and hint bounds are checked below/at/above the limit. Internal accounting
checks isolate exact result/work thresholds; public inputs may hit the output
budget before the candidate budget. Concurrency tests use barriers, accepting
calls that completed before cancellation and requiring later token reuse to fail.
A valid short deadline may expire on a busy host; no hard cancellation latency
or ordinary unit-test timing assertion is made.
