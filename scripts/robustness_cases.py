"""Independent expected cases and deterministic pairwise generation. No engine imports."""

import calendar
import itertools
import json
import random
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ORACLES = json.loads(
    (ROOT / "tests/fixtures/odia-oracles.json").read_text(encoding="utf-8")
)
NUMBERS = ORACLES["numbers"]
POLICIES = ("preserve", "reject", "fallback")
DIMENSIONS = (range(3), range(6), range(4), range(3), range(3), range(5))


def pairs(row):
    return {
        (i, row[i], j, row[j]) for i in range(len(row)) for j in range(i + 1, len(row))
    }


def pairwise(seed):
    candidates = list(itertools.product(*DIMENSIONS))
    random.Random(seed).shuffle(candidates)
    uncovered = set().union(*(pairs(row) for row in candidates))
    selected = []
    while uncovered:
        row = max(candidates, key=lambda row: len(pairs(row) & uncovered))
        selected.append(row)
        uncovered -= pairs(row)
        candidates.remove(row)
    return selected


def script(text, mode):
    count = 0
    result = []
    for char in text:
        if char in "0123456789":
            if mode == 1 or (mode == 2 and count % 2 == 0):
                char = chr(ord("୦") + int(char))
            count += 1
        result.append(char)
    return "".join(result)


def wrap(text, mode):
    left, right = [
        ("", ""),
        ("(", ")"),
        ("[", "]"),
        ('"', '"'),
        ("“", "”"),
        ("[“", "”]"),
    ][mode]
    return left + text + right


def positive(case_id, source, spoken):
    return {
        "id": case_id,
        "input": source,
        "expected": {
            "normalized_text": spoken,
            "complete": True,
            "issues": [],
            "fallbacks": [],
        },
    }


def negative(
    case_id, source, category="invalid_expression", span=None, strategy="literal"
):
    return {
        "id": case_id,
        "input": source,
        "invalid": True,
        "strategy": strategy,
        "category": category,
        "span": span or [0, len(source.encode("utf-8"))],
    }


def generated_cases(seed=1729, deep=False):
    cases = []
    for value, word in enumerate(NUMBERS):
        for mode in range(3):
            cases.append(
                positive(f"numeral-{value}-{mode}", script(str(value), mode), word)
            )
    for value, word in ORACLES["scale_boundaries"].items():
        for sign, prefix in [("", ""), ("+", "ଯୁକ୍ତ "), ("-", "ବିଯୁକ୍ତ ")]:
            cases.append(
                positive(f"scale-{value}-{sign or 'none'}", sign + value, prefix + word)
            )
    # Exhaustive finite clock domain, independent of production date/time helpers.
    for hour in range(24):
        for minute in range(60):
            cases.append(
                positive(
                    f"clock-{hour:02d}{minute:02d}",
                    f"{hour:02d}:{minute:02d}",
                    f"{NUMBERS[hour]} ଘଣ୍ଟା {NUMBERS[minute]} ମିନିଟ",
                )
            )
    for year_text, spoken_year in ORACLES["years"].items():
        year = int(year_text)
        for month in range(1, 13):
            last = calendar.monthrange(year, month)[1]
            for day in [0, 1, last, last + 1]:
                formats = [
                    f"{year:04d}-{month:02d}-{day:02d}",
                    f"{day:02d}/{month:02d}/{year:04d}",
                    f"{day:02d}-{month:02d}-{year:04d}",
                ]
                for form, source in enumerate(formats):
                    cid = f"calendar-{year}-{month}-{day}-{form}"
                    if 1 <= day <= last:
                        cases.append(
                            positive(
                                cid,
                                source,
                                f"{NUMBERS[day]} {ORACLES['months'][month - 1]} {spoken_year}",
                            )
                        )
                    else:
                        cases.append(negative(cid, source))
    rows = list(itertools.product(*DIMENSIONS)) if deep else pairwise(seed)
    for index, (digits, wrapper, spacing, prefix, suffix, family) in enumerate(rows):
        space = ["", " ", "\u00a0", "\u202f"][spacing]
        source = [
            f"₹{space}25",
            f"25{space}INR",
            f"25{space}kg",
            f"%{space}25",
            f"25{space}%",
        ][family]
        output = ["ପଚିଶ ଟଙ୍କା", "ପଚିଶ ଟଙ୍କା", "ପଚିଶ କିଲୋଗ୍ରାମ", "ପଚିଶ ପ୍ରତିଶତ", "ପଚିଶ ପ୍ରତିଶତ"][family]
        lead = ["", "English ", "e\u0301 "][prefix]
        tail = ["", "।", "; 6 kg"][suffix]
        spoken_tail = ["", "।", "; ଛଅ କିଲୋଗ୍ରାମ"][suffix]
        cases.append(
            positive(
                f"matrix-{index}",
                script(lead + wrap(source, wrapper) + tail, digits),
                lead + wrap(output, wrapper) + spoken_tail,
            )
        )
    # Full three-way coverage for money x wrappers x list boundaries.
    for digits, wrapper, separator in itertools.product(
        range(3), range(6), [",", ", ", "; "]
    ):
        source = wrap(f"₹25{separator}₹30", wrapper)
        cases.append(
            positive(
                f"money-list-{digits}-{wrapper}-{separator!r}",
                script(source, digits),
                wrap(f"ପଚିଶ ଟଙ୍କା{separator}ତିରିଶ ଟଙ୍କା", wrapper),
            )
        )
    for minor in [0, 1, 9, 10, 50, 99]:
        tail = f" {NUMBERS[minor]} ପଇସା" if minor else ""
        for digits in range(3):
            cases.append(
                positive(
                    f"paise-{minor}-{digits}",
                    script(f"₹25.{minor:02d}", digits),
                    "ପଚିଶ ଟଙ୍କା" + tail,
                )
            )
    for precision in [1, 2, 8, 9]:
        cases.append(
            positive(
                f"fraction-{precision}",
                "25." + "0" * precision,
                "ପଚିଶ ଦଶମିକ " + " ".join([NUMBERS[0]] * precision),
            )
        )
    units = {
        "kg": "କିଲୋଗ୍ରାମ",
        "g": "ଗ୍ରାମ",
        "mg": "ମିଲିଗ୍ରାମ",
        "km": "କିଲୋମିଟର",
        "m": "ମିଟର",
        "cm": "ସେଣ୍ଟିମିଟର",
        "mm": "ମିଲିମିଟର",
        "L": "ଲିଟର",
        "mL": "ମିଲିଲିଟର",
        "°C": "ଡିଗ୍ରୀ ସେଲସିୟସ",
    }
    for label, word in units.items():
        for space in ["", " ", "\u00a0", "\u202f"]:
            cases.append(
                positive(f"unit-{label}-{space!r}", f"25{space}{label}", f"ପଚିଶ {word}")
            )
    mutations = [
        ("1,23", "invalid_expression"),
        ("1,,234", "invalid_expression"),
        ("01,234", "invalid_expression"),
        ("1,234,56", "invalid_expression"),
        ("25..50", "invalid_expression"),
        ("25.0000000000", "invalid_expression"),
        ("₹25.500", "invalid_expression"),
        ("₹₹25", "invalid_expression"),
        ("₹25 INR", "invalid_expression"),
        ("% 25 %", "invalid_expression"),
        ("25 KG", "unsupported"),
        ("25 kg\u0301", "unsupported"),
        ("25 kg kg", "unsupported"),
        ("(25 kg", "invalid_expression"),
        ("(25 kg]", "invalid_expression"),
        ("24:00", "invalid_expression"),
        ("12:60", "invalid_expression"),
        ("00/01/2024", "invalid_expression"),
        ("01/13/2024", "invalid_expression"),
    ]
    for index, (source, category) in enumerate(mutations):
        for mode in range(3):
            cases.append(
                negative(
                    f"mutation-{index}-{mode}",
                    script(source, mode),
                    category,
                    strategy="unicode_code_point" if "]" in source else "literal",
                )
            )
    for index, source in enumerate(
        [
            "+91 98765 43210",
            "+91\t98765\n43210",
            "+91 (98765 43210)",
            "+919876543210",
            "https://example.com/5?a=1,2;3",
            "www.example.in/25",
            "a12@example.com",
            "example.COM",
        ]
    ):
        cases.append(
            negative(
                f"protected-{index}",
                source,
                "protected_identifier",
                strategy="unicode_code_point" if "?" in source else "literal",
            )
        )
    for index, source in enumerate(
        ["Dr.Smith", "Smith.Jones", "English ଓଡ଼ିଆ", "P and S"]
    ):
        cases.append(positive(f"prose-{index}", source, source))
    return cases


def requests(seed=1729, deep=False):
    for case in generated_cases(seed, deep):
        for policy in POLICIES:
            request = {
                "id": f"{case['id']}/{policy}",
                "input": case["input"],
                "policy": policy,
            }
            if case.get("invalid"):
                issue = {"range": case["span"], "category": case["category"]}
                if policy == "preserve":
                    expected = {
                        "normalized_text": case["input"],
                        "complete": False,
                        "issues": [issue],
                        "fallbacks": [],
                    }
                elif policy == "reject":
                    expected = {"error": "unresolved", "issues": [issue]}
                else:
                    expected = {
                        "complete": True,
                        "issues": [],
                        "fallback_used": True,
                        "fallbacks": [
                            {
                                "range": case["span"],
                                "attempted_class": "identifier"
                                if case["category"] == "protected_identifier"
                                else "expression",
                                "original_category": case["category"],
                                "reason": {
                                    "invalid_expression": "invalid_form",
                                    "unsupported": "unsupported_form",
                                    "protected_identifier": "protected_identifier",
                                }[case["category"]],
                                "strategy": case["strategy"],
                            }
                        ],
                    }
                request["expected"] = expected
            else:
                request["expected"] = case["expected"]
            yield request
    # Typed hints: valid reordering/adjacency, invalid overlap, bounds and graphemes.
    for index, (source, hints, expected) in enumerate(
        [
            (
                "001",
                [{"start": 0, "end": 3, "kind": "cardinal"}],
                {"normalized_text": "ଏକ", "complete": True},
            ),
            (
                "1;2",
                [
                    {"start": 2, "end": 3, "kind": "digits"},
                    {"start": 0, "end": 1, "kind": "digits"},
                ],
                {"normalized_text": "ଏକ;ଦୁଇ", "complete": True},
            ),
            (
                "1",
                [{"start": 0, "end": 0, "kind": "digits"}],
                {"error": "invalid_hint"},
            ),
            (
                "1",
                [{"start": 1, "end": 0, "kind": "digits"}],
                {"error": "invalid_hint"},
            ),
            (
                "1",
                [{"start": 0, "end": 2, "kind": "digits"}],
                {"error": "invalid_hint"},
            ),
            (
                "1",
                [{"start": 0, "end": 1, "kind": "digits"}] * 2,
                {"error": "invalid_hint"},
            ),
            (
                "e\u0301 1",
                [{"start": 0, "end": 1, "kind": "digits"}],
                {"error": "invalid_hint"},
            ),
            (
                "୫ kg",
                [{"start": 0, "end": 1, "kind": "cardinal"}],
                {"error": "invalid_hint"},
            ),
            (
                "+91 98765 43210",
                [{"start": 0, "end": 3, "kind": "digits"}],
                {"error": "invalid_hint"},
            ),
        ]
    ):
        for policy in POLICIES:
            yield {
                "id": f"hint-{index}/{policy}",
                "input": source,
                "policy": policy,
                "hints": hints,
                "expected": expected,
            }
    extra_hints = [
        (
            "2024-02-29",
            "date",
            0,
            10,
            {
                "normalized_text": NUMBERS[29]
                + " "
                + ORACLES["months"][1]
                + " "
                + ORACLES["years"]["2024"],
                "complete": True,
            },
        ),
        ("2025-02-29", "date", 0, 10, {"error": "invalid_hint"}),
        (
            "23:59",
            "time",
            0,
            5,
            {
                "normalized_text": NUMBERS[23] + " ଘଣ୍ଟା " + NUMBERS[59] + " ମିନିଟ",
                "complete": True,
            },
        ),
        ("24:00", "time", 0, 5, {"error": "invalid_hint"}),
        (
            "+91 98765 43210",
            "digits",
            0,
            15,
            {
                "normalized_text": "ଯୁକ୍ତ "
                + " ".join(NUMBERS[int(d)] for d in "919876543210"),
                "complete": True,
            },
        ),
        ("+91 98765 43210", "cardinal", 0, 15, {"error": "invalid_hint"}),
        ("AB12", "digits", 2, 4, {"error": "invalid_hint"}),
        ("5kg", "cardinal", 0, 1, {"error": "invalid_hint"}),
        ("୧୨", "digits", 0, 3, {"error": "invalid_hint"}),
        ("1", "digits", 0, (1 << 63) - 1, {"error": "invalid_hint"}),
    ]
    for index, (source, kind, start, end, expected) in enumerate(extra_hints):
        for policy in POLICIES:
            yield {
                "id": f"extra-hint-{index}/{policy}",
                "input": source,
                "policy": policy,
                "hints": [{"kind": kind, "start": start, "end": end}],
                "expected": expected,
            }
    for operation in ["normalize", "clean"]:
        for size in [32767, 32768, 32769]:
            source = "ଅ" * (size // 3) + "a" * (size % 3)
            expected = (
                {
                    "normalized_text": source,
                    "complete": True,
                    "issues": [],
                    "fallbacks": [],
                }
                if size <= 32768
                else {"error": "limit_exceeded", "limit_kind": "input"}
            )
            yield {
                "id": f"utf8-limit-{operation}-{size}",
                "input": source,
                "operation": operation,
                "expected": expected,
            }
    for digits, whitespace, form in itertools.product(
        ["preserve", "ascii", "odia"], ["preserve", "collapse"], ["e\u0301", "é"]
    ):
        source = f"  {form} କ୍\u200dଷ କ୍\u200cଷ ୧2\r\n"
        tail = {"preserve": "୧2", "ascii": "12", "odia": "୧୨"}[digits]
        expected = (
            f"  é କ୍\u200dଷ କ୍\u200cଷ {tail}\r\n"
            if whitespace == "preserve"
            else f" é କ୍\u200dଷ କ୍\u200cଷ {tail} "
        )
        yield {
            "id": f"clean-{digits}-{whitespace}-{form!r}",
            "input": source,
            "operation": "clean",
            "digits": digits,
            "whitespace": whitespace,
            "expected": {
                "normalized_text": expected,
                "complete": True,
                "issues": [],
                "fallbacks": [],
            },
        }
    for operation in ["normalize", "clean"]:
        for index, source in enumerate(["", " \t", "a\u202e", "a\0", "a" * 32769]):
            yield {
                "id": f"invalid-input-{operation}-{index}",
                "input": source,
                "operation": operation,
                "expected": {"error": "limit_exceeded", "limit_kind": "input"}
                if len(source.encode("utf-8")) > 32768
                else {"error": "invalid_input"},
            }
        yield {
            "id": f"cancelled-{operation}",
            "input": "5 kg",
            "operation": operation,
            "cancelled": True,
            "expected": {"error": "cancelled"},
        }
