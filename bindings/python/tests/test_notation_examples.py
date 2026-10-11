import json
from pathlib import Path

import pytest
from normalizer_tr import Hint, NormalizationError, Normalizer

FIXTURES = Path(__file__).resolve().parents[3] / "tests" / "fixtures"
CASES = json.loads((FIXTURES / "notation-examples.json").read_text(encoding="utf-8"))


@pytest.mark.parametrize("case", CASES, ids=lambda case: case["text"][:30])
@pytest.mark.parametrize("policy", ("preserve", "reject", "fallback"))
def test_complete_primary_sentences(case, policy):
    result = Normalizer().normalize(case["text"], ambiguity_policy=policy)
    assert result.normalized_text == case["expected"]
    assert result.complete and result.issues == result.fallbacks == ()
    source = case["text"].encode("utf-8")
    cursor = 0
    for segment in result.segments:
        assert segment.start_byte == cursor
        cursor = segment.end_byte
        source[segment.start_byte : cursor].decode("utf-8")
    assert cursor == len(source)
    assert "".join(segment.text for segment in result.segments) == case["expected"]


@pytest.mark.parametrize("policy", ("preserve", "reject", "fallback"))
def test_composite_hints_keep_whole_owners_and_original_unicode(policy):
    source = "ö e\u0301 12*833,25 TL"
    start = len("ö e\u0301 ".encode("utf-8"))
    with pytest.raises(NormalizationError) as error:
        Normalizer().normalize(
            source, ambiguity_policy=policy, hints=(Hint(start, start + 2, "cardinal"),)
        )
    assert error.value.code == "invalid_hint"


def test_bad_iban_is_not_certified_and_prose_period_is_not_spoken():
    source = "IBAN numaranız TR12 0006 4000 0011 2345 6789 01."
    n = Normalizer()
    preserve = n.normalize(source)
    assert not preserve.complete and len(preserve.issues) == 1
    assert preserve.issues[0].start_byte == len("IBAN numaranız ".encode("utf-8"))
    assert preserve.issues[0].end_byte == len(source.encode("utf-8")) - 1
    with pytest.raises(NormalizationError) as error:
        n.normalize(source, ambiguity_policy="reject")
    assert error.value.code == "unresolved" and error.value.issues == preserve.issues
    fallback = n.normalize(source, ambiguity_policy="fallback")
    assert fallback.complete and len(fallback.fallbacks) == 1
    assert not any(segment.kind == "iban" for segment in fallback.segments)
    assert fallback.normalized_text.endswith(".")
    assert not fallback.normalized_text.endswith("nokta")
