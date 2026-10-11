//! Whole account capture and punctuation versus independent following notation.
use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, NormalizeError, NormalizeOptions, Normalizer, SegmentKind,
    SourceRange,
};

#[test]
fn sentence_period_does_not_release_account_groups_or_certify_invalid_prefixes() {
    let n = Normalizer::new().unwrap();
    let account = "TR33 0006 1005 1978 6457 8413 26";
    let expected = "te re üç üç, sıfır sıfır sıfır altı, bir sıfır sıfır beş, bir dokuz yedi sekiz, altı dört beş yedi, sekiz dört bir üç, iki altı";
    for policy in [
        AmbiguityPolicy::Preserve,
        AmbiguityPolicy::Reject,
        AmbiguityPolicy::Fallback,
    ] {
        let options = NormalizeOptions {
            ambiguity_policy: policy,
            ..Default::default()
        };
        let r = n.normalize(&format!("{account}."), &options).unwrap();
        assert_eq!(r.normalized_text(), format!("{expected}."));
        assert_eq!(r.segments()[0].kind(), SegmentKind::Iban);
        assert_eq!(r.segments()[0].range(), SourceRange::new(0, account.len()));
        assert_eq!(
            n.normalize(
                &format!("{account}."),
                &NormalizeOptions {
                    ambiguity_policy: policy,
                    hints: vec![Hint::new(SourceRange::new(30, 33), HintKind::Ordinal)],
                }
            ),
            Err(NormalizeError::InvalidHint)
        );
        let following = n.normalize(&format!("{account} 25 TL"), &options).unwrap();
        assert_eq!(
            following.normalized_text(),
            format!("{expected} yirmi beş Türk lirası")
        );
    }
    for suffix in [" 00", " 00.", " 00..", " 00abc", " 00'ya", "'ya"] {
        let source = format!("{account}{suffix}");
        let r = n.normalize(&source, &NormalizeOptions::default()).unwrap();
        assert!(!r.complete(), "{source}");
        assert!(
            !r.segments().iter().any(|s| s.kind() == SegmentKind::Iban),
            "{source}"
        );
        assert!(
            !r.segments().iter().any(|s| s.kind() == SegmentKind::Money),
            "{source}"
        );
        assert_eq!(r.issues().len(), 1, "{source}");
        assert_eq!(
            n.normalize(
                &source,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::Unresolved(r.issues().to_vec()))
        );
        let fallback = n
            .normalize(
                &source,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Fallback,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(fallback.fallbacks().len(), 1);
        assert_eq!(fallback.fallbacks()[0].range(), r.issues()[0].range());
        assert!(
            !fallback
                .segments()
                .iter()
                .any(|s| s.kind() == SegmentKind::Iban)
        );
    }
    let labeled = n
        .normalize(&format!("{account} TL"), &NormalizeOptions::default())
        .unwrap();
    assert_eq!(labeled.segments()[0].kind(), SegmentKind::Iban);
    assert!(
        !labeled
            .segments()
            .iter()
            .any(|s| s.kind() == SegmentKind::Money)
    );
    for tail in ["25TL", "5kg"] {
        let r = n
            .normalize(&format!("{account} {tail}"), &NormalizeOptions::default())
            .unwrap();
        assert!(r.complete());
        assert_eq!(r.segments()[0].kind(), SegmentKind::Iban);
        assert!(
            r.segments()
                .iter()
                .any(|s| matches!(s.kind(), SegmentKind::Money | SegmentKind::Unit))
        );
    }
    let invalid = "TR12 0006 4000 0011 2345 6789 01.";
    let r = n
        .normalize(
            invalid,
            &NormalizeOptions {
                ambiguity_policy: AmbiguityPolicy::Fallback,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!r.segments().iter().any(|s| s.kind() == SegmentKind::Iban));
    assert!(!r.normalized_text().ends_with("nokta"));
    assert_eq!(r.fallbacks().len(), 1);
}
