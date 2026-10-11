//! Exact failed-phone and electronic-trimming semantics on adversarial input sizes.
use normalizer_tr::{
    AmbiguityPolicy, LimitKind, MAX_INPUT_BYTES, NormalizeError, NormalizeOptions, Normalizer,
    SegmentKind, SourceRange,
};

#[test]
fn declined_short_phone_prefixes_keep_individual_diagnostics_and_limits() {
    let n = Normalizer::new().unwrap();
    for count in [1, 128, 512] {
        let input = "01 1 ".repeat(count);
        let result = n.normalize(&input, &NormalizeOptions::default()).unwrap();
        assert_eq!(result.normalized_text(), "01 bir ".repeat(count));
        assert_eq!(result.issues().len(), count);
        for (index, issue) in result.issues().iter().enumerate() {
            assert_eq!(issue.range(), SourceRange::new(index * 5, index * 5 + 2));
        }
        assert_eq!(
            n.normalize(
                &input,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::Unresolved(result.issues().to_vec()))
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
        assert_eq!(fallback.fallbacks().len(), count);
    }
    for ambiguity_policy in [
        AmbiguityPolicy::Preserve,
        AmbiguityPolicy::Reject,
        AmbiguityPolicy::Fallback,
    ] {
        let input = "01 1 ".repeat(MAX_INPUT_BYTES / 5);
        assert_eq!(
            n.normalize(
                &input,
                &NormalizeOptions {
                    ambiguity_policy,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::LimitExceeded(LimitKind::Candidates))
        );
    }
}

#[test]
fn excess_electronic_parentheses_are_prose_without_repeated_prefix_counts() {
    let n = Normalizer::new().unwrap();
    let prefix = "https://ornek.com/";
    let speech = "ha te te pe es iki nokta eğik çizgi eğik çizgi ornek nokta kom eğik çizgi";
    for bytes in [4096, 8192, 16384, MAX_INPUT_BYTES] {
        let tail = ")".repeat(bytes - prefix.len());
        let input = format!("{prefix}{tail}");
        for ambiguity_policy in [
            AmbiguityPolicy::Preserve,
            AmbiguityPolicy::Reject,
            AmbiguityPolicy::Fallback,
        ] {
            let result = n
                .normalize(
                    &input,
                    &NormalizeOptions {
                        ambiguity_policy,
                        ..Default::default()
                    },
                )
                .unwrap();
            assert_eq!(result.normalized_text(), format!("{speech}{tail}"));
            assert!(result.complete() && result.fallbacks().is_empty());
            assert_eq!(result.segments()[0].kind(), SegmentKind::Electronic);
            assert_eq!(
                result.segments()[0].range(),
                SourceRange::new(0, prefix.len())
            );
        }
    }
    for (input, expected) in [
        (
            "https://ornek.com/(a))",
            "ha te te pe es iki nokta eğik çizgi eğik çizgi ornek nokta kom eğik çizgi aç parantez a kapat parantez)",
        ),
        (
            "https://ornek.com/(a).",
            "ha te te pe es iki nokta eğik çizgi eğik çizgi ornek nokta kom eğik çizgi aç parantez a kapat parantez.",
        ),
    ] {
        assert_eq!(
            n.normalize(input, &NormalizeOptions::default())
                .unwrap()
                .normalized_text(),
            expected
        );
    }
}
