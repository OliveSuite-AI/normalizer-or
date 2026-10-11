//! Cross-reader admission regressions reproduced in the independent PR review.
use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, IssueCategory, NormalizeError, NormalizeOptions, Normalizer,
    SegmentKind, SourceRange,
};

const POLICIES: [AmbiguityPolicy; 3] = [
    AmbiguityPolicy::Preserve,
    AmbiguityPolicy::Reject,
    AmbiguityPolicy::Fallback,
];

#[test]
fn products_stop_before_independently_owned_address_fields() {
    let n = Normalizer::new().unwrap();
    for (source, expected) in [
        (
            "adres No:5 2 x 3 Kat:3",
            "adres numara beş iki çarpı üç kat üç",
        ),
        (
            "adres No:5 2 × 3 Kat:3",
            "adres numara beş iki çarpı üç kat üç",
        ),
        (
            "adres No:5 2*3 Kat: 3",
            "adres numara beş iki çarpı üç kat üç",
        ),
        (
            "adres No:5 2 x 3 245. Sk.",
            "adres numara beş iki çarpı üç iki yüz kırk beşinci sokak",
        ),
    ] {
        for ambiguity_policy in POLICIES {
            let result = n
                .normalize(
                    source,
                    &NormalizeOptions {
                        ambiguity_policy,
                        ..Default::default()
                    },
                )
                .unwrap();
            assert_eq!(result.normalized_text(), expected);
            assert!(
                result.complete() && result.issues().is_empty() && result.fallbacks().is_empty()
            );
            let mut cursor = 0;
            for segment in result.segments() {
                assert_eq!(segment.range().start(), cursor);
                cursor = segment.range().end();
            }
            assert_eq!(cursor, source.len());
            let product_start = source.find('2').unwrap();
            assert_eq!(
                n.normalize(
                    source,
                    &NormalizeOptions {
                        ambiguity_policy,
                        hints: vec![Hint::new(
                            SourceRange::new(product_start, product_start + 1),
                            HintKind::Cardinal
                        )],
                    }
                ),
                Err(NormalizeError::InvalidHint)
            );
        }
    }
}

#[test]
fn clock_range_admission_does_not_claim_grouped_numbers_or_date_prose() {
    let n = Normalizer::new().unwrap();
    for (source, expected) in [
        ("2.000-3.000 kişi", "iki bin ila üç bin kişi"),
        (
            "1.234 - 2.345 kg",
            "bin iki yüz otuz dört ila iki bin üç yüz kırk beş kilogram",
        ),
        (
            "tarih 31.12.2026 - toplantı",
            "tarih otuz bir Aralık iki bin yirmi altı - toplantı",
        ),
    ] {
        for ambiguity_policy in POLICIES {
            let result = n
                .normalize(
                    source,
                    &NormalizeOptions {
                        ambiguity_policy,
                        ..Default::default()
                    },
                )
                .unwrap();
            assert_eq!(result.normalized_text(), expected);
            assert!(
                result.complete() && result.issues().is_empty() && result.fallbacks().is_empty()
            );
        }
    }
    for source in ["24:00-17:00", "13:00-17:60", "13:00:00-17:00:00"] {
        let result = n.normalize(source, &NormalizeOptions::default()).unwrap();
        assert_eq!(
            result.issues()[0].range(),
            SourceRange::new(0, source.len())
        );
    }
}

#[test]
fn mixed_arithmetic_never_certifies_a_product_prefix() {
    let n = Normalizer::new().unwrap();
    for source in [
        "2 x 3 = 6",
        "2 x 3 + 4",
        "2 x 3 / 4",
        "2 x 3 - 4",
        "2*3=6",
        "2*3 + 4",
        "2*3+4",
        "2*3 +4",
        "2 x 3 kg +4",
        "2 x 3 =",
    ] {
        let preserved = n.normalize(source, &NormalizeOptions::default()).unwrap();
        assert_eq!(preserved.normalized_text(), source, "{source}");
        assert_eq!(preserved.issues().len(), 1);
        assert_eq!(
            preserved.issues()[0].range(),
            SourceRange::new(0, source.len())
        );
        assert_eq!(preserved.issues()[0].category(), IssueCategory::Unsupported);
        assert_eq!(
            n.normalize(
                source,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::Unresolved(preserved.issues().to_vec()))
        );
        let fallback = n
            .normalize(
                source,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Fallback,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(fallback.fallbacks().len(), 1);
        assert_eq!(
            fallback.fallbacks()[0].range(),
            SourceRange::new(0, source.len())
        );
        assert!(
            !fallback
                .segments()
                .iter()
                .any(|segment| segment.rule_id() == "expression.multiplication")
        );
    }
}

#[test]
fn protected_phone_operands_are_not_signed_cardinals() {
    let n = Normalizer::new().unwrap();
    for source in [
        "2 x +905551234567",
        "+905551234567*2",
        "2 × +90 555 123 45 67",
        "2 x 0850 222 33 44",
        "08502223344*2",
    ] {
        let preserved = n.normalize(source, &NormalizeOptions::default()).unwrap();
        assert_eq!(preserved.normalized_text(), source);
        assert_eq!(preserved.issues().len(), 1);
        assert_eq!(
            preserved.issues()[0].range(),
            SourceRange::new(0, source.len())
        );
        assert!(
            !preserved
                .segments()
                .iter()
                .any(|segment| segment.kind() == SegmentKind::Cardinal)
        );
        assert_eq!(
            n.normalize(
                source,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::Unresolved(preserved.issues().to_vec()))
        );
        let fallback = n
            .normalize(
                source,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Fallback,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(
            !fallback
                .segments()
                .iter()
                .any(|segment| segment.kind() == SegmentKind::Cardinal)
        );
    }
    for source in ["2 x +90", "2 x +9055", "-2*3"] {
        assert!(
            n.normalize(source, &NormalizeOptions::default())
                .unwrap()
                .complete()
        );
    }
}
