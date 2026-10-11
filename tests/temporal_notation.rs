//! Independent date-tail, clock-range and intent-policy goldens.
use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, NormalizeError, NormalizeOptions, Normalizer, SourceRange,
};

fn primary(input: &str, expected: &str) {
    let normalizer = Normalizer::new().unwrap();
    for ambiguity_policy in [
        AmbiguityPolicy::Preserve,
        AmbiguityPolicy::Reject,
        AmbiguityPolicy::Fallback,
    ] {
        let result = normalizer
            .normalize(
                input,
                &NormalizeOptions {
                    ambiguity_policy,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(result.complete(), "{input}: {:?}", result.issues());
        assert!(result.fallbacks().is_empty(), "{input}");
        assert_eq!(result.normalized_text(), expected, "{input}");
    }
}

#[test]
fn complete_dates_use_written_component_validation_and_spoken_year_attachment() {
    for (source, suffix, expected) in [
        ("2026-12-31", "i", "otuz bir Aralık iki bin yirmi altıyı"),
        ("2026-12-31", "e", "otuz bir Aralık iki bin yirmi altıya"),
        ("2026-12-31", "de", "otuz bir Aralık iki bin yirmi altıda"),
        ("2026-12-31", "den", "otuz bir Aralık iki bin yirmi altıdan"),
        ("2026-12-31", "in", "otuz bir Aralık iki bin yirmi altının"),
        (
            "2026-12-31",
            "deki",
            "otuz bir Aralık iki bin yirmi altıdaki",
        ),
        ("2026-12-31", "dir", "otuz bir Aralık iki bin yirmi altıdır"),
        (
            "2026-12-31",
            "dekidir",
            "otuz bir Aralık iki bin yirmi altıdakidir",
        ),
        ("31.12.2026", "yı", "otuz bir Aralık iki bin yirmi altıyı"),
        ("31.12.2026", "ya", "otuz bir Aralık iki bin yirmi altıya"),
        ("31.12.2026", "da", "otuz bir Aralık iki bin yirmi altıda"),
        ("31.12.2026", "dan", "otuz bir Aralık iki bin yirmi altıdan"),
        ("31.12.2026", "nın", "otuz bir Aralık iki bin yirmi altının"),
        (
            "31/12/2026",
            "dakidir",
            "otuz bir Aralık iki bin yirmi altıdakidir",
        ),
        (
            "2026/12/31",
            "dekidir",
            "otuz bir Aralık iki bin yirmi altıdakidir",
        ),
    ] {
        primary(
            &format!("tarih {source}'{suffix}"),
            &format!("tarih {expected}"),
        );
    }
    primary("2026/12/31", "otuz bir Aralık iki bin yirmi altı");
    primary("2026/12/31'de", "otuz bir Aralık iki bin yirmi altıda");
    primary(
        "31/12/2026 tarihinde",
        "otuz bir Aralık iki bin yirmi altı tarihinde",
    );
    primary(
        "31/12/2026 tarihine",
        "otuz bir Aralık iki bin yirmi altı tarihine",
    );
    primary("tarih 0001-01-01", "tarih bir Ocak bir");
    primary("tarih 0400-02-29", "tarih yirmi dokuz Şubat dört yüz");
    primary(
        "31.12.2026'da saat 14.30'da",
        "otuz bir Aralık iki bin yirmi altıda saat on dört otuzda",
    );
}

#[test]
fn suffixed_clocks_and_compact_spaced_ranges_are_primary() {
    for (input, expected) in [
        ("14.30'da", "on dört otuzda"),
        ("14.00'te", "on dörtte"),
        ("00.00'da", "sıfırda"),
        ("4.00'te", "dörtte"),
        ("14.30-16.45", "on dört otuz ila on altı kırk beş"),
        ("14:30–16:45", "on dört otuz ila on altı kırk beş"),
        (
            "14.30 - 16.45 arasında",
            "on dört otuz ile on altı kırk beş arasında",
        ),
        ("14:30- 16.45", "on dört otuz ila on altı kırk beş"),
        ("14.30 -16:45", "on dört otuz ila on altı kırk beş"),
        ("23.00-01.00", "yirmi üç ila bir"),
        ("14.30-14.30", "on dört otuz ila on dört otuz"),
    ] {
        primary(input, expected);
    }
}

#[test]
fn range_hints_share_connector_context_and_reject_partial_owners() {
    let normalizer = Normalizer::new().unwrap();
    let source = "14.30 - 16.45 arasında";
    for ambiguity_policy in [
        AmbiguityPolicy::Preserve,
        AmbiguityPolicy::Reject,
        AmbiguityPolicy::Fallback,
    ] {
        let result = normalizer
            .normalize(
                source,
                &NormalizeOptions {
                    ambiguity_policy,
                    hints: vec![Hint::new(
                        SourceRange::new(0, "14.30 - 16.45".len()),
                        HintKind::Range,
                    )],
                },
            )
            .unwrap();
        assert_eq!(
            result.normalized_text(),
            "on dört otuz ile on altı kırk beş arasında"
        );
        assert!(result.complete());
        for (range, kind) in [
            (SourceRange::new(0, 5), HintKind::Time),
            (SourceRange::new(8, 13), HintKind::Time),
        ] {
            assert_eq!(
                normalizer.normalize(
                    source,
                    &NormalizeOptions {
                        ambiguity_policy,
                        hints: vec![Hint::new(range, kind)],
                    }
                ),
                Err(NormalizeError::InvalidHint)
            );
        }
    }
}

#[test]
fn unsupported_temporal_shapes_and_suffixes_are_whole_failed_owners() {
    let normalizer = Normalizer::new().unwrap();
    for source in [
        "tarih 2026-12-31'da",
        "tarih 31.12.2026'de",
        "tarih 31/12/2026'lar",
        "tarih 31.12.2026'm",
        "tarih 31.12.2026'dakiler",
        "tarih 31.12.2026'ydı",
        "2026/2/03",
        "2026/02/3",
        "1900/02/29",
        "0000/01/01",
        "14.00'da",
        "14.30'dan",
        "14.30'ta",
        "14.30",
        "31/12/2026",
        "31/12/2026 tarihin",
        "14.30-24.00",
        "14.30-16.60",
        "14:30:00-16:45",
    ] {
        let preserved = normalizer
            .normalize(source, &NormalizeOptions::default())
            .unwrap();
        assert!(!preserved.complete(), "{source}");
        assert_eq!(preserved.normalized_text(), source, "{source}");
        assert_eq!(
            normalizer.normalize(
                source,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::Unresolved(preserved.issues().to_vec())),
            "{source}"
        );
        let fallback = normalizer
            .normalize(
                source,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Fallback,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(fallback.complete(), "{source}");
        assert!(!fallback.fallbacks().is_empty(), "{source}");
    }
}
