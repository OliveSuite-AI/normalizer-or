//! Address context, atomic fields, three-policy goldens and original-source ownership.
use normalizer_tr::{
    AmbiguityPolicy, FallbackClass, FallbackReason, Hint, HintKind, IssueCategory, LimitKind,
    MAX_INPUT_BYTES, NormalizeError, NormalizeOptions, NormalizeResult, Normalizer, SegmentKind,
    SourceRange, WorkControl,
};
use std::time::{Duration, Instant};

const POLICIES: [AmbiguityPolicy; 3] = [
    AmbiguityPolicy::Preserve,
    AmbiguityPolicy::Reject,
    AmbiguityPolicy::Fallback,
];

fn options(policy: AmbiguityPolicy) -> NormalizeOptions {
    NormalizeOptions {
        ambiguity_policy: policy,
        ..Default::default()
    }
}

#[test]
fn unpunctuated_explicit_postal_labels_survive_the_address_scan_fast_path() {
    goldens(&[
        ("posta kodu 03456", "posta kodu sıfır üç dört beş altı"),
        ("Posta kodu 03456", "posta kodu sıfır üç dört beş altı"),
        ("POSTA KODU 03456", "posta kodu sıfır üç dört beş altı"),
    ]);
}

fn partition(input: &str, result: &NormalizeResult) {
    let mut cursor = 0;
    for segment in result.segments() {
        assert_eq!(segment.range().start(), cursor, "{input}");
        assert!(segment.range().end() > cursor, "{input}");
        assert!(input.is_char_boundary(cursor), "{input}");
        assert!(input.is_char_boundary(segment.range().end()), "{input}");
        if segment.kind() == SegmentKind::Verbatim {
            assert_eq!(
                segment.text(),
                &input[cursor..segment.range().end()],
                "{input}"
            );
        }
        cursor = segment.range().end();
    }
    assert_eq!(cursor, input.len(), "{input}");
    assert_eq!(
        result
            .segments()
            .iter()
            .map(|segment| segment.text())
            .collect::<String>(),
        result.normalized_text(),
        "{input}"
    );
}

fn goldens(cases: &[(&str, &str)]) {
    let normalizer = Normalizer::new().unwrap();
    for &(input, expected) in cases {
        let mut first = None;
        for policy in POLICIES {
            let result = normalizer.normalize(input, &options(policy)).unwrap();
            assert_eq!(result.normalized_text(), expected, "{input}: {policy:?}");
            assert!(
                result.complete(),
                "{input}: {policy:?}: {:?}",
                result.issues()
            );
            assert!(result.issues().is_empty(), "{input}: {policy:?}");
            assert!(!result.fallback_used(), "{input}: {policy:?}");
            partition(input, &result);
            if let Some(first) = &first {
                assert_eq!(&result, first, "{input}: {policy:?}");
            } else {
                first = Some(result);
            }
        }
    }
}

#[test]
fn original_address_examples_are_primary_in_all_policies() {
    goldens(&[
        (
            "Adresiniz Atatürk Mah. Cumhuriyet Cad. No:12/4 Kat:3 Daire:7 Kadıköy/İstanbul.",
            "Adresiniz Atatürk mahalle Cumhuriyet cadde numara on iki bölü dört kat üç daire yedi Kadıköy, İstanbul.",
        ),
        (
            "Teslimat adresi: Bağdat Cd. 245. Sk. No: 5 D: 2, 34728 Kadıköy.",
            "Teslimat adresi: Bağdat cadde iki yüz kırk beşinci sokak numara beş daire iki, posta kodu üç dört yedi iki sekiz Kadıköy.",
        ),
    ]);
}

#[test]
fn reviewed_alias_case_families_and_colon_spacing() {
    goldens(&[
        (
            "adres Atatürk mah. No:12",
            "adres Atatürk mahalle numara on iki",
        ),
        (
            "Adres Atatürk Mh. No :12",
            "Adres Atatürk mahalle numara on iki",
        ),
        (
            "ADRES ATATÜRK MH. NO : 12",
            "ADRES ATATÜRK mahalle numara on iki",
        ),
        ("adresi Bağdat cad. no: 5", "adresi Bağdat cadde numara beş"),
        ("Adresi Bağdat Cd. No:5", "Adresi Bağdat cadde numara beş"),
        ("ADRESİ BAĞDAT CD. NO:5", "ADRESİ BAĞDAT cadde numara beş"),
        ("adresiniz Lale sk. kat:0", "adresiniz Lale sokak kat sıfır"),
        (
            "Adresiniz Lale Sok. Kat: 0",
            "Adresiniz Lale sokak kat sıfır",
        ),
        (
            "ADRESİNİZ LALE SOK. KAT :0",
            "ADRESİNİZ LALE sokak kat sıfır",
        ),
        ("adres: No:5 d:2", "adres: numara beş daire iki"),
        ("adres: No:5 Daire :2", "adres: numara beş daire iki"),
        ("adres: No:5 DAİRE : 2", "adres: numara beş daire iki"),
    ]);
}

#[test]
fn inference_is_frame_wide_and_order_independent() {
    goldens(&[
        (
            "adres: No:12/4",
            "adres: bina numarası on iki, daire numarası dört",
        ),
        (
            "adres: No: 12 / 4",
            "adres: bina numarası on iki, daire numarası dört",
        ),
        (
            "adres: No:12/4 Daire:7",
            "adres: numara on iki bölü dört daire yedi",
        ),
        (
            "adres: D:7 No:12/4",
            "adres: daire yedi numara on iki bölü dört",
        ),
        (
            "adres: No:12/4 Kat:3 Daire:7",
            "adres: numara on iki bölü dört kat üç daire yedi",
        ),
        (
            "adres: No:12/4; adres: Daire:7 No:5/2",
            "adres: bina numarası on iki, daire numarası dört; adres: daire yedi numara beş bölü iki",
        ),
    ]);
}

#[test]
fn validated_field_combinations_do_not_need_a_cue() {
    goldens(&[
        ("Bağdat Cd. No:5", "Bağdat cadde numara beş"),
        (
            "Atatürk Mah. Cumhuriyet Cad.",
            "Atatürk mahalle Cumhuriyet cadde",
        ),
        ("No:5 Kat:3", "numara beş kat üç"),
        ("245. Sk. No:5", "iki yüz kırk beşinci sokak numara beş"),
    ]);
}

#[test]
fn floor_signs_are_structural_not_house_number_guesses() {
    goldens(&[
        ("adres Kat:-2 No:5", "adres kat eksi iki numara beş"),
        ("adres Kat:+2 Daire:7", "adres kat artı iki daire yedi"),
        ("adres Kat:0", "adres kat sıfır"),
    ]);
}

#[test]
fn postcode_labels_and_reviewed_tail_positions_preserve_zeroes() {
    goldens(&[
        (
            "adres No:5 03450 Kadıköy.",
            "adres numara beş posta kodu sıfır üç dört beş sıfır Kadıköy.",
        ),
        (
            "adres No:5 00000 Kadıköy/İstanbul.",
            "adres numara beş posta kodu sıfır sıfır sıfır sıfır sıfır Kadıköy, İstanbul.",
        ),
        (
            "adres Posta kodu:03450",
            "adres posta kodu sıfır üç dört beş sıfır",
        ),
        (
            "adres posta kodu : 34728",
            "adres posta kodu üç dört yedi iki sekiz",
        ),
        (
            "adres POSTA KODU: 34728",
            "adres posta kodu üç dört yedi iki sekiz",
        ),
        (
            "adres Posta Kodu 34728",
            "adres posta kodu üç dört yedi iki sekiz",
        ),
        ("Posta kodu: 03450", "posta kodu sıfır üç dört beş sıfır"),
        (
            "adres No:5 34728 sonra",
            "adres numara beş otuz dört bin yedi yüz yirmi sekiz sonra",
        ),
        ("34728", "otuz dört bin yedi yüz yirmi sekiz"),
    ]);
}

#[test]
fn cues_alone_and_aliases_outside_context_do_not_expand() {
    let normalizer = Normalizer::new().unwrap();
    for input in [
        "adresiniz",
        "Adres iyi bilinir.",
        "adresi ve Cad. sözcükleri",
        "Mah. Mh. Cad. Cd. Sk. Sok.",
        "Bağdat Cd.",
        "No:12",
        "Kat:3",
        "D:2",
        "D:\\Belgeler\\No:12",
        "D:/Belgeler/No:12",
        "adres: No:5. Mah. ve Cd.",
        "adres: No:5; Mah. ve Cd.",
        "adres: No:5\nMah. ve Cd.",
    ] {
        let result = normalizer
            .normalize(input, &options(AmbiguityPolicy::Preserve))
            .unwrap();
        assert!(
            !result
                .segments()
                .iter()
                .any(|segment| segment.rule_id() == "address.field"
                    && matches!(
                        &input[segment.range().start()..segment.range().end()],
                        "Mah." | "Mh." | "Cad." | "Cd." | "Sk." | "Sok."
                    )),
            "{input}"
        );
        partition(input, &result);
    }
    for input in ["adresiniz", "Adres iyi bilinir."] {
        goldens(&[(input, input)]);
    }
}

fn failed_field(input: &str, failed: &str, preserve: &str) {
    let normalizer = Normalizer::new().unwrap();
    let start = input.find(failed).unwrap();
    let range = SourceRange::new(start, start + failed.len());
    let result = normalizer
        .normalize(input, &options(AmbiguityPolicy::Preserve))
        .unwrap();
    assert_eq!(result.normalized_text(), preserve, "{input}");
    assert!(!result.complete(), "{input}");
    assert_eq!(result.issues().len(), 1, "{input}");
    assert_eq!(result.issues()[0].range(), range, "{input}");
    assert_eq!(
        result.issues()[0].category(),
        IssueCategory::InvalidExpression,
        "{input}"
    );
    assert!(!result.fallback_used(), "{input}");
    assert_eq!(
        result
            .segments()
            .iter()
            .filter(|segment| segment.kind() == SegmentKind::Unresolved)
            .map(|segment| segment.range())
            .collect::<Vec<_>>(),
        vec![range],
        "{input}"
    );
    partition(input, &result);
    match normalizer.normalize(input, &options(AmbiguityPolicy::Reject)) {
        Err(NormalizeError::Unresolved(issues)) => assert_eq!(issues, result.issues(), "{input}"),
        other => panic!("unexpected strict field result for {input}: {other:?}"),
    }
    let fallback = normalizer
        .normalize(input, &options(AmbiguityPolicy::Fallback))
        .unwrap();
    assert!(fallback.complete(), "{input}");
    assert!(fallback.issues().is_empty(), "{input}");
    assert_eq!(fallback.fallbacks().len(), 1, "{input}");
    assert_eq!(fallback.fallbacks()[0].range(), range, "{input}");
    assert_eq!(
        fallback.fallbacks()[0].attempted_class(),
        FallbackClass::Number,
        "{input}"
    );
    assert_eq!(
        fallback.fallbacks()[0].reason(),
        FallbackReason::InvalidForm,
        "{input}"
    );
    assert_eq!(
        fallback.fallbacks()[0].original_category(),
        Some(IssueCategory::InvalidExpression),
        "{input}"
    );
    assert_eq!(
        fallback
            .segments()
            .iter()
            .filter(|segment| segment.kind() != SegmentKind::Fallback)
            .map(|segment| (segment.range(), segment.kind(), segment.text()))
            .collect::<Vec<_>>(),
        result
            .segments()
            .iter()
            .filter(|segment| segment.kind() != SegmentKind::Unresolved)
            .map(|segment| (segment.range(), segment.kind(), segment.text()))
            .collect::<Vec<_>>(),
        "{input}"
    );
    partition(input, &fallback);
}

#[test]
fn malformed_fields_are_atomic_and_valid_neighbors_stay_primary() {
    for (input, failed, expected) in [
        (
            "adres No:abc Kat:3 D:7",
            "No:abc",
            "adres No:abc kat üç daire yedi",
        ),
        ("adres No:12A Kat:3", "No:12A", "adres No:12A kat üç"),
        ("adres No:12/ Kat:3", "No:12/", "adres No:12/ kat üç"),
        ("adres No:/4 Kat:3", "No:/4", "adres No:/4 kat üç"),
        (
            "adres No:12/4/2 Kat:3",
            "No:12/4/2",
            "adres No:12/4/2 kat üç",
        ),
        (
            "adres No:12 / 4 / 2 Kat:3",
            "No:12 / 4 / 2",
            "adres No:12 / 4 / 2 kat üç",
        ),
        ("adres No:1,5 Kat:3", "No:1,5", "adres No:1,5 kat üç"),
        ("adres No:-2 Kat:3", "No:-2", "adres No:-2 kat üç"),
        ("adres No:0 Kat:3", "No:0", "adres No:0 kat üç"),
        ("adres No:5 Daire:0", "Daire:0", "adres numara beş Daire:0"),
        ("adres No:5 Kat:1/2", "Kat:1/2", "adres numara beş Kat:1/2"),
        (
            "adres No:5 Daire:7A",
            "Daire:7A",
            "adres numara beş Daire:7A",
        ),
        ("adres No:5 Kat:", "Kat:", "adres numara beş Kat:"),
        ("adres No: Kat:3", "No:", "adres No: kat üç"),
        ("adres No:", "No:", "adres No:"),
        (
            "adres No:5 Posta kodu:1234",
            "Posta kodu:1234",
            "adres numara beş Posta kodu:1234",
        ),
        (
            "adres No:5 Posta kodu:123456",
            "Posta kodu:123456",
            "adres numara beş Posta kodu:123456",
        ),
        (
            "adres No:5 Posta kodu:12A45",
            "Posta kodu:12A45",
            "adres numara beş Posta kodu:12A45",
        ),
    ] {
        failed_field(input, failed, expected);
    }
}

#[test]
fn malformed_explicit_apartments_suppress_inference_even_when_later() {
    for (input, failed, expected) in [
        (
            "adres No:12/4 Daire:abc",
            "Daire:abc",
            "adres numara on iki bölü dört Daire:abc",
        ),
        (
            "adres No:12/4 Kat:3 D:",
            "D:",
            "adres numara on iki bölü dört kat üç D:",
        ),
        (
            "adres Daire:abc No:12/4",
            "Daire:abc",
            "adres Daire:abc numara on iki bölü dört",
        ),
        (
            "adres No:12/4 Daire: Kat:3",
            "Daire:",
            "adres numara on iki bölü dört Daire: kat üç",
        ),
    ] {
        failed_field(input, failed, expected);
    }
}

#[test]
fn numeric_street_period_is_ordinal_only_inside_address_context() {
    goldens(&[
        ("adres 245. Sk.", "adres iki yüz kırk beşinci sokak"),
        ("adres 1. Sok. No:5", "adres birinci sokak numara beş"),
        ("adres 1000. Sk.", "adres bininci sokak"),
    ]);
    let input = "adres No:5. Sonra 245.";
    let result = Normalizer::new()
        .unwrap()
        .normalize(input, &NormalizeOptions::default())
        .unwrap();
    assert_eq!(result.normalized_text(), "adres numara beş. Sonra 245.");
    assert!(!result.complete());
    assert_eq!(result.issues()[0].category(), IssueCategory::Ambiguous);
    partition(input, &result);
}

#[test]
fn nfd_prefixes_and_locality_names_preserve_original_source_partitions() {
    goldens(&[
        (
            "o\u{308}n adres No:12/4 Kadıko\u{308}y/I\u{307}stanbul.",
            "o\u{308}n adres bina numarası on iki, daire numarası dört Kadıko\u{308}y, I\u{307}stanbul.",
        ),
        (
            "e\u{301} adres Bag\u{306}dat Cd. No:5 03450 Kadıko\u{308}y.",
            "e\u{301} adres Bag\u{306}dat cadde numara beş posta kodu sıfır üç dört beş sıfır Kadıko\u{308}y.",
        ),
    ]);
}

#[test]
fn electronic_interiors_keep_priority_and_independent_name_aliases_survive() {
    let normalizer = Normalizer::new().unwrap();
    for electronic in [
        "https://ornek.com/No:12/4/Kat:3?D:7",
        "www.ornek.com/No:12/4",
        "No.12@ornek.com",
        "https://ornek.com/a@b?x=2",
    ] {
        let standalone = normalizer
            .normalize(electronic, &NormalizeOptions::default())
            .unwrap();
        for policy in POLICIES {
            let input = format!("adres No:5 {electronic} Kat:3");
            let result = normalizer.normalize(&input, &options(policy)).unwrap();
            assert_eq!(
                result.normalized_text(),
                format!("adres numara beş {} kat üç", standalone.normalized_text()),
                "{input}"
            );
            assert!(result.complete(), "{input}");
            assert!(!result.fallback_used(), "{input}");
            assert_eq!(
                result
                    .segments()
                    .iter()
                    .filter(|segment| segment.kind() == SegmentKind::Electronic)
                    .count(),
                1,
                "{input}"
            );
            partition(&input, &result);
        }
    }
    goldens(&[("adres No:5 Google Kat:3", "adres numara beş gugıl kat üç")]);
}

#[test]
fn hints_cannot_escape_an_atomic_address_field() {
    let normalizer = Normalizer::new().unwrap();
    let input = "adres No:12/4 Daire:7";
    for (range, kind) in [
        (SourceRange::new(9, 11), HintKind::Cardinal),
        (SourceRange::new(12, 13), HintKind::Cardinal),
        (SourceRange::new(19, 20), HintKind::Cardinal),
        (SourceRange::new(6, 13), HintKind::Digits),
    ] {
        for policy in POLICIES {
            assert_eq!(
                normalizer.normalize(
                    input,
                    &NormalizeOptions {
                        ambiguity_policy: policy,
                        hints: vec![Hint::new(range, kind)],
                    }
                ),
                Err(NormalizeError::InvalidHint),
                "{range:?}"
            );
        }
    }
}

#[test]
fn repeated_fields_respect_controls_and_existing_work_limits() {
    let normalizer = Normalizer::new().unwrap();
    let input = format!("adres {}", "No:1 Daire:2 ".repeat(1000));
    assert!(input.len() < MAX_INPUT_BYTES);
    let result = normalizer
        .normalize(&input, &NormalizeOptions::default())
        .unwrap();
    assert!(result.complete());
    assert_eq!(
        result.normalized_text(),
        format!("adres {}", "numara bir daire iki ".repeat(1000))
    );
    partition(&input, &result);
    let cancelled = WorkControl::default();
    cancelled.cancel();
    let deadline = WorkControl::new(Some(Instant::now() - Duration::from_secs(1)));
    for control in [cancelled, deadline] {
        for policy in POLICIES {
            assert_eq!(
                normalizer.normalize_controlled(&input, &options(policy), &control),
                Err(NormalizeError::Cancelled)
            );
        }
    }
    let over_budget = format!("adres {}", "No:1 ".repeat(2200));
    assert!(over_budget.len() < MAX_INPUT_BYTES);
    for policy in POLICIES {
        assert_eq!(
            normalizer.normalize(&over_budget, &options(policy)),
            Err(NormalizeError::LimitExceeded(LimitKind::Candidates))
        );
    }
}
