"""Preserve, reject, and fallback for malformed and protected expressions."""

from normalizer_or import NormalizationError, Normalizer


def main():
    n = Normalizer()
    sources = [
        "1,,234",
        "₹25.005",
        "₹1 234",
        "5 KG",
        "5%%",
        "29/02/2025",
        "24:00",
        "AB12",
        "https://example.com/a12?q=5",
        "+91 98765 43210",
    ]
    for source in sources:
        kept = n.normalize(source, ambiguity_policy="preserve")
        assert kept.normalized_text == source and not kept.complete
        assert len(kept.issues) == 1 and not kept.fallback_used
        issue = kept.issues[0]
        assert (issue.start_byte, issue.end_byte) == (0, len(source.encode("utf-8")))
        try:
            n.normalize(source, ambiguity_policy="reject")
        except NormalizationError as exc:
            assert exc.code == "unresolved" and exc.issues == kept.issues
        else:
            raise AssertionError(f"reject accepted {source!r}")
        read = n.normalize(source, ambiguity_policy="fallback")
        assert read.complete and not read.issues and read.fallback_used
        assert len(read.fallbacks) == 1
        diagnostic = read.fallbacks[0]
        assert (diagnostic.start_byte, diagnostic.end_byte) == (
            0,
            len(source.encode("utf-8")),
        )
        print(
            f"{source!r}: preserve issue={issue.category}; fallback={read.normalized_text!r}"
        )
        print(f"  assumption: {diagnostic.reason}, strategy: {diagnostic.strategy}")
    # Fallback reads source literally; it does not repair an invalid date.
    print(f"PASS: {len(sources)} expressions under all three policies")


if __name__ == "__main__":
    main()
