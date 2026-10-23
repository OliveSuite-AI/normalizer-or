<div align="center">

# normalizer-or

### Odia text, ready for speech.

A Rust engine with a typed Python API for Odia speech normalization and written-text cleanup.

**Author: Sambit Sekhar**

[![CI](https://github.com/OliveSuite-AI/normalizer-or/actions/workflows/ci.yml/badge.svg)](https://github.com/OliveSuite-AI/normalizer-or/actions/workflows/ci.yml)
[![Publish](https://github.com/OliveSuite-AI/normalizer-or/actions/workflows/release.yml/badge.svg)](https://github.com/OliveSuite-AI/normalizer-or/actions/workflows/release.yml)
[![Robustness](https://github.com/OliveSuite-AI/normalizer-or/actions/workflows/robustness.yml/badge.svg)](https://github.com/OliveSuite-AI/normalizer-or/actions/workflows/robustness.yml)
[![PyPI](https://img.shields.io/pypi/v/normalizer-or)](https://pypi.org/project/normalizer-or/)
[![Python](https://img.shields.io/badge/Python-3.11%E2%80%933.14-3776AB?logo=python&logoColor=white)](https://github.com/OliveSuite-AI/normalizer-or/blob/main/bindings/python/README.md)
[![Rust](https://img.shields.io/badge/Rust-1.94%2B-CE422B?logo=rust)](https://github.com/OliveSuite-AI/normalizer-or/blob/main/Cargo.toml)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue)](https://github.com/OliveSuite-AI/normalizer-or/blob/main/LICENSE)
[![Typed](https://img.shields.io/badge/typing-py.typed-22863A)](https://github.com/OliveSuite-AI/normalizer-or/tree/main/bindings/python/python/normalizer_or)
[![Status](https://img.shields.io/badge/status-alpha-orange)](https://github.com/OliveSuite-AI/normalizer-or/blob/main/src/data/provenance.md)

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/OliveSuite-AI/normalizer-or/blob/main/bindings/python/examples/normalizer_or_colab.ipynb)

[Quick start](#quick-start) · [Examples](#examples) · [Python API](https://github.com/OliveSuite-AI/normalizer-or/blob/main/bindings/python/README.md) · [Formats](https://github.com/OliveSuite-AI/normalizer-or/blob/main/docs/normalization.md) · [Publishing](https://github.com/OliveSuite-AI/normalizer-or/blob/main/docs/publishing.md)

</div>

```text
ଆଜି ୨୫ kg ଚାଉଳ।  →  ଆଜି ପଚିଶ କିଲୋଗ୍ରାମ ଚାଉଳ।
₹25.50             →  ପଚିଶ ଟଙ୍କା ପଚାଶ ପଇସା
```

Runs offline with **no speech model, API key, network service, or runtime Python dependencies**.

## What it does

| Operation | Purpose | Example |
|---|---|---|
| `normalize` | Expand supported notation into Odia speech text | `୨୫ kg` → `ପଚିଶ କିଲୋଗ୍ରାମ` |
| `clean` | Apply NFC and optional digit/whitespace conversion | `123` → `୧୨୩` with `digits="odia"` |

- **Odia and ASCII digits:** mixed digits, Indian/international grouping, lakh/crore scales, signs and exact decimals.
- **Everyday notation:** INR/paise, percentages, ten unit labels, Gregorian dates and 24-hour clocks.
- **Inspectable results:** completeness, source segments, issues and fallback assumptions with original UTF-8 byte offsets.
- **Bounded execution:** input/output limits, cooperative cancellation and deadlines; reuse one normalizer across threads.
- **Markdown and formula speech:** Python's `strip_markdown()` removes common formatting and preserves math spans; elementary algebra formulas receive Odia symbolic readings.

English prose and names stay as written. URLs, email addresses and opaque identifiers
are protected from partial numeric expansion. Supported elementary algebra is
spoken; unsupported formulas stay intact as unresolved expressions. Coverage and spellings are **alpha
defaults** pending native-speaker/audio review; passing tests do not establish speech quality.

## Quick start

### Install

After the first successful PyPI release:

```sh
pip install normalizer-or
# Or use uv in your virtual environment:
uv pip install normalizer-or
# Or add it to a uv-managed project:
uv add normalizer-or
```

PyPI publication requires the one-time [publisher setup](https://github.com/OliveSuite-AI/normalizer-or/blob/main/docs/publishing.md).
The PyPI version badge populates after the first upload. Until then, install
from GitHub with Git and Rust available:

```sh
python -m pip install "git+https://github.com/OliveSuite-AI/normalizer-or.git#subdirectory=bindings/python"
# Or:
uv pip install "git+https://github.com/OliveSuite-AI/normalizer-or.git#subdirectory=bindings/python"
```

Compatible release wheels need no Rust compiler. Source builds require Rust.
Supported interpreters are CPython 3.11–3.14. From a local checkout, use
`python -m pip install ./bindings/python`.

### Normalize and clean

```python
from normalizer_or import Normalizer

n = Normalizer()
spoken = n.normalize("ଆଜି ୨୫ kg ଚାଉଳ।")
assert spoken.normalized_text == "ଆଜି ପଚିଶ କିଲୋଗ୍ରାମ ଚାଉଳ।"
assert spoken.complete

written = n.clean("  ଓଡ଼ିଆ 123\n", digits="odia", whitespace="collapse")
assert written.normalized_text == " ଓଡ଼ିଆ ୧୨୩ "
```

### Choose how to handle unresolved text

| Policy | Result |
|---|---|
| `preserve` — default | Retain unresolved source; return `complete=False` and issues |
| `reject` | Raise `NormalizationError` with diagnostics |
| `fallback` | Read the source literally; record assumptions in `fallbacks` |

```python
result = n.normalize("29/02/2025", ambiguity_policy="preserve")
assert not result.complete
assert result.normalized_text == "29/02/2025"
print(result.issues)
```

Fallback does not repair dates or certify identifiers. Hint and diagnostic ranges
count **original UTF-8 bytes**, not Python characters. Cleanup is a separate call;
speech normalization never silently rewrites the input first.

## Examples

| Input | Speech output |
|---|---|
| `1୨3` | ଏକ ଶହ ତେଇଶ |
| `1,00,000` | ଏକ ଲକ୍ଷ |
| `1,00,00,000` | ଏକ କୋଟି |
| `₹25.50` | ପଚିଶ ଟଙ୍କା ପଚାଶ ପଇସା |
| `12.5%` | ବାର ଦଶମିକ ପାଞ୍ଚ ପ୍ରତିଶତ |
| `09:30` | ନଅ ଘଣ୍ଟା ତିରିଶ ମିନିଟ |

Try the [Colab notebook](https://colab.research.google.com/github/OliveSuite-AI/normalizer-or/blob/main/bindings/python/examples/normalizer_or_colab.ipynb)
in a fresh CPU runtime, or run all seven [self-checking Python scripts](https://github.com/OliveSuite-AI/normalizer-or/blob/main/bindings/python/examples/README.md):

```sh
python bindings/python/examples/run_all.py
```

Examples cover speech, cleanup, all three policies, all four hint kinds, UTF-8
source inspection, JSON, errors, cancellation, threaded batches and JSONL files.

## Rust

Use a local path dependency until the Rust crate is published:

```toml
[dependencies]
normalizer-or = { path = "/path/to/normalizer-or" }
```

```rust
use normalizer_or::{NormalizeError, NormalizeOptions, Normalizer};

fn main() -> Result<(), NormalizeError> {
    let n = Normalizer::new()?;
    let result = n.normalize("୨୫ INR", &NormalizeOptions::default())?;
    assert_eq!(result.normalized_text(), "ପଚିଶ ଟଙ୍କା");
    Ok(())
}
```

## Tests and releases

CI checks the Rust core, minimum Rust version, Python bindings and independent
Rust/Python conformance records. Scheduled robustness jobs exercise additional
seeds and long inputs. The [testing guide](https://github.com/OliveSuite-AI/normalizer-or/blob/main/docs/testing.md) explains replaying failures.

A merged PR to `main` is published **after its CI passes**. Release automation
allocates a new patch version, builds and tests 16 wheels plus an sdist, validates
metadata, and uploads through PyPI Trusted Publishing. Manual build-only and
TestPyPI runs are available. See [release setup and versioning](https://github.com/OliveSuite-AI/normalizer-or/blob/main/docs/publishing.md).

## Documentation

| Guide | Contents |
|---|---|
| [Python API](https://github.com/OliveSuite-AI/normalizer-or/blob/main/bindings/python/README.md) | Results, hints, controls and errors |
| [Normalization contract](https://github.com/OliveSuite-AI/normalizer-or/blob/main/docs/normalization.md) | Formats, boundaries, policies and limits |
| [Runnable examples](https://github.com/OliveSuite-AI/normalizer-or/blob/main/bindings/python/examples/README.md) | Scripts and Colab notebook |
| [Publishing](https://github.com/OliveSuite-AI/normalizer-or/blob/main/docs/publishing.md) | PyPI setup, automatic versions and release recovery |
| [Development](https://github.com/OliveSuite-AI/normalizer-or/blob/main/CONTRIBUTING.md) | Build and verification commands |
| [Robustness](https://github.com/OliveSuite-AI/normalizer-or/blob/main/docs/testing.md) | Oracles, properties and cross-language checks |
| [Performance](https://github.com/OliveSuite-AI/normalizer-or/blob/main/PERFORMANCE.md) | Measurement guidance |
| [Provenance](https://github.com/OliveSuite-AI/normalizer-or/blob/main/src/data/provenance.md) | Vocabulary and review status |

Licensed under [Apache-2.0](https://github.com/OliveSuite-AI/normalizer-or/blob/main/LICENSE). Required source and dependency credits remain
in [third-party notices](https://github.com/OliveSuite-AI/normalizer-or/blob/main/THIRD_PARTY_NOTICES.md).
