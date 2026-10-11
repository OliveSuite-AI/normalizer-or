use super::*;

#[test]
fn product_leaf_parser_owns_every_valid_quantity_surface_and_tail() {
    for (source, expected) in [
        ("25 TL'm", "yirmi beş Türk liram"),
        ("TL'm 25", "yirmi beş Türk liram"),
        ("25TL'm", "yirmi beş Türk liram"),
        ("₺25'm", "yirmi beş Türk liram"),
        ("₺ 25'm", "yirmi beş Türk liram"),
        ("25₺'m", "yirmi beş Türk liram"),
        (
            "-1.234,50 TL'm",
            "eksi bin iki yüz otuz dört Türk lirası elli kuruşum",
        ),
        (
            "1 234,50 TL'm",
            "bin iki yüz otuz dört Türk lirası elli kuruşum",
        ),
        (
            "₺1\u{202f}234,05'dır",
            "bin iki yüz otuz dört Türk lirası beş kuruştur",
        ),
        (
            "TL 1 234,50",
            "bin iki yüz otuz dört Türk lirası elli kuruş",
        ),
        ("1 234TL", "bin iki yüz otuz dört Türk lirası"),
        ("5km/sa'tir", "beş kilometre saattir"),
        ("5 m/s'mdir", "beş metre saniyemdir"),
        ("kg'larımızda 5", "beş kilogramlarımızda"),
    ] {
        let quantity =
            Quantity::parse_operand(source).unwrap_or_else(|error| panic!("{source}: {error:?}"));
        assert_eq!(quantity.render().into_text(), expected, "{source}");
    }
}

#[test]
fn isolated_parser_never_accepts_extra_marker_operator_or_malformed_grouping() {
    for source in [
        "",
        " 25 TL",
        "25 TL ",
        "25 TL USD",
        "₺25 TL",
        "25 TL x 2",
        "25 TL'm x 2",
        "25*2 TL",
        "1 23 TL",
        "001 234 TL",
        "1.23 TL",
        "1,005 TL",
        "5 kgfoo",
        "TL 25'm",
        "TL'm 25'n",
        "25TL''m",
        "5 km/sa'tirdir",
        "25 TL 'm",
    ] {
        assert!(Quantity::parse_operand(source).is_err(), "{source}");
    }
    assert_eq!(
        Quantity::parse_operand("25 TL'leri").unwrap_err(),
        IssueCategory::Ambiguous
    );
}

#[test]
fn case_precedence_and_plural_agreement_are_explicit() {
    for (source, expected) in [
        ("25 TL'yi", "yirmi beş Türk lirasını"),
        ("25 TL'nin", "yirmi beş Türk lirasının"),
        ("25 TL'ler", "yirmi beş Türk liraları"),
        ("25 TL'lerimden", "yirmi beş Türk liralarımdan"),
        ("25,05 TL'leri", "yirmi beş Türk lirası beş kuruşları"),
        ("5 m³'ü", "beş metrekübü"),
    ] {
        assert_eq!(
            Quantity::parse_operand(source)
                .unwrap()
                .render()
                .into_text(),
            expected,
            "{source}"
        );
    }
}
