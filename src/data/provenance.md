# Odia data provenance and review status

The active Odia vocabulary is a small project-authored table in `src/vocabulary.rs`.
Number words 0–99 and hundred/thousand/lakh/crore composition, rupee/paise,
unit names, time labels, and fallback readings are explicit alpha defaults.
They have automated text goldens, not completed native-speaker or audio review.
Do not claim production pronunciation quality from `complete=true` or passing tests.

Gregorian month spellings were cross-checked against Unicode CLDR **release-48**:
https://github.com/unicode-org/cldr/blob/release-48/common/main/or.xml
Copyright © 1991–2025 Unicode, Inc.; Unicode License V3. CLDR is a pinned
reference for locale vocabulary, not a number-spellout runtime or sentence oracle.
The Unicode license is retained in THIRD_PARTY_NOTICES.md and Python wheel notices.
No whole CLDR file, bulk dictionary, learned model, or external engine is bundled.

Odia decimal digits U+0B66–U+0B6F are mapped by scalar value. NFC and grapheme
segmentation come from the locked unicode-normalization/unicode-segmentation
libraries. Joiners, virama, nukta, and vowel signs are preserved; the implementation
does not invent equivalences from similar-looking glyphs.

Source-coordinate mapping, public result records, and cooperative controls
retain their Apache-2.0 source attribution in THIRD_PARTY_NOTICES.md. Odia
recognition, vocabulary, rendering, and regression expectations are maintained
in this repository.
