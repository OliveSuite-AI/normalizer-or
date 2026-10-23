"""Validate the Odia numeral inventory without duplicating normalization rules."""

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
vocabulary = (ROOT / "src/vocabulary.rs").read_text(encoding="utf-8")
small = vocabulary.split("pub(crate) const SMALL:", 1)[1].split("];", 1)[0]
words = re.findall(r'"([^"\n]+)"', small)
assert len(words) == 100 and len(set(words)) == 100
for word in words:
    assert all(0x0B00 <= ord(char) <= 0x0B7F or char.isspace() for char in word), word
print("100 distinct Odia numeral entries validated")
