# Odia normalization contract

## Written cleanup

`clean` always applies NFC per original grapheme. It preserves spelling, case,
punctuation, English text, virama, nukta, vowel signs, ZWJ, and ZWNJ. It does not
use NFKC, remove markup/URLs, or infer spelling corrections.

`digits`: `preserve` (default), `ascii`, or `odia`; only ASCII and Odia decimal
digits participate. `whitespace`: `preserve` (default) or `collapse` (each run
becomes one ordinary space, including at input edges; no trimming).
Changes have segment kind `Normalization` / Python `normalization`, rule `text.cleanup`.
Adjacent equal cleanup segments can be merged. Cleanup is idempotent.

## Speech

| Family | Accepted formats and reading |
|---|---|
| Integers | ASCII `0–9`, Odia `୦–୯`, or mixed digits; values below `10^18` |
| Grouping | Ungrouped, Indian `1,23,456`, or international `123,456`; never mixed or malformed groups |
| Scales | Hundred, thousand, lakh, crore; larger values recursively decompose in crores |
| Signs | Explicit ASCII `+` and `-`, including signed zero |
| Leading zeros | Plain integer sequences read digitwise; Cardinal hint overrides; padded decimals/quantities remain invalid |
| Decimals | `.` separator, 1–9 fractional digits read separately; trailing zeros retained |
| INR | `₹` or exact `INR` prefix/suffix, compact or horizontal-space separated; 0–2 fractional digits; zero paise omitted |
| Percent | `%12`, `% 12`, `12%`, `12 %`; number followed by `ପ୍ରତିଶତ` |
| Units | Exact `kg`, `g`, `mg`, `km`, `m`, `cm`, `mm`, `L`, `mL`, `°C`; compact or spaced; no arbitrary case aliases |
| Gregorian date | `YYYY-MM-DD`, `DD/MM/YYYY`, `DD-MM-YYYY`; exact widths and valid calendar; day, month name, cardinal year |
| Clock | `HH:MM`, 00:00–23:59; hour + `ଘଣ୍ଟା` + minute + `ମିନିଟ`; no AM/PM inference or seconds |

Decimal commas and dotted thousands grouping are not supported. Money uses exact
integers, never floats, rounding, conversion, or exchange rates. Quantities
cannot become valid across tabs or newlines. A tab or other unsupported
horizontal separator stays inside one invalid quantity owner; line breaks end
quantity lookahead. Prose whitespace/punctuation stays
exact. Matched straight/smart quotes and parentheses/brackets may enclose spaced
quantities, including nested pairs; wrappers remain outside the expression's
source span. Unmatched wrappers cannot expose numeric fragments. A terminal full
stop is sentence punctuation, not an ordinal. Commas between contiguous digits
stay numeric syntax (including malformed grouping); other sentence/list commas
separate supported quantities, including `₹25,₹30` and `5kg,6kg`.

Whole-expression ownership precedes rendering. Duplicate/mixed labels and percent
markers, malformed marked units, and currency-adjacent horizontal numeric runs
are validated together. Space-grouped money is unsupported: `₹1 234` and
`123 456 INR` remain one unresolved amount, never a successfully normalized
fragment. Explicit list/sentence boundaries and independently labeled quantities
end that owner. Preserve reports one issue, Reject fails, and Fallback reports
one literal source-reading record. This conservative rule also treats unlabeled
numeric continuations adjacent to money as unresolved rather than guessing intent.

Supported hints are Cardinal, Digits, Date, and Time (lowercase names in Python).
Hints must cover whole expressions, be nonoverlapping, and align with original
grapheme boundaries. Digits allows an initial plus and space/dot/slash/hyphen/
parenthesis separators. Hints cannot certify invalid dates or cut quantities,
protected identifiers, or electronic source. A whole Digits hint may explicitly
read a protected phone; Cardinal/Date/Time and partial hints cannot override it.

Ordinals, suffix inflection, ranges, multiplication, abbreviations, pronunciation
aliases, address fields, telephone prosody, account validation, Roman numeral
speech, and electronic-address speech are deferred. Ordinary words are preserved;
unsupported structured forms produce issues.

Phone protection recognizes explicit `+91` forms and numeric runs following
`phone`, `tel`, `mobile`, `ଫୋନ`, or `ମୋବାଇଲ`, with an optional attached or spaced colon (ASCII cues
are case insensitive). Digits may be ASCII/Odia. These are protected source
spans, not validated or assigned telephone numbers. Tab/newline-separated phone
fragments stay protected as well; they never gain cardinal readings. Once ten national digits
are collected, a following independent quantity remains separate. Explicit
quantity labels take precedence over a generic cue (`phone: 5 kg` stays a unit).
Ordinary uncued integers retain cardinal behavior.

Elementary algebra formulas are parsed and spoken by default. Paired LaTeX
display delimiters (`$$…$$`, `\[…\]`) and inline delimiters (`\(…\)`, plus
`$…$` when the body contains math syntax) own one formula segment. Supported
forms include integers, single Latin variables, grouping, implicit or explicit
multiplication, division, powers, and equations using `+`, `-`, `*`, `/`, `×`,
`÷`, `^`, `²`, `³`, or `=`. Basic `\frac{…}{…}`, `\cdot`, `\times`, `\div`,
`\left`, `\right`, `\boxed{…}`, and recognized `\text{…}` labels and metric
units (Area, Perimeter, length, breadth, m, cm, mm, km) are accepted. Repeated
equalities and separate equations on display-math lines are read in sequence.
Explicitly delimited scalar formulas and continuation lines beginning with `=`
are spoken. Compact or spaced undelimited algebra such as `x² + 5x + 6` is
recognized; an isolated undelimited form such as `5x` remains prose. Formulas are
read symbolically with Odia variable names, operator readings, spoken grouping,
`ଘାତ` for powers, and `ଲବ`/`ହର` for fractions. The parser does not simplify or
solve expressions.
Unsupported or malformed formulas remain one unresolved expression, so
Preserve, Reject, and Fallback retain their existing meanings. Unmatched math
delimiters are not treated as formula blocks.

Electronic source protection recognizes HTTP/HTTPS, `www`, and email-shaped
source before splitting punctuation in paths/queries. Bare domains require ASCII
labels of 1–63 letters/digits/hyphens, without leading/trailing hyphens, and one
of `com`, `net`, `org`, `in`, `gov`, `edu`, `io`, `ai`, `app`, `dev`, `co` (case
insensitive). Ordinary dotted names such as `Dr.Smith` remain prose in every
policy. No domains are fetched or certified.

English names/letters are not
translated or transliterated. Digit-bearing unknown words remain whole owners.

## Policies and provenance

Preserve retains unresolved original text and returns `complete=false` with
issues. Reject returns an unresolved error with those issues and no partial
result. Fallback renders unresolved source literally with Odia digit/symbol
names and Unicode hexadecimal code-point readings for unknown symbols. It
reports handled assumptions in `fallbacks`, keeps primary readings unchanged,
and does not certify invalid logical values. Bounded coverage is not a universal
Unicode pronunciation engine. Ordinary punctuation and Odia marks are retained.

Speech recognition uses NFC internally. Verbatim and unresolved output always
comes from original input. No digit conversion is applied globally to protected
source. Segments partition original source contiguously and concatenate to the
output. All hint/diagnostic/segment ranges use original UTF-8 bytes.

Both operations reject empty/whitespace-only input, forbidden controls, and
Bidi_Control characters. Both allow ordinary tabs/line breaks and Indic joiners.
Cancellation/deadlines are cooperative. Limits: input 32 KiB, hints 256,
non-verbatim work segments 4096, logical owned result 512 KiB. Merged cleanup
runs count as one segment; output limits can be reached before work limits.
Engineering failures remain errors in every policy.
