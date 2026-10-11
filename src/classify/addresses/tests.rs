use super::*;
use crate::{
    domain::composite::ComponentValue,
    interpretation::{Reading, Value},
    resources::Resources,
    source_map::SourceMap,
};

#[test]
fn owners_are_sorted_token_started_and_components_partition_absolute_nfc_ranges() {
    let control = WorkControl::default();
    let resources = Resources::load().unwrap();
    for input in [
        "o\u{308}n adres No:12/4 Kadıko\u{308}y/I\u{307}stanbul.",
        "adres No : 12 / 4 Kat:-2 Daire:7",
        "Teslimat adresi: Bağdat Cd. 245. Sk. No: 5 D: 2, 03450 Kadıköy.",
        "adres No:abc Kat:3 Daire:",
        "adres No:5,Kat:3",
        "adres No:5;Kat:3",
        "adres (No:5) Kat:3",
        "adres No: 12. Kat:3",
        "adres No:5 \"No:12 Mah.\"@ornek.com Kat:3",
        "Posta kodu: 03450",
    ] {
        let source = SourceMap::new(input, &control).unwrap();
        let text = source.text();
        let tokens = crate::classify::scan::tokens(&source, &resources, &control)
            .unwrap()
            .tokens;
        let candidates = collect(text, &control).unwrap();
        let mut previous = 0;
        for candidate in candidates {
            assert!(candidate.range.start >= previous, "{input}");
            assert!(candidate.range.start < candidate.range.end, "{input}");
            assert!(
                tokens
                    .iter()
                    .any(|token| token.range.start == candidate.range.start),
                "{input}: {:?}",
                candidate.range
            );
            source.original(candidate.range).unwrap();
            if let Reading::Resolved(Value::Composite(value)) = candidate.reading {
                let mut cursor = candidate.range.start;
                for component in value.components {
                    assert_eq!(component.range.start, cursor, "{input}");
                    assert!(component.range.end > cursor, "{input}");
                    source.original(component.range).unwrap();
                    if let ComponentValue::Text {
                        kind: SegmentKind::Verbatim,
                        text: original,
                        ..
                    } = component.value
                    {
                        assert_eq!(original, text[cursor..component.range.end], "{input}");
                    }
                    cursor = component.range.end;
                }
                assert_eq!(cursor, candidate.range.end, "{input}");
            }
            previous = candidate.range.end;
        }
    }
}

#[test]
fn full_quoted_electronic_owners_have_no_address_interior_claims() {
    let text = "adres No:5 \"No:12 Mah.\"@ornek.com Kat:3";
    let start = text.find('"').unwrap();
    let end = text.find(" Kat:").unwrap();
    let candidates = collect(text, &WorkControl::default()).unwrap();
    assert_eq!(candidates.len(), 2);
    for candidate in candidates {
        assert!(candidate.range.end <= start || candidate.range.start >= end);
    }
}

#[test]
fn a_cue_plus_ordinary_postcode_prose_is_not_a_field() {
    for text in [
        "Adres ve posta kodu önemlidir.",
        "adresiniz ve No kelimesi",
        "adresi ile Kat ve Daire hakkında konuşalım",
        "Mah. Mh. Cad. Cd. Sk. Sok.",
    ] {
        assert!(
            collect(text, &WorkControl::default()).unwrap().is_empty(),
            "{text}"
        );
    }
}
