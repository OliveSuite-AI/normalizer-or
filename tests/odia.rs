//! Odia contracts, provenance invariants, and Unicode regressions.
#[allow(dead_code)]
mod support;

use normalizer_or::*;
use serde_json::Value;
use unicode_segmentation::UnicodeSegmentation;

fn check_partition(input: &str, result: &NormalizeResult) {
    let mut boundaries: Vec<_> = input.grapheme_indices(true).map(|(i, _)| i).collect();
    boundaries.push(input.len());
    let mut cursor = 0;
    for segment in result.segments() {
        assert_eq!(segment.range().start(), cursor, "{input:?}");
        assert!(boundaries.contains(&cursor));
        cursor = segment.range().end();
        assert!(boundaries.contains(&cursor));
        if matches!(
            segment.kind(),
            SegmentKind::Verbatim | SegmentKind::Unresolved
        ) {
            assert_eq!(segment.text(), &input[segment.range().start()..cursor]);
        }
    }
    assert_eq!(cursor, input.len());
    assert_eq!(
        result
            .segments()
            .iter()
            .map(|s| s.text())
            .collect::<String>(),
        result.normalized_text()
    );
    assert_eq!(result.complete(), result.issues().is_empty());
    assert_eq!(result.locale(), "or-IN");
    assert_eq!(result.normalizer_id(), normalizer_or::NORMALIZER_ID);
}

#[test]
fn shared_contract_all_policies() {
    let normalizer = Normalizer::new().unwrap();
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/odia.json")).unwrap();
    for case in cases {
        support::check_case(&normalizer, &case);
        // Retain original-source partition checks independently of expectations.
        for policy in [AmbiguityPolicy::Preserve, AmbiguityPolicy::Fallback] {
            let result = normalizer
                .normalize(
                    case["input"].as_str().unwrap(),
                    &NormalizeOptions {
                        ambiguity_policy: policy,
                        ..Default::default()
                    },
                )
                .unwrap();
            check_partition(case["input"].as_str().unwrap(), &result);
        }
    }
}

#[test]
fn cleanup_mapping_and_options() {
    let n = Normalizer::new().unwrap();
    let input = "  ଡ\u{0b3c} କ୍\u{200d}ଷ କ୍\u{200c}ଷ e\u{301}\r\n୧2  ";
    for whitespace in [WhitespacePolicy::Preserve, WhitespacePolicy::Collapse] {
        for digits in [DigitStyle::Preserve, DigitStyle::Ascii, DigitStyle::Odia] {
            let options = CleanOptions { whitespace, digits };
            let result = n.clean(input, &options).unwrap();
            check_partition(input, &result);
            assert!(result.normalized_text().contains("କ୍\u{200d}ଷ କ୍\u{200c}ଷ é"));
            assert_eq!(
                n.clean(result.normalized_text(), &options)
                    .unwrap()
                    .normalized_text(),
                result.normalized_text()
            );
            let end = match digits {
                DigitStyle::Preserve => "୧2",
                DigitStyle::Ascii => "12",
                DigitStyle::Odia => "୧୨",
            };
            assert!(result.normalized_text().contains(end));
            if whitespace == WhitespacePolicy::Collapse {
                assert!(!result.normalized_text().contains("\r\n"));
                assert!(result.normalized_text().starts_with(' '));
            } else {
                assert!(result.normalized_text().contains("\r\n"));
            }
        }
    }
    let input = "ଓଡ଼ିଆ English।";
    assert_eq!(
        n.clean(input, &CleanOptions::default())
            .unwrap()
            .normalized_text(),
        input
    );
}

#[test]
fn hints_use_original_bytes_and_whole_expressions() {
    let n = Normalizer::new().unwrap();
    let input = "e\u{301} ଓଡ଼ିଆ ୦୧୨";
    let start = input.find('୦').unwrap();
    let result = n
        .normalize(
            input,
            &NormalizeOptions {
                hints: vec![Hint::new(
                    SourceRange::new(start, input.len()),
                    HintKind::Cardinal,
                )],
                ..Default::default()
            },
        )
        .unwrap();
    check_partition(input, &result);
    assert!(result.normalized_text().ends_with("ବାର"));
    assert!(result.normalized_text().starts_with("e\u{301}"));
    for (text, start, end, kind) in [
        ("e\u{301} 12", 0, 1, HintKind::Digits),
        ("5 kg", 0, 1, HintKind::Cardinal),
        ("AB12", 2, 4, HintKind::Cardinal),
        ("29/02/2025", 0, 10, HintKind::Date),
        ("1", 0, 2, HintKind::Cardinal),
    ] {
        assert_eq!(
            n.normalize(
                text,
                &NormalizeOptions {
                    hints: vec![Hint::new(SourceRange::new(start, end), kind)],
                    ..Default::default()
                }
            ),
            Err(NormalizeError::InvalidHint),
            "{text:?}"
        );
    }
    let input = "+୦୧ 23";
    assert_eq!(
        n.normalize(
            input,
            &NormalizeOptions {
                hints: vec![Hint::new(
                    SourceRange::new(0, input.len()),
                    HintKind::Digits
                )],
                ..Default::default()
            }
        )
        .unwrap()
        .normalized_text(),
        "ଯୁକ୍ତ ଶୂନ୍ୟ ଏକ ଦୁଇ ତିନି"
    );
}

#[test]
fn controls_limits_and_partial_results() {
    let n = Normalizer::new().unwrap();
    let control = WorkControl::default();
    control.cancel();
    assert_eq!(
        n.normalize_controlled("5", &Default::default(), &control),
        Err(NormalizeError::Cancelled)
    );
    assert_eq!(
        n.clean_controlled("5", &Default::default(), &control),
        Err(NormalizeError::Cancelled)
    );
    let control = WorkControl::new(Some(std::time::Instant::now()));
    assert_eq!(
        n.normalize_controlled("5", &Default::default(), &control),
        Err(NormalizeError::Cancelled)
    );
    for input in ["", " \t\n", "hi\0", "hi\u{202e}"] {
        assert_eq!(
            n.normalize(input, &Default::default()),
            Err(NormalizeError::InvalidInput)
        );
        assert_eq!(
            n.clean(input, &Default::default()),
            Err(NormalizeError::InvalidInput)
        );
    }
    let input = "a".repeat(MAX_INPUT_BYTES + 1);
    assert_eq!(
        n.normalize(&input, &Default::default()),
        Err(NormalizeError::LimitExceeded(LimitKind::Input))
    );
    let result = n
        .normalize("5 kg; 29/02/2025; ୫", &Default::default())
        .unwrap();
    check_partition("5 kg; 29/02/2025; ୫", &result);
    assert_eq!(result.normalized_text(), "ପାଞ୍ଚ କିଲୋଗ୍ରାମ; 29/02/2025; ପାଞ୍ଚ");
    assert_eq!(result.issues().len(), 1);
    assert!(matches!(
        n.normalize(&"1 ".repeat(5000), &Default::default()),
        Err(NormalizeError::LimitExceeded(_))
    ));
    assert!(matches!(
        n.normalize(
            "1",
            &NormalizeOptions {
                hints: vec![Hint::new(SourceRange::new(0, 1), HintKind::Cardinal); MAX_HINTS + 1],
                ..Default::default()
            }
        ),
        Err(NormalizeError::LimitExceeded(LimitKind::Hints))
    ));
}

#[test]
fn fallback_unknown_scalars_and_preserved_prose() {
    let n = Normalizer::new().unwrap();
    let result = n
        .normalize(
            "🙂 5",
            &NormalizeOptions {
                ambiguity_policy: AmbiguityPolicy::Fallback,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(result.normalized_text().starts_with("ୟୁନିକୋଡ୍"));
    assert_eq!(
        result.fallbacks()[0].strategy(),
        FallbackStrategy::UnicodeCodePoint
    );
    for input in ["API CPU Dr. Python", "P and S"] {
        assert_eq!(
            n.normalize(input, &Default::default())
                .unwrap()
                .normalized_text(),
            input
        );
    }
}

#[cfg(feature = "serde")]
#[test]
fn serialization_preserves_coordinates_and_identity() {
    let result = Normalizer::new()
        .unwrap()
        .normalize("୫", &Default::default())
        .unwrap();
    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["locale"], "or-IN");
    assert_eq!(json["segments"][0]["range"]["end"], 3);
}

#[test]
fn nested_wrappers_and_fallback_marks_are_bounded_and_faithful() {
    let n = Normalizer::new().unwrap();
    let input = format!("{}5{}", "(".repeat(5000), ")".repeat(5000));
    let result = n.normalize(&input, &Default::default()).unwrap();
    check_partition(&input, &result);
    assert_eq!(
        result.normalized_text(),
        format!("{}ପାଞ୍ଚ{}", "(".repeat(5000), ")".repeat(5000))
    );
    let result = n
        .normalize(
            "୫ରେ",
            &NormalizeOptions {
                ambiguity_policy: AmbiguityPolicy::Fallback,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(result.normalized_text(), "ପାଞ୍ଚ ରେ");
    for input in ["୫\u{0301}", "5\u{0b3e}", "(5)\u{0301}"] {
        let result = n.normalize(input, &Default::default()).unwrap();
        check_partition(input, &result);
        assert_eq!(result.normalized_text(), input);
    }
}

#[test]
fn malformed_owners_keep_neighbors_and_original_coordinates() {
    let n = Normalizer::new().unwrap();
    for body in ["₹1 234", "% 12 %", "5 kg kg"] {
        let prefix = "e\u{301} [";
        let input = format!("{prefix}{body}]; 6 kg");
        for policy in [AmbiguityPolicy::Preserve, AmbiguityPolicy::Fallback] {
            let result = n
                .normalize(
                    &input,
                    &NormalizeOptions {
                        ambiguity_policy: policy,
                        ..Default::default()
                    },
                )
                .unwrap();
            check_partition(&input, &result);
            let range = SourceRange::new(prefix.len(), prefix.len() + body.len());
            if policy == AmbiguityPolicy::Preserve {
                assert_eq!(result.issues()[0].range(), range);
            } else {
                assert_eq!(result.fallbacks()[0].range(), range);
            }
            assert!(result.normalized_text().ends_with("; ଛଅ କିଲୋଗ୍ରାମ"));
        }
        assert_eq!(
            n.normalize(
                &input,
                &NormalizeOptions {
                    hints: vec![Hint::new(
                        SourceRange::new(prefix.len(), prefix.len() + 1),
                        HintKind::Cardinal
                    )],
                    ..Default::default()
                }
            ),
            Err(NormalizeError::InvalidHint)
        );
    }
}

#[test]
fn latex_math_is_spoken_as_one_formula_segment() {
    let n = Normalizer::new().unwrap();
    let formula = r"\frac{1}{6}x^2 + \frac{5}{6}x + 1 = \left(\frac{1}{6}x^2 + \frac{5}{6}x\right) + \left(1\right)";
    for block in [
        format!("$$\n{formula}\n$$"),
        format!("\\[{formula}\\]"),
        format!("\\({formula}\\)"),
        format!("${formula}$"),
    ] {
        let input = format!("Before {block} after 6");
        let start = "Before ".len();
        let end = start + block.len();
        let result = n.normalize(&input, &Default::default()).unwrap();
        check_partition(&input, &result);
        assert_eq!(result.issues().len(), 0, "{block:?}");
        assert!(result.complete());
        assert_eq!(
            result
                .segments()
                .iter()
                .find(|s| s.kind() == SegmentKind::Formula)
                .unwrap()
                .range(),
            SourceRange::new(start, end)
        );
        assert!(result.normalized_text().contains("ଯୁକ୍ତ"));
        assert_eq!(
            result.normalized_text().rsplit_once(" after ").unwrap().1,
            "ଛଅ"
        );

        let fallback = n
            .normalize(
                &input,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Fallback,
                    ..Default::default()
                },
            )
            .unwrap();
        check_partition(&input, &fallback);
        assert!(fallback.fallbacks().is_empty(), "{block:?}");
        assert!(fallback.normalized_text().ends_with("after ଛଅ"));

        assert!(
            n.normalize(
                &input,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            )
            .is_ok()
        );
    }

    let superscript = "$x² + 5x + 6$";
    let result = n.normalize(superscript, &Default::default()).unwrap();
    assert!(result.complete());
    assert_eq!(result.issues().len(), 0);
    assert!(result.normalized_text().contains("ଘାତ"));
}

#[test]
fn compact_plain_formula_is_recognized_but_ambiguous_token_is_not() {
    let n = Normalizer::new().unwrap();
    let result = n.normalize("x²+5x+6", &Default::default()).unwrap();
    assert_eq!(
        result.normalized_text(),
        "ଏକ୍ସ ର ଘାତ ଦୁଇ ଯୁକ୍ତ ପାଞ୍ଚ ଗୁଣନ ଏକ୍ସ ଯୁକ୍ତ ଛଅ"
    );
    assert_eq!(result.segments()[0].kind(), SegmentKind::Formula);
    let spaced = n.normalize("x² + 5x + 6", &Default::default()).unwrap();
    assert!(spaced.normalized_text().contains("ଘାତ"));
    assert_eq!(spaced.segments()[0].kind(), SegmentKind::Formula);
    let fraction = n.normalize("1/6", &Default::default()).unwrap();
    assert!(fraction.normalized_text().contains("ଲବ"));
    let grouped = n.normalize("x*(x+5)=6", &Default::default()).unwrap();
    assert!(grouped.normalized_text().contains("ସମାନ"));
    assert!(grouped.normalized_text().contains("ବାମ ବନ୍ଧନୀ"));
    let ambiguous = n.normalize("5x", &Default::default()).unwrap();
    assert_eq!(ambiguous.normalized_text(), "5x");
    assert!(ambiguous.issues().is_empty());
}

#[test]
fn malformed_formula_remains_whole_and_unresolved() {
    let n = Normalizer::new().unwrap();
    let source = "$x^ + 2$";
    let result = n.normalize(source, &Default::default()).unwrap();
    assert_eq!(result.normalized_text(), source);
    assert_eq!(result.issues().len(), 1);
    assert_eq!(
        result.issues()[0].range(),
        SourceRange::new(0, source.len())
    );
}

#[test]
fn explicit_math_scalars_continuations_and_boxed_equations_are_spoken() {
    let n = Normalizer::new().unwrap();
    for (source, expected) in [
        ("$5x$", "ପାଞ୍ଚ ଗୁଣନ ଏକ୍ସ"),
        ("$6$", "ଛଅ"),
        ("$$=x+1$$", "ସମାନ ଏକ୍ସ ଯୁକ୍ତ ଏକ"),
        (r"$$\boxed{x^2 + 5x + 6 = 0}$$", "ସମାନ"),
    ] {
        let result = n.normalize(source, &Default::default()).unwrap();
        assert!(result.complete(), "{source}");
        assert!(result.normalized_text().contains(expected), "{source}");
    }
    let heading = n.normalize("ପଦକ୍ଷେପ 1: ଖୋଜନ୍ତୁ", &Default::default()).unwrap();
    assert!(heading.complete());
    assert!(heading.normalized_text().starts_with("ପଦକ୍ଷେପ ଏକ:"));
}

#[test]
fn latex_text_labels_and_metric_units_are_spoken() {
    let n = Normalizer::new().unwrap();
    for (formula, expected) in [
        (
            r"$$\text{Area} = \text{length} \times \text{breadth}$$",
            "କ୍ଷେତ୍ରଫଳ ସମାନ ଲମ୍ବ ଗୁଣନ ପ୍ରସ୍ଥ",
        ),
        (
            r"$$\text{Area} = 8 \times 5 = 40 \text{ cm}^2$$",
            "କ୍ଷେତ୍ରଫଳ ସମାନ ଆଠ ଗୁଣନ ପାଞ୍ଚ ସମାନ ଚାଳିଶ ବର୍ଗ ସେଣ୍ଟିମିଟର",
        ),
        (
            r"$$\text{Perimeter} = 2 \times (8 + 5) = 2 \times 13 = 26 \text{ cm}$$",
            "ପରିଧି ସମାନ ଦୁଇ ଗୁଣନ ବାମ ବନ୍ଧନୀ ଆଠ ଯୁକ୍ତ ପାଞ୍ଚ ଡାହାଣ ବନ୍ଧନୀ",
        ),
        (
            r"$$\text{Area} = \text{length} \times \text{breadth}
\text{Perimeter} = 2 \times (\text{length} + \text{breadth})$$",
            "ପରିଧି ସମାନ",
        ),
    ] {
        let result = n.normalize(formula, &Default::default()).unwrap();
        assert!(result.complete(), "{formula}: {:?}", result.issues());
        assert_eq!(result.issues().len(), 0);
        assert!(result.normalized_text().contains(expected), "{formula}");
    }
}

#[test]
fn phone_protection_and_explicit_whole_digit_hints() {
    let n = Normalizer::new().unwrap();
    for (input, start) in [("+91 98765 43210", 0), ("phone: 9876543210", 7)] {
        for kind in [HintKind::Cardinal, HintKind::Date, HintKind::Time] {
            assert_eq!(
                n.normalize(
                    input,
                    &NormalizeOptions {
                        hints: vec![Hint::new(SourceRange::new(start, input.len()), kind)],
                        ..Default::default()
                    }
                ),
                Err(NormalizeError::InvalidHint)
            );
        }
        assert_eq!(
            n.normalize(
                input,
                &NormalizeOptions {
                    hints: vec![Hint::new(
                        SourceRange::new(start, start + 3),
                        HintKind::Digits
                    )],
                    ..Default::default()
                }
            ),
            Err(NormalizeError::InvalidHint)
        );
        let result = n
            .normalize(
                input,
                &NormalizeOptions {
                    hints: vec![Hint::new(
                        SourceRange::new(start, input.len()),
                        HintKind::Digits,
                    )],
                    ..Default::default()
                },
            )
            .unwrap();
        check_partition(input, &result);
        assert!(result.complete());
        assert!(!result.fallback_used());
    }
    let result = n
        .normalize("+91 98765 43210 5 kg", &Default::default())
        .unwrap();
    assert_eq!(result.issues()[0].range(), SourceRange::new(0, 15));
    assert!(result.normalized_text().ends_with("ପାଞ୍ଚ କିଲୋଗ୍ରାମ"));
}
