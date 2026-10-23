# Development

This repository implements Odia written-text cleanup and speech normalization.
Keep linguistic expectations independently authored; never regenerate goldens
just to make tests pass. Source attribution is in THIRD_PARTY_NOTICES.md.

The core uses Rust 1.99 (declared minimum 1.94); the Python wrapper uses PyO3 and
maturin. Language rules live only in Rust: `engine` owns bounded recognition,
policies and output, `numerals` owns exact numbers, `vocabulary` owns Odia tables,
and `source_map` projects NFC recognition to original UTF-8/grapheme coordinates.
The original public API records and cooperative controls remain in `src/api`.

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo test --locked --no-default-features
cargo doc --locked --no-deps --all-features
uv venv --python 3.13 .venv
uv pip install --python .venv/bin/python -c bindings/python/requirements-dev.txt maturin pytest ruff mypy
.venv/bin/maturin build --release --locked --manifest-path bindings/python/Cargo.toml --interpreter .venv/bin/python --out target/wheels
# Install the exact produced wheel, then test the installed package:
uv pip install --python .venv/bin/python target/wheels/normalizer_or-*.whl
.venv/bin/python -m pytest bindings/python/tests -q
.venv/bin/ruff check bindings/python scripts
.venv/bin/ruff format --check bindings/python scripts
.venv/bin/mypy bindings/python/python --check-untyped-defs
python3 scripts/check_vocabulary.py
```

`tests/fixtures/odia.json` is shared by Rust and installed-Python tests. Add
positive, invalid, and mixed-context cases with source-range expectations.
Property tests exercise arbitrary Unicode, determinism, numeric equivalence,
partitions, and cleanup idempotence. Keep engineering checks independent of
linguistic correctness; native-speaker/audio review is still a release gate.
No unsafe code, floating-point money, network runtime, global case folding,
English transliteration, or inference from already-emitted speech.

Merged PRs to `main` publish through the release workflow after CI succeeds.
The workflow applies an automatically allocated patch version to its build
checkout, tests the complete wheel matrix and sdist, and uses PyPI Trusted
Publishing. See [Python releases](docs/publishing.md) for one-time account setup,
manual build/TestPyPI runs, versioning, and recovery. Production speech quality
still needs linguistic sign-off; automated releases retain the alpha status.

## Robustness and conformance

See [the testing guide](docs/testing.md) for PR/deep profiles, replay commands,
fixture expectations, and artifact interpretation. Build the test-only exporter
before comparing the installed Python wheel with Rust:

```sh
cargo build --locked --features serde --example contract-probe
.venv/bin/python -m scripts.robustness --seed 1729
```

The default property profile uses 512 examples and seed 1729. Both are overridden
by PROPTEST_CASES and PROPTEST_RNG_SEED; no property suite fixes a lower count.
Generated outputs are ignored artifacts under target, never approved goldens.
