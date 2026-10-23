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
