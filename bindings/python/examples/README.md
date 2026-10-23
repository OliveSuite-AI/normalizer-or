# Runnable Python examples

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/OliveSuite-AI/normalizer-or/blob/main/bindings/python/examples/normalizer_or_colab.ipynb)

For Google Colab, open the notebook above in a fresh CPU runtime and select
**Runtime → Run all**. It installs the pinned source revision and Rust, then
demonstrates the API and runs all seven scripts. No GPU or API key is needed.
Initial source compilation needs internet and can take several minutes.

From the repository root, install the package into your active virtual environment:

```sh
python -m pip install ./bindings/python
```

Or install with uv into a virtual environment:

```sh
uv venv
uv pip install ./bindings/python
```

Source installation needs Rust. These examples use the installed `normalizer_or`
package and the Python standard library; they do not alter Python's import path
or require pytest. Odia vocabulary is still an alpha default pending linguistic
review. See [publishing](../../../docs/publishing.md) for installation options.

Run everything from the repository root:

```sh
python bindings/python/examples/run_all.py
# With uv's virtual environment:
uv run --no-project python bindings/python/examples/run_all.py
```

Each script prints results, asserts expected behavior, and exits with a nonzero
status on failure. Run without `-O` so Python keeps assertions enabled.
The runner uses UTF-8 output and stops on the first failed example. To run a single
example, replace `run_all.py` in the command with a filename below.

| Script | Demonstrates |
|---|---|
| `showcase.py` | 40 verified speech cases: ASCII/Odia/mixed digits, grouping, lakh/crore, signs, leading zeros, decimals, INR/paise, percentages, every supported unit, date formats, leap day, clocks, mixed prose and punctuation |
| `cleanup.py` | All six digit/whitespace option combinations, NFC composition, joiners, edge spaces, idempotence, cleanup versus speech |
| `policies.py` | Preserve/reject/fallback for ten malformed or protected expressions; whole-span issues and fallback assumptions |
| `hints.py` | Cardinal, digits, date and time hints; UTF-8 offsets after Odia text; explicit phone digit reading; invalid hints |
| `diagnostics.py` | Segment/source inspection, multiple issues, output reconstruction and JSON serialization |
| `controls.py` | Invalid Unicode/input/options, permanent cancellation, valid deadlines and the 32 KiB input limit |
| `batch_and_json.py` | Shared normalizer in a thread pool, ordered results, per-row errors and a UTF-8 JSONL file round trip |

`batch_and_json.py` writes to a temporary directory and removes it on completion.
Change the output path in that script to retain a result file. JSON records include
completeness and diagnostics, so an application can distinguish unresolved text
from successful readings. Fallback reports assumptions; it does not repair or
validate malformed data. Deadline examples avoid timing-dependent assertions.

The examples are also executed from an unrelated working directory by
`bindings/python/tests/test_examples.py`, which runs in the existing Python CI
matrix against each installed wheel. To run the regression suite with pytest
installed in the same environment:

```sh
python -m pytest bindings/python/tests -q
```

Notebook example cells are also executed in the test suite against the installed
package. The network/toolchain setup cell is syntax-checked locally; a hosted
Colab runtime is needed to validate that installation step end to end.
