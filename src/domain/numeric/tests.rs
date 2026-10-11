use super::*;

#[test]
fn products_establish_grouping_without_guessing_padding_or_notation() {
    for (source, expected) in [
        ("1.234", "bin iki yüz otuz dört"),
        (
            "-1.234,050",
            "eksi bin iki yüz otuz dört virgül sıfır beş sıfır",
        ),
        ("+2", "artı iki"),
        ("-0,00", "eksi sıfır virgül sıfır sıfır"),
        ("4'lerimizdeki", "dörtlerimizdeki"),
        ("1.234'lerimiz", "bin iki yüz otuz dörtlerimiz"),
    ] {
        let parsed = Numeric::product_operand(source).unwrap();
        assert_eq!(parsed.render().into_text(), expected, "{source}");
    }
    for source in [
        "00042", "001.234", "1.23", "1 234", "1e3", "12/3", "1.234.", "4'de", "2'lerler", "2 TL",
    ] {
        assert!(Numeric::product_operand(source).is_none(), "{source}");
    }
    assert!(Numeric::product_operand("2'lerimiz").unwrap().is_count());
    assert!(!Numeric::product_operand("-2").unwrap().is_count());
    assert!(!Numeric::product_operand("2,0").unwrap().is_count());
    assert_eq!(
        Numeric::automatic("1.234").unwrap_err().category,
        IssueCategory::Ambiguous
    );
}
