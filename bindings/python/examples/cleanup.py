"""All cleanup options, Unicode composition, joiners, and idempotence."""

from normalizer_or import Normalizer


def main():
    n = Normalizer()
    source = "  କୋ କ୍\u200dଷ କ୍\u200cଷ English\r\n୧2  "
    for digits, expected_digits in [
        ("preserve", "୧2"),
        ("ascii", "12"),
        ("odia", "୧୨"),
    ]:
        for whitespace in ["preserve", "collapse"]:
            result = n.clean(source, digits=digits, whitespace=whitespace)
            expected = f"  କୋ କ୍\u200dଷ କ୍\u200cଷ English\r\n{expected_digits}  "
            if whitespace == "collapse":
                expected = f" କୋ କ୍\u200dଷ କ୍\u200cଷ English {expected_digits} "
            assert result.normalized_text == expected
            assert result.complete and not result.fallback_used
            assert (
                n.clean(expected, digits=digits, whitespace=whitespace).normalized_text
                == expected
            )
            print(f"{digits=}, {whitespace=}: {expected!r}")
    assert n.clean("123", digits="odia").normalized_text == "୧୨୩"
    assert n.normalize("123").normalized_text == "ଏକ ଶହ ତେଇଶ"
    assert (
        n.clean("https://example.com/123").normalized_text == "https://example.com/123"
    )
    print(
        "PASS: cleanup preserves joiners and edge spaces; all six option combinations"
    )


if __name__ == "__main__":
    main()
