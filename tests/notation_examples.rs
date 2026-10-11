//! Independent complete-sentence goldens for the supported notation families.
use normalizer_tr::{AmbiguityPolicy, NormalizeOptions, Normalizer};
use serde_json::Value;

#[test]
fn complete_notation_sentences_have_independent_primary_goldens_in_every_policy() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/notation-examples.json")).unwrap();
    let n = Normalizer::new().unwrap();
    for case in cases {
        let source = case["text"].as_str().unwrap();
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
            assert_eq!(
                r.normalized_text(),
                case["expected"].as_str().unwrap(),
                "{source} {policy:?}"
            );
            assert!(
                r.complete() && r.issues().is_empty() && r.fallbacks().is_empty(),
                "{source}: issues={:?}, fallbacks={:?}",
                r.issues(),
                r.fallbacks()
            );
        }
    }
}
