"""Inspect original source spans and serialize immutable result records."""

import json
from dataclasses import asdict

from normalizer_or import Normalizer


def main():
    source = "ଓଡ଼ିଆ ୨୫ kg; AB12; 24:00"
    result = Normalizer().normalize(source)
    encoded = source.encode("utf-8")
    assert result.normalized_text == "ଓଡ଼ିଆ ପଚିଶ କିଲୋଗ୍ରାମ; AB12; 24:00"
    assert not result.complete and len(result.issues) == 2
    assert [
        encoded[i.start_byte : i.end_byte].decode("utf-8") for i in result.issues
    ] == ["AB12", "24:00"]
    cursor = 0
    for segment in result.segments:
        assert segment.start_byte == cursor
        original = encoded[segment.start_byte : segment.end_byte].decode("utf-8")
        print(
            f"[{segment.start_byte}:{segment.end_byte}] {segment.kind}: {original!r} → {segment.text!r}"
        )
        cursor = segment.end_byte
    assert cursor == len(encoded)
    assert "".join(s.text for s in result.segments) == result.normalized_text
    record = asdict(result)
    record["fallback_used"] = (
        result.fallback_used
    )  # Properties are not dataclass fields.
    serialized = json.dumps(record, ensure_ascii=False, indent=2)
    assert json.loads(serialized)["normalized_text"] == result.normalized_text
    print(serialized)
    print("PASS: segments, multiple diagnostics, JSON serialization")


if __name__ == "__main__":
    main()
