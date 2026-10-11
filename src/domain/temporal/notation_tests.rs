use super::*;
use crate::{morphology::Inflection, verbalize};

#[test]
fn all_date_formats_validate_source_tail_and_render_independent_target_tail() {
    for (source, suffix, expected) in [
        ("2026-12-31", "de", "otuz bir Aralık iki bin yirmi altıda"),
        (
            "2026/12/31",
            "dekidir",
            "otuz bir Aralık iki bin yirmi altıdakidir",
        ),
        ("31/12/2026", "dır", "otuz bir Aralık iki bin yirmi altıdır"),
        (
            "31.12.2026",
            "nındır",
            "otuz bir Aralık iki bin yirmi altınındır",
        ),
        ("2024-12-02", "ye", "iki Aralık iki bin yirmi dörde"),
        ("2024-12-02", "de", "iki Aralık iki bin yirmi dörtte"),
        ("0001-01-03", "tür", "üç Ocak birdir"),
    ] {
        let date = Date::parse(source).unwrap();
        let tail = DateTail::parse(date, Some(suffix)).unwrap();
        let mut spoken = verbalize::date_spoken(date);
        tail.apply(&mut spoken);
        assert_eq!(spoken.into_text(), expected, "{source}'{suffix}");
    }
    for (source, suffix) in [
        ("2026-12-31", "da"),
        ("31.12.2026", "de"),
        ("2026-12-31", "m"),
        ("31.12.2026", "lar"),
        ("31.12.2026", "dakiler"),
        ("31.12.2026", "dakiye"),
        ("31.12.2026", "dırdır"),
        ("31.12.2026", "ya'dır"),
    ] {
        assert!(
            DateTail::parse(Date::parse(source).unwrap(), Some(suffix)).is_none(),
            "{source}'{suffix}"
        );
    }
}

#[test]
fn fixed_width_and_gregorian_checks_do_not_guess_an_order() {
    for source in [
        "2026/2/03",
        "2026/02/3",
        "26/02/03",
        "2026/31/12",
        "12/31/2026",
        "1900/02/29",
        "0000/01/01",
        "2026/12/31/01",
    ] {
        assert!(Date::parse(source).is_none(), "{source}");
    }
    for source in ["0001/01/01", "0400/02/29", "2000/02/29", "2024/02/29"] {
        assert!(Date::parse(source).unwrap().year_first_slash());
    }
    assert!(Date::parse("31/12/2026").unwrap().slash());
}

#[test]
fn clocks_use_hour_at_zero_minutes_and_ranges_preserve_endpoints() {
    assert_eq!(
        Clock::parse("14.00")
            .unwrap()
            .tail()
            .strip_suffix_prefix(Inflection::Locative, "te"),
        Some("")
    );
    for (source, expected) in [
        ("14.30-16.45", ("on dört otuz", "on altı kırk beş")),
        ("23:00 - 01:00", ("yirmi üç", "bir")),
        ("14.30–14.30", ("on dört otuz", "on dört otuz")),
    ] {
        let range = ClockRange::parse(source, true).unwrap();
        assert_eq!(
            verbalize::time_spoken(range.start()).into_text(),
            expected.0
        );
        assert_eq!(verbalize::time_spoken(range.end()).into_text(), expected.1);
        assert_eq!(range.connector(), "ile");
    }
    for source in [
        "24:00-01:00",
        "14:30-16:60",
        "14:30:00-16:30",
        "14.30'da-16.30",
        "14.30-16.30'da",
        "14.30-16.30-17.00",
        "14.30-16",
        "2026-12-31",
    ] {
        assert!(ClockRange::parse(source, false).is_none(), "{source}");
    }
}
