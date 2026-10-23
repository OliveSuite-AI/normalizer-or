//! Odia contracts, provenance invariants, and Unicode regressions.
use normalizer_or::*;
use proptest::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

#[path = "support/config.rs"]
mod test_config;
use test_config::config;

fn digit_script(text: &str, mode: u8) -> String {
    text.chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_ascii_digit() && (mode == 1 || (mode == 2 && i % 2 == 0)) {
                char::from_u32('୦' as u32 + c as u32 - '0' as u32).unwrap()
            } else {
                c
            }
        })
        .collect()
}

fn partition(input: &str, result: &NormalizeResult) {
    let mut boundaries: Vec<_> = input.grapheme_indices(true).map(|(i, _)| i).collect();
    boundaries.push(input.len());
    let mut cursor = 0;
    for s in result.segments() {
        assert_eq!(cursor, s.range().start());
        assert!(boundaries.contains(&cursor));
        cursor = s.range().end();
        assert!(boundaries.contains(&cursor));
        if matches!(s.kind(), SegmentKind::Verbatim | SegmentKind::Unresolved) {
            assert_eq!(s.text(), &input[s.range().start()..cursor]);
        }
    }
    assert_eq!(cursor, input.len());
    assert_eq!(
        result.normalized_text(),
        result
            .segments()
            .iter()
            .map(|s| s.text())
            .collect::<String>()
    );
}
proptest! {
    #![proptest_config(config())]
    #[test]
    fn integers_and_digit_scripts_agree(value in 0_u64..1_000_000_000_000_000_000) {
        let n=Normalizer::new().unwrap(); let ascii=value.to_string();
        let odia:String=ascii.chars().map(|c|char::from_u32('୦' as u32+c as u32-'0' as u32).unwrap()).collect();
        let a=n.normalize(&ascii,&Default::default()).unwrap(); let b=n.normalize(&odia,&Default::default()).unwrap();
        prop_assert_eq!(a.normalized_text(),b.normalized_text()); prop_assert!(a.complete()); partition(&ascii,&a); partition(&odia,&b);
    }
    #[test]
    fn arbitrary_unicode_partitions_and_cleanup_idempotence(chars in prop::collection::vec(any::<char>(),0..100)) {
        let input:String=chars.into_iter().filter(|c| !c.is_control() && !matches!(c,'\u{061c}'|'\u{200e}'|'\u{200f}'|'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')).collect();
        if input.trim().is_empty() { return Ok(()); }
        let n=Normalizer::new().unwrap();
        for policy in [AmbiguityPolicy::Preserve,AmbiguityPolicy::Fallback] {
            let opts=NormalizeOptions{ambiguity_policy:policy,..Default::default()};
            let result=n.normalize(&input,&opts).unwrap(); partition(&input,&result);
            prop_assert_eq!(&result,&n.normalize(&input,&opts).unwrap());
        }
        let result=n.clean(&input,&Default::default()).unwrap(); partition(&input,&result);
        let repeated=n.clean(result.normalized_text(),&Default::default()).unwrap();
        prop_assert_eq!(result.normalized_text(),repeated.normalized_text());
    }
}

proptest! {
    #![proptest_config(config())]
    #[test]
    fn boundary_compositions_preserve_policy_and_source_invariants(parts in prop::collection::vec(
        prop::sample::select(vec!["₹1 234", "% 12 %", "5 kg", "(5 kg)", "₹25,₹30", "+91 98765 43210", "phone: 9876543210", "Dr.Smith", "e\u{301}", "( 5 kg", "1,23", "🙂", "https://x/5?q=1,2", "5 kg\u{301}", "‘୫ kg’"]),1..15)) {
        let input=parts.join("; ");
        let n=Normalizer::new().unwrap();
        let preserved=n.normalize(&input,&Default::default()).unwrap(); partition(&input,&preserved);
        let rejected=n.normalize(&input,&NormalizeOptions{ambiguity_policy:AmbiguityPolicy::Reject,..Default::default()});
        if preserved.complete() { prop_assert_eq!(rejected.unwrap(),preserved.clone()); }
        else { prop_assert_eq!(rejected,Err(NormalizeError::Unresolved(preserved.issues().to_vec()))); }
        let fallback=n.normalize(&input,&NormalizeOptions{ambiguity_policy:AmbiguityPolicy::Fallback,..Default::default()}).unwrap();
        partition(&input,&fallback); prop_assert!(fallback.complete());
        let primary=|r:&NormalizeResult| r.segments().iter().filter(|s|!matches!(s.kind(),SegmentKind::Verbatim|SegmentKind::Unresolved|SegmentKind::Fallback)).cloned().collect::<Vec<_>>();
        prop_assert_eq!(primary(&preserved),primary(&fallback));
    }
}

proptest! {
    #![proptest_config(config())]
    #[test]
    fn single_mutations_keep_complete_owners(value in 0u8..100, mode in 0u8..3, mutation in 0u8..7, wrapped in any::<bool>()) {
        let source=match mutation {
            0=>format!("₹₹{value}"),1=>format!("%{value}%"),2=>format!("{value} KG"),
            3=>format!("{value}.0000000000"),4=>format!("{value} kg\u{301}"),5=>format!("{value} kg kg"),_=>format!("{value},,234"),
        };
        let source=digit_script(&source,mode);
        let (left,right)=if wrapped {("“", "”")} else {("", "")};
        let prefix=format!("e\u{301} {left}");
        let input=format!("{prefix}{source}{right}; 6 kg");
        let n=Normalizer::new().unwrap();
        let result=n.normalize(&input,&Default::default()).unwrap(); partition(&input,&result);
        prop_assert!(!result.complete()); prop_assert_eq!(result.issues().len(),1);
        prop_assert_eq!(result.issues()[0].range(),SourceRange::new(prefix.len(),prefix.len()+source.len()));
        prop_assert_eq!(result.issues()[0].category(),if matches!(mutation,2|4|5) {IssueCategory::Unsupported} else {IssueCategory::InvalidExpression});
        prop_assert_eq!(result.normalized_text(),format!("{prefix}{source}{right}; ଛଅ କିଲୋଗ୍ରାମ"));
        let fallback=n.normalize(&input,&NormalizeOptions{ambiguity_policy:AmbiguityPolicy::Fallback,..Default::default()}).unwrap();
        partition(&input,&fallback); prop_assert!(fallback.complete()); prop_assert_eq!(fallback.fallbacks().len(),1);
        prop_assert_eq!(fallback.fallbacks()[0].range(),result.issues()[0].range());
    }
    #[test]
    fn canonical_equivalence_and_cleanup_options(value in 0u8..100, mode in 0u8..3) {
        use unicode_normalization::UnicodeNormalization;
        let n=Normalizer::new().unwrap();
        let tail=digit_script(&format!("{value} kg"),mode);
        let inputs=[format!("e\u{301} {tail}"),format!("é {tail}")];
        let a=n.normalize(&inputs[0],&Default::default()).unwrap();
        let b=n.normalize(&inputs[1],&Default::default()).unwrap();
        prop_assert_eq!(a.normalized_text().nfc().collect::<String>(),b.normalized_text().nfc().collect::<String>());
        prop_assert_eq!(a.segments().last().unwrap().range().start(),4);
        prop_assert_eq!(b.segments().last().unwrap().range().start(),3);
        for digits in [DigitStyle::Preserve,DigitStyle::Ascii,DigitStyle::Odia] {
            for whitespace in [WhitespacePolicy::Preserve,WhitespacePolicy::Collapse] {
                let options=CleanOptions{digits,whitespace};
                let source=format!("  {}\tକ୍\u{200d}ଷ କ୍\u{200c}ଷ\r\n",inputs[0]);
                let result=n.clean(&source,&options).unwrap(); partition(&source,&result);
                let repeated=n.clean(result.normalized_text(),&options).unwrap();
                prop_assert_eq!(result.normalized_text(),repeated.normalized_text());
                prop_assert!(result.normalized_text().contains("କ୍\u{200d}ଷ କ୍\u{200c}ଷ"), "joiners must survive cleanup");
            }
        }
    }
}
