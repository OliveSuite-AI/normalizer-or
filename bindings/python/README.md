# Python API

The `normalizer_or` package calls the Rust Odia engine and has no runtime Python dependencies.
Use CPython 3.11–3.14. Source builds need the pinned Rust toolchain and maturin.

```sh
python -m pip install ./bindings/python
```

Run the [complete example suite](examples/README.md) from the repository root:

```sh
python bindings/python/examples/run_all.py
```

The scripts include expected-output assertions and cover speech, cleanup,
policies, hints, diagnostics, errors, controls, threading, and JSONL files.

```python
from normalizer_or import Hint, Normalizer

n = Normalizer()
spoken = n.normalize("୨୫ kg", ambiguity_policy="preserve")
written = n.clean("123", digits="odia", whitespace="preserve")
assert spoken.normalized_text == "ପଚିଶ କିଲୋଗ୍ରାମ"
assert written.normalized_text == "୧୨୩"

# Hint coordinates are UTF-8 bytes, including for Odia.
r = n.normalize("୦୧୨", hints=[Hint(0, len("୦୧୨".encode("utf-8")), "cardinal")])
assert r.normalized_text == "ବାର"
```

### Markdown input

Use `strip_markdown` when the source contains common Markdown headings, lists,
emphasis, inline code, or links. It removes those formatting markers while
preserving their text and copying paired LaTeX spans unchanged. Bold spans that
look like algebra are wrapped in inline dollar delimiters to keep their terms
together. The normalizer speaks supported elementary algebra formulas in Odia;
unsupported formula syntax remains intact as an unresolved expression.

```python
from normalizer_or import Normalizer, strip_markdown

markdown = "# ଉତ୍ତର\n\n**ଫଳାଫଳ**: $x^2 + 5x + 6 = 0$"
prepared = strip_markdown(markdown)
result = Normalizer().normalize(prepared)

assert prepared == "ଉତ୍ତର\n\nଫଳାଫଳ: $x^2 + 5x + 6 = 0$"
assert result.complete
assert "ଘାତ" in result.normalized_text
```

`strip_markdown` handles common Markdown patterns; it is not a complete
CommonMark renderer. Diagnostic and segment byte offsets refer to the stripped
text passed to `normalize`, not the original Markdown source.

`normalize(text, *, ambiguity_policy="preserve", hints=(), cancellation=None,
deadline_ms=None)` returns an immutable `NormalizeResult`. Supported policies:
`preserve`, `reject`, `fallback`. Hint kinds: `cardinal`, `digits`, `date`, `time`.

`clean(text, *, digits="preserve", whitespace="preserve", cancellation=None,
deadline_ms=None)` returns the same immutable result type. Digit styles:
`preserve`, `ascii`, `odia`. Whitespace policies: `preserve`, `collapse` (no trim).

Both methods support a reusable `CancellationToken`; `cancel()` cancels active
calls and future calls using it. Deadlines are integer milliseconds 1–60000,
measured monotonically; booleans are not valid integers. Both release the GIL
while the Rust operation runs. Input/hint/control/limit errors raise
`NormalizationError` with `code`, `issues`, and optional `limit_kind` properties.
Invalid Python option values raise ValueError/TypeError. Unpaired Unicode
surrogates raise `NormalizationError(code="invalid_input")`.

Results contain normalized_text, locale, normalizer_id, complete, segments,
issues, fallbacks, and a derived fallback_used property. Segments/diagnostics
refer to original UTF-8 byte ranges, not Python character indices. See the
[normalization contract](../../docs/normalization.md) for formats and limits.
