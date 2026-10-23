# normalizer-or

**Author:** Sambit Sekhar

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/OliveSuite-AI/normalizer-or/blob/main/bindings/python/examples/normalizer_or_colab.ipynb)

Offline Odia text normalization with a Rust core and a typed Python API.
Licensed under Apache-2.0. It needs no speech model, network service, or Python runtime dependencies.

Two operations have separate purposes:

- `normalize`: turn supported numbers, INR amounts, dates, clocks, percentages, and units into Odia speech text.
- `clean`: apply NFC Unicode normalization and optional digit/whitespace conversion to written text.

```text
ଆଜି ୨୫ kg ଚାଉଳ। → ଆଜି ପଚିଶ କିଲୋଗ୍ରାମ ଚାଉଳ।
₹25.50 → ପଚିଶ ଟଙ୍କା ପଚାଶ ପଇସା
```

## Python

Build/install this checkout (CPython 3.11–3.14 and the Rust toolchain are needed
for a source build; this package has not been published to PyPI):

```sh
python -m pip install ./bindings/python
```

Install directly from GitHub with pip or uv (a source build requires Rust):

```sh
python -m pip install "git+https://github.com/OliveSuite-AI/normalizer-or.git#subdirectory=bindings/python"
# With a virtual environment active:
uv pip install "git+https://github.com/OliveSuite-AI/normalizer-or.git#subdirectory=bindings/python"
```

See [Python publishing](docs/publishing.md) for the PyPI release checklist and
the installation commands that become available after publication.

Run all [Python examples](bindings/python/examples/README.md) after installation:

```sh
python bindings/python/examples/run_all.py
```

```python
from normalizer_or import Normalizer

normalizer = Normalizer()
result = normalizer.normalize("ଆଜି ୨୫ kg ଚାଉଳ।")
assert result.normalized_text == "ଆଜି ପଚିଶ କିଲୋଗ୍ରାମ ଚାଉଳ।"
assert result.complete

written = normalizer.clean("  ଓଡ଼ିଆ 123\n", digits="odia", whitespace="collapse")
assert written.normalized_text == " ଓଡ଼ିଆ ୧୨୩ "
```

## Rust

Use a local path dependency until the package is published:

```toml
[dependencies]
normalizer-or = { path = "/path/to/normalizer-or" }
```

```rust
use normalizer_or::{CleanOptions, DigitStyle, NormalizeOptions, Normalizer};
let n = Normalizer::new()?;
assert_eq!(n.normalize("୨୫ INR", &NormalizeOptions::default())?.normalized_text(), "ପଚିଶ ଟଙ୍କା");
let options = CleanOptions { digits: DigitStyle::Odia, ..Default::default() };
assert_eq!(n.clean("123", &options)?.normalized_text(), "୧୨୩");
# Ok::<(), normalizer_or::NormalizeError>(())
```

## Behavior

ASCII and Odia digits can be mixed. Speech uses Indian lakh/crore scales and
Odia-script words. English prose, names, and standalone Latin letters stay as written.
No spelling correction, English transliteration, or general Odia suffix inference is performed.

Unresolved notation has three policies: `preserve` (default) returns unchanged
source plus issues, `reject` raises an error, and `fallback` reads the written
source literally with separate fallback diagnostics. Fallback does not repair
invalid dates or certify identifiers. URLs, emails, and opaque digit-bearing
identifiers are protected from partial numeric expansion.

Every result includes `locale="or-IN"`, a build identity, source segments, issues,
and fallback records. **All coordinates are original UTF-8 byte offsets**, including
Python hints; they are not character positions. Cleanup changes also retain original
source ranges. Speech never silently runs cleanup first.

This is an alpha implementation. Spellings and speech choices are explicit,
testable project defaults; native-speaker and voice/audio acceptance remain
required before a production speech release. `complete` reports resolved supported
work, not pronunciation quality or universal language coverage.

See [formats and limits](docs/normalization.md), [Python API](bindings/python/README.md),
[data provenance](src/data/provenance.md), [development](CONTRIBUTING.md),
[robustness testing](docs/testing.md), and [performance](PERFORMANCE.md).
