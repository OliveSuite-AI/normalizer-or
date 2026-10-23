"""Odia-only speech and written cleanup examples."""

from normalizer_or import Normalizer

n = Normalizer()
for source in ["୨୫ kg", "₹1,23,456.50", "23/10/2026", "09:30", "English ଓଡ଼ିଆ 123"]:
    result = n.normalize(source)
    print(source, "→", result.normalized_text, "complete:", result.complete)
print(n.clean("  123\n", digits="odia", whitespace="collapse").normalized_text)
