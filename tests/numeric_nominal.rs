//! Independent finite-tail goldens and atomic failures across policies.
use normalizer_tr::{AmbiguityPolicy, NormalizeError, NormalizeOptions, Normalizer, SegmentKind};

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
        assert!(
            result
                .segments()
                .iter()
                .any(|segment| segment.kind() != SegmentKind::Verbatim)
        );
    }
}

#[test]
fn six_possessors_plural_cases_ki_and_copula_are_primary() {
    for (input, expected) in [
        ("2'm", "ikim"),
        ("2'n", "ikin"),
        ("2'si", "ikisi"),
        ("2'miz", "ikimiz"),
        ("2'niz", "ikiniz"),
        ("2'leri", "ikileri"),
        ("3'ler", "üçler"),
        ("3'lerimizden", "üçlerimizden"),
        ("3'ünüzü", "üçünüzü"),
        ("4'üm", "dördüm"),
        ("4'ümdekidir", "dördümdekidir"),
        ("6'larındakidir", "altılarındakidir"),
        ("5'tir", "beştir"),
        ("9'undur", "dokuzundur"),
        ("2'sindekidir", "ikisindekidir"),
        ("1.'m", "birincim"),
        ("4'üncümüzdeki", "dördüncümüzdeki"),
        ("6'ncı'larımızdan", "altıncılarımızdan"),
        ("4,04'üm", "dört virgül sıfır dördüm"),
        ("100'ümüz", "yüzümüz"),
        ("1000'imiz", "binimiz"),
        ("1000000'umuz", "bir milyonumuz"),
        ("1000000000'ımız", "bir milyarımız"),
        ("1000000000000'umuz", "bir trilyonumuz"),
        ("1000000000000000'umuz", "bir katrilyonumuz"),
        ("%20'si", "yüzde yirmisi"),
        ("%3'lerimizdekidir", "yüzde üçlerimizdekidir"),
        ("%3,25'tir", "yüzde üç virgül iki beştir"),
        ("%5'lik", "yüzde beşlik"),
    ] {
        primary(input, expected);
    }
}

#[test]
fn currency_heads_minor_units_and_rate_denominators_are_primary() {
    for (input, expected) in [
        ("25 TL'm", "yirmi beş Türk liram"),
        ("25 TL'lerim", "yirmi beş Türk liralarım"),
        ("25,50 TL'm", "yirmi beş Türk lirası elli kuruşum"),
        ("600 TL'dir", "altı yüz Türk lirasıdır"),
        ("600,05 TL'dir", "altı yüz Türk lirası beş kuruştur"),
        ("25 TRY'm", "yirmi beş Türk liram"),
        ("25₺'m", "yirmi beş Türk liram"),
        ("25 TL'ye", "yirmi beş Türk lirasına"),
        ("25,05 TL'den", "yirmi beş Türk lirası beş kuruştan"),
        ("25 USD'm", "yirmi beş dolarım"),
        ("5 m³'ümdeki", "beş metrekübümdeki"),
        ("5 kg'larımızdakidir", "beş kilogramlarımızdakidir"),
        ("5 km/sa", "beş kilometre saat"),
        ("5km/sa'tir", "beş kilometre saattir"),
        ("5km/h'lerimizdeki", "beş kilometre saatlerimizdeki"),
        ("5 m/s", "beş metre saniye"),
        ("5m/s'mdir", "beş metre saniyemdir"),
    ] {
        primary(input, expected);
    }
}

#[test]
fn bad_stage_order_and_conflicting_new_currency_tails_stay_atomic() {
    let normalizer = Normalizer::new().unwrap();
    for input in [
        "3'lerler",
        "3'ümüm",
        "3'denler",
        "3'tekiler",
        "3'tekine",
        "3'tirdi",
        "3'tekiydi",
        "25 TL'leri",
        "5kg'larımızlarımız",
        "2''m",
    ] {
        let preserved = normalizer
            .normalize(input, &NormalizeOptions::default())
            .unwrap();
        assert!(!preserved.complete(), "{input}");
        assert_eq!(preserved.normalized_text(), input);
        assert_eq!(preserved.issues()[0].range().start(), 0);
        assert_eq!(preserved.issues()[0].range().end(), input.len());
        assert_eq!(
            normalizer.normalize(
                input,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::Unresolved(preserved.issues().to_vec())),
            "{input}"
        );
        let fallback = normalizer
            .normalize(
                input,
                &NormalizeOptions {
                    ambiguity_policy: AmbiguityPolicy::Fallback,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(fallback.complete());
        assert!(!fallback.fallbacks().is_empty());
    }
}
