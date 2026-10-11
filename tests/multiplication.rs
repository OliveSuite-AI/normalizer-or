//! Flat products, strict whole operands and component source coordinates.
use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, NormalizeError, NormalizeOptions, Normalizer, SegmentKind,
    SourceRange,
};

#[test]
fn flat_products_keep_quantity_readings_and_original_components() {
    let n = Normalizer::new().unwrap();
    for (source, expected) in [
        (
            "12 x 833,25 TL",
            "on iki çarpı sekiz yüz otuz üç Türk lirası yirmi beş kuruş",
        ),
        (
            "12*2*833,25 TL",
            "on iki çarpı iki çarpı sekiz yüz otuz üç Türk lirası yirmi beş kuruş",
        ),
        (
            "833,25 TL × 12",
            "sekiz yüz otuz üç Türk lirası yirmi beş kuruş çarpı on iki",
        ),
        (
            "12 x 1 234,50 TL",
            "on iki çarpı bin iki yüz otuz dört Türk lirası elli kuruş",
        ),
        ("1.234 x 2", "bin iki yüz otuz dört çarpı iki"),
        ("1.234 x 2.", "bin iki yüz otuz dört çarpı iki."),
        ("-2 * 1,05", "eksi iki çarpı bir virgül sıfır beş"),
        ("2 × 5kg", "iki çarpı beş kilogram"),
        ("2 x $25", "iki çarpı yirmi beş dolar"),
    ] {
        for policy in [
            AmbiguityPolicy::Preserve,
            AmbiguityPolicy::Reject,
            AmbiguityPolicy::Fallback,
        ] {
            let r = n
                .normalize(
                    source,
                    &NormalizeOptions {
                        ambiguity_policy: policy,
                        ..Default::default()
                    },
                )
                .unwrap();
            assert_eq!(r.normalized_text(), expected, "{source} {policy:?}");
            assert!(r.complete() && r.issues().is_empty() && r.fallbacks().is_empty());
            let mut cursor = 0;
            for segment in r.segments() {
                assert_eq!(segment.range().start(), cursor);
                cursor = segment.range().end();
                assert!(source.is_char_boundary(cursor));
            }
            assert_eq!(cursor, source.len());
            assert!(r.segments().iter().any(|s| s.kind() == SegmentKind::Symbol));
        }
    }
}

#[test]
fn invalid_products_are_atomic_and_do_not_evaluate_or_swallow_values() {
    let n = Normalizer::new().unwrap();
    for source in [
        "2 x AB12",
        "* 2",
        "x 2",
        "2 x abc@ema.com",
        "2 * nope",
        "12 x 2..",
        "3 x abc2@ema.com",
        "2 x 0850 222 33 44",
        "2 x 833,255 TL",
        "2 x 1 23,50 TL",
        "2 ** 3",
        "2 x x 3",
        "2 kg x 3 kg",
        "-2 x 5 kg",
    ] {
        let r = n.normalize(source, &NormalizeOptions::default()).unwrap();
        assert!(!r.complete(), "{source}");
        assert_eq!(r.normalized_text(), source, "{source}");
        assert_eq!(r.issues().len(), 1, "{source}");
        assert_eq!(
            r.issues()[0].range(),
            SourceRange::new(0, source.len()),
            "{source}"
        );
        assert_eq!(
            n.normalize(
                source,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::Unresolved(r.issues().to_vec())),
        );
    }
    let source = "12 x 833,25 TL";
    for policy in [
        AmbiguityPolicy::Preserve,
        AmbiguityPolicy::Reject,
        AmbiguityPolicy::Fallback,
    ] {
        assert_eq!(
            n.normalize(
                source,
                &NormalizeOptions {
                    ambiguity_policy: policy,
                    hints: vec![Hint::new(SourceRange::new(0, 2), HintKind::Cardinal)],
                }
            ),
            Err(NormalizeError::InvalidHint)
        );
    }
    let code = n.normalize("12x34", &NormalizeOptions::default()).unwrap();
    assert_eq!(code.normalized_text(), "12x34");
}
