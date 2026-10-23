"""All four hint kinds, original UTF-8 offsets, and invalid hint handling."""

from normalizer_or import Hint, NormalizationError, Normalizer


def main():
    n = Normalizer()
    prefix = "ଓଡ଼ିଆ "
    for expression, kind, expected in [
        ("୦୧୨", "cardinal", "ବାର"),
        ("୦୧୨", "digits", "ଶୂନ୍ୟ ଏକ ଦୁଇ"),
        ("23/10/2026", "date", "ତେଇଶ ଅକ୍ଟୋବର ଦୁଇ ହଜାର ଛବିଶ"),
        ("09:30", "time", "ନଅ ଘଣ୍ଟା ତିରିଶ ମିନିଟ"),
    ]:
        source = prefix + expression
        start = len(prefix.encode("utf-8"))  # Byte offset, not len(prefix).
        hint = Hint(start, len(source.encode("utf-8")), kind)
        result = n.normalize(source, hints=[hint])
        assert result.complete and result.normalized_text == prefix + expected
        print(f"{kind}: {source!r} → {result.normalized_text!r}")
    phone = "+91 98765 43210"
    assert not n.normalize(phone).complete
    result = n.normalize(phone, hints=[Hint(0, len(phone.encode("utf-8")), "digits")])
    assert result.complete and not result.fallback_used
    assert result.normalized_text == "ଯୁକ୍ତ ନଅ ଏକ ନଅ ଆଠ ସାତ ଛଅ ପାଞ୍ଚ ଚାରି ତିନି ଦୁଇ ଏକ ଶୂନ୍ୟ"
    print(f"Explicit phone digit reading: {result.normalized_text}")
    for source, hint in [
        ("୨୫", Hint(0, 1, "digits")),  # Splits a UTF-8 scalar.
        ("5 kg", Hint(0, 1, "cardinal")),  # Cuts a quantity.
        ("29/02/2025", Hint(0, 10, "date")),  # Invalid date.
    ]:
        try:
            n.normalize(source, hints=[hint])
        except NormalizationError as exc:
            assert exc.code in ("invalid_hint", "unresolved")
            print(f"Rejected hint for {source!r}: {exc.code}")
        else:
            raise AssertionError(f"invalid hint accepted: {source!r}")
    print("PASS: hints and original UTF-8 coordinates")


if __name__ == "__main__":
    main()
