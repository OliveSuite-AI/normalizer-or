"""Installed-wheel tests share the Rust contract inputs and expected readings."""

import json
from pathlib import Path
from dataclasses import FrozenInstanceError

import pytest
from normalizer_or import (
    NORMALIZER_ID,
    CancellationToken,
    Hint,
    NormalizationError,
    Normalizer,
    strip_markdown,
)
from scripts.contracts import check_fixture_case, run_request

CASES = json.loads(
    (Path(__file__).parents[3] / "tests/fixtures/odia.json").read_text(encoding="utf-8")
)


def partition(text, result):
    cursor = 0
    for segment in result.segments:
        assert segment.start_byte == cursor
        assert text.encode()[segment.start_byte : segment.end_byte].decode()
        cursor = segment.end_byte
    assert cursor == len(text.encode())
    assert "".join(s.text for s in result.segments) == result.normalized_text
    assert result.complete == (not result.issues)
    assert result.locale == "or-IN"
    assert result.normalizer_id == NORMALIZER_ID


@pytest.mark.parametrize("case", CASES, ids=lambda c: c["id"])
@pytest.mark.parametrize("policy", ["preserve", "reject", "fallback"])
def test_shared_contract(case, policy):
    n = Normalizer()
    preserved = run_request(n, dict(case, policy="preserve"))
    actual = run_request(n, dict(case, policy=policy))
    check_fixture_case(case, policy, actual, preserved)
    if "error" not in actual:
        partition(case["input"], n.normalize(case["input"], ambiguity_policy=policy))


@pytest.mark.parametrize(
    "digits,end", [("preserve", "୧2"), ("ascii", "12"), ("odia", "୧୨")]
)
@pytest.mark.parametrize("whitespace", ["preserve", "collapse"])
def test_clean(digits, end, whitespace):
    text = "  କ୍\u200dଷ କ୍\u200cଷ e\u0301\r\n୧2  "
    n = Normalizer()
    result = n.clean(text, digits=digits, whitespace=whitespace)
    partition(text, result)
    assert "କ୍\u200dଷ କ୍\u200cଷ é" in result.normalized_text
    assert end in result.normalized_text
    assert ("\r\n" in result.normalized_text) == (whitespace == "preserve")
    assert (
        n.clean(
            result.normalized_text, digits=digits, whitespace=whitespace
        ).normalized_text
        == result.normalized_text
    )
    assert any(s.kind == "normalization" for s in result.segments)
    with pytest.raises(FrozenInstanceError):
        result.complete = False


def test_hints_and_original_coordinates():
    text = "e\u0301 ଓଡ଼ିଆ ୦୧୨"
    start = len(text[: text.index("୦")].encode())
    result = Normalizer().normalize(
        text, hints=[Hint(start, len(text.encode()), "cardinal")]
    )
    partition(text, result)
    assert result.normalized_text.endswith("ବାର")
    assert result.normalized_text.startswith("e\u0301")
    assert result.segments[-1].start_byte == start
    for source, hint in [
        ("e\u0301 12", Hint(0, 1, "digits")),
        ("5 kg", Hint(0, 1, "cardinal")),
        ("AB12", Hint(2, 4, "cardinal")),
    ]:
        with pytest.raises(NormalizationError) as caught:
            Normalizer().normalize(source, hints=[hint])
        assert caught.value.code == "invalid_hint"


@pytest.mark.parametrize(
    "block",
    [
        "$$\n\\frac{1}{6}x^2 + \\frac{5}{6}x + 1 = \\left(\\frac{1}{6}x^2 + \\frac{5}{6}x\\right) + \\left(1\\right)\n$$",
        r"\[\frac{1}{6}x^2 + \frac{5}{6}x + 1 = \left(\frac{1}{6}x^2 + \frac{5}{6}x\right) + \left(1\right)\]",
        r"\(\frac{1}{6}x^2 + \frac{5}{6}x + 1 = \left(\frac{1}{6}x^2 + \frac{5}{6}x\right) + \left(1\right)\)",
        r"$\frac{1}{6}x^2 + \frac{5}{6}x + 1 = \left(\frac{1}{6}x^2 + \frac{5}{6}x\right) + \left(1\right)$",
    ],
)
def test_latex_math_is_spoken_as_a_whole_formula_segment(block):
    text = f"Before {block} after 6"
    start = len("Before ".encode())
    end = start + len(block.encode())
    normalizer = Normalizer()

    result = normalizer.normalize(text)
    partition(text, result)
    assert result.complete
    assert len(result.issues) == 0
    formula = next(s for s in result.segments if s.kind == "formula")
    assert (formula.start_byte, formula.end_byte) == (start, end)
    assert "ଯୁକ୍ତ" in result.normalized_text
    assert result.normalized_text.endswith("after ଛଅ")

    fallback = normalizer.normalize(text, ambiguity_policy="fallback")
    assert len(fallback.fallbacks) == 0
    assert fallback.normalized_text.endswith("after ଛଅ")

    assert normalizer.normalize(text, ambiguity_policy="reject").complete


def test_plain_formula_and_ambiguous_coefficient():
    normalizer = Normalizer()
    result = normalizer.normalize("x²+5x+6")
    assert result.segments[0].kind == "formula"
    assert "ଘାତ" in result.normalized_text
    assert normalizer.normalize("5x").normalized_text == "5x"


def test_explicit_scalar_and_continuation_math():
    normalizer = Normalizer()
    expected = {
        "$5x$": "ପାଞ୍ଚ ଗୁଣନ ଏକ୍ସ",
        "$6$": "ଛଅ",
        "$$=x+1$$": "ସମାନ ଏକ୍ସ ଯୁକ୍ତ ଏକ",
        r"$$\boxed{x^2 + 5x + 6 = 0}$$": "ସମାନ",
    }
    for source, phrase in expected.items():
        result = normalizer.normalize(source)
        assert result.complete
        assert phrase in result.normalized_text
    heading = normalizer.normalize("ପଦକ୍ଷେପ 1: ଖୋଜନ୍ତୁ")
    assert heading.complete
    assert heading.normalized_text.startswith("ପଦକ୍ଷେପ ଏକ:")


def test_rectangle_area_and_perimeter_paragraph_end_to_end():
    source = r"""ଏକ ଆୟତକ୍ଷେତ୍ରର କ୍ଷେତ୍ରଫଳ ଓ ପରିଧି ସୂତ୍ରରୁ ହିସାବ କରାଯାଏ।

**କ୍ଷେତ୍ରଫଳ:**
$$
\text{Area} = \text{length} \times \text{breadth}
$$
ଦିଆଯାଇଛି:
- ଲମ୍ବ = 8 ସେମି
- ପ୍ରସ୍ଥ = 5 ସେମି

ତେଣୁ,
$$
\text{Area} = 8 \times 5 = 40 \text{ cm}^2
$$

**ପରିଧି:**
$$
\text{Perimeter} = 2 \times (\text{length} + \text{breadth})
$$
$$
\text{Perimeter} = 2 \times (8 + 5) = 2 \times 13 = 26 \text{ cm}
$$

✅ ସୂତ୍ରଗୁଡ଼ିକ:
$$
\text{Area} = \text{length} \times \text{breadth}
\text{Perimeter} = 2 \times (\text{length} + \text{breadth})
$$"""
    prepared = strip_markdown(source)
    result = Normalizer().normalize(prepared)
    assert result.complete
    assert not result.issues
    assert result.normalized_text.count("କ୍ଷେତ୍ରଫଳ ସମାନ") == 3
    assert result.normalized_text.count("ପରିଧି ସମାନ") == 3
    assert "ଚାଳିଶ ବର୍ଗ ସେଣ୍ଟିମିଟର" in result.normalized_text
    assert "ଛବିଶ ସେଣ୍ଟିମିଟର" in result.normalized_text


@pytest.mark.parametrize(
    "formula",
    [
        "$x^2 + 5x + 6 = 0$",
        "$$\nx^2 + 5x + 6 = 0\n$$",
        r"\(x^2 + 5x + 6 = 0\)",
        r"\[x^2 + 5x + 6 = 0\]",
    ],
)
def test_strip_markdown_preserves_latex_before_normalization(formula):
    source = f"# ଉତ୍ତର\n\n**6** **ସଠିକ୍ ପ୍ରକ୍ରିୟା**: {formula}\n- `୬`"
    prepared = strip_markdown(source)
    assert prepared == f"ଉତ୍ତର\n\n6 ସଠିକ୍ ପ୍ରକ୍ରିୟା: {formula}\n୬"

    result = Normalizer().normalize(prepared)
    assert result.normalized_text.startswith("ଉତ୍ତର\n\nଛଅ ସଠିକ୍ ପ୍ରକ୍ରିୟା: ")
    assert result.normalized_text.endswith("\nଛଅ")
    assert "ଘାତ" in result.normalized_text
    assert result.complete
    assert any(s.kind == "formula" for s in result.segments)


def test_strip_markdown_protects_bold_algebra_but_normalizes_bold_numbers():
    source = "**x² + 5x + 6** and **6**"
    prepared = strip_markdown(source)
    assert prepared == "$x² + 5x + 6$ and 6"

    result = Normalizer().normalize(prepared)
    assert "ଘାତ" in result.normalized_text
    assert "and ଛଅ" in result.normalized_text
    assert result.complete
    assert any(s.kind == "formula" for s in result.segments)


@pytest.mark.parametrize("operation", ["normalize", "clean"])
def test_validation_controls_and_deadlines(operation):
    call = getattr(Normalizer(), operation)
    for text in ["", " \t", "a\0", "a\u202e", "\ud800"]:
        with pytest.raises(NormalizationError) as caught:
            call(text)
        assert caught.value.code == "invalid_input"
    token = CancellationToken()
    token.cancel()
    with pytest.raises(NormalizationError) as caught:
        call("୫", cancellation=token)
    assert caught.value.code == "cancelled"
    with pytest.raises(ValueError):
        call("a", deadline_ms=0)
    with pytest.raises(TypeError):
        call("a", deadline_ms=True)
    with pytest.raises(TypeError):
        call(12)
    with pytest.raises(NormalizationError) as caught:
        call("a" * 32769)
    assert caught.value.limit_kind == "input"


def test_options_and_removed_hints():
    with pytest.raises(ValueError):
        Hint(0, 1, "telephone")
    with pytest.raises(ValueError):
        Normalizer().clean("a", digits="latin")
    with pytest.raises(ValueError):
        Normalizer().clean("a", whitespace="trim")
    with pytest.raises(ValueError):
        Normalizer().normalize("a", ambiguity_policy="guess")


def test_non_utf8_locale_file_loading():
    """Fixture/vocabulary loading must not inherit a machine's legacy encoding."""
    import os
    import subprocess
    import sys

    root = Path(__file__).parents[3]
    env = dict(os.environ, LC_ALL="C", PYTHONUTF8="0", PYTHONCOERCECLOCALE="0")
    for script in [Path(__file__), root / "scripts/check_vocabulary.py"]:
        subprocess.run(
            [
                sys.executable,
                "-c",
                "import runpy,sys; runpy.run_path(sys.argv[1])",
                str(script),
            ],
            env=env,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )


def test_boundary_hints_and_original_ranges():
    n = Normalizer()
    for source, start in [
        ("+91 98765 43210", 0),
        ("phone: 9876543210", len("phone: ")),
    ]:
        end = len(source.encode("utf-8"))
        with pytest.raises(NormalizationError) as caught:
            n.normalize(source, hints=[Hint(start, start + 3, "digits")])
        assert caught.value.code == "invalid_hint"
        with pytest.raises(NormalizationError) as caught:
            n.normalize(source, hints=[Hint(start, end, "cardinal")])
        assert caught.value.code == "invalid_hint"
        result = n.normalize(source, hints=[Hint(start, end, "digits")])
        assert result.complete and not result.fallback_used
        assert result.segments[-1].kind == "digits"
        partition(source, result)
    for body in ["₹1 234", "% 12 %", "5 kg kg"]:
        source = "e\u0301 [" + body + "]; 6 kg"
        start = len("e\u0301 [".encode("utf-8"))
        result = n.normalize(source)
        partition(source, result)
        assert len(result.issues) == 1
        assert (result.issues[0].start_byte, result.issues[0].end_byte) == (
            start,
            start + len(body.encode("utf-8")),
        )
        assert result.normalized_text.endswith("; ଛଅ କିଲୋଗ୍ରାମ")
        with pytest.raises(NormalizationError) as caught:
            n.normalize(source, hints=[Hint(start, start + 1, "cardinal")])
        assert caught.value.code == "invalid_hint"
