use super::*;
use crate::morphology::{Harmony, Spoken, Word, WordEnd, nominal::POSSESSORS};

#[test]
fn finite_stages_and_four_harmonies_have_independent_goldens() {
    for (word, harmony, expected) in [
        (
            "ev",
            Harmony::FrontFlat,
            ["im", "in", "i", "imiz", "iniz", "leri"],
        ),
        (
            "ad",
            Harmony::BackFlat,
            ["ım", "ın", "ı", "ımız", "ınız", "ları"],
        ),
        (
            "yol",
            Harmony::BackRound,
            ["um", "un", "u", "umuz", "unuz", "ları"],
        ),
        (
            "gül",
            Harmony::FrontRound,
            ["üm", "ün", "ü", "ümüz", "ünüz", "leri"],
        ),
    ] {
        let source = Word::new(word, harmony, WordEnd::Voiced);
        for (possessor, suffix) in POSSESSORS.into_iter().flatten().zip(expected) {
            let form = TailInflection {
                possessor: Some(possessor),
                ..Default::default()
            };
            assert!(form.realization(source).matches(suffix), "{word}{suffix}");
        }
    }
    for (text, source, suffix, expected) in [
        (
            "dört",
            Word::new("dört", Harmony::FrontRound, WordEnd::Softens),
            "ümdekidir",
            "dördümdekidir",
        ),
        (
            "iki",
            Word::new("iki", Harmony::FrontFlat, WordEnd::Vowel),
            "sindekidir",
            "ikisindekidir",
        ),
        (
            "altı",
            Word::new("altı", Harmony::BackFlat, WordEnd::Vowel),
            "larındakidir",
            "altılarındakidir",
        ),
        (
            "metreküp",
            Word::new("metreküp", Harmony::FrontRound, WordEnd::SoftensP),
            "ümüzdeki",
            "metrekübümüzdeki",
        ),
    ] {
        let form = TailInflection::select(source, suffix, source).unwrap();
        let mut spoken = Spoken::lexical(text, source);
        form.apply(&mut spoken);
        assert_eq!(spoken.into_text(), expected);
    }
}

#[test]
fn ordinary_cases_precede_syncretic_possession_and_unsupported_orders_fail() {
    let source = Word::new("üç", Harmony::FrontRound, WordEnd::Voiceless);
    assert_eq!(
        TailInflection::select(source, "ü", source).unwrap(),
        TailInflection::case(Inflection::Accusative)
    );
    for suffix in [
        "",
        "ki",
        "lerler",
        "ümüm",
        "denler",
        "tekiler",
        "tekine",
        "tirdi",
        "tekiydi",
        "leriminimin",
    ] {
        assert!(
            TailInflection::select(source, suffix, source).is_err(),
            "{suffix}"
        );
    }
    assert!(TailInflection::select(source, &"x".repeat(MAX_BYTES + 1), source).is_err());
    let form = TailInflection::select(source, "leridir", source).unwrap();
    let mut output = Spoken::lexical("üç", source);
    form.apply(&mut output);
    assert_eq!(output.into_text(), "üçleridir");
}

#[test]
fn case_end_states_and_plural_coalescence_are_not_inherited_from_the_original_head() {
    for (source, suffix, expected) in [
        (
            Word::new("üç", Harmony::FrontRound, WordEnd::Voiceless),
            "tekidir",
            "üçtekidir",
        ),
        (
            Word::new("üç", Harmony::FrontRound, WordEnd::Voiceless),
            "lerdir",
            "üçlerdir",
        ),
        (
            Word::new("dört", Harmony::FrontRound, WordEnd::Softens),
            "ündür",
            "dördündür",
        ),
        (
            Word::new("iki", Harmony::FrontFlat, WordEnd::Vowel),
            "sinedir",
            "ikisinedir",
        ),
        (
            Word::new("altı", Harmony::BackFlat, WordEnd::Vowel),
            "sınıdır",
            "altısınıdır",
        ),
        (
            Word::new("dokuz", Harmony::BackRound, WordEnd::Voiced),
            "dandır",
            "dokuzdandır",
        ),
    ] {
        let form = TailInflection::select(source, suffix, source).unwrap();
        let mut spoken = Spoken::lexical(source.text, source);
        form.apply(&mut spoken);
        assert_eq!(spoken.into_text(), expected, "{suffix}");
    }
    let source = Word::new("iki", Harmony::FrontFlat, WordEnd::Vowel);
    let one_plural = TailInflection {
        possessor: Some(Possessor::ThirdPlural),
        ..Default::default()
    };
    let coalesced = TailInflection {
        plural: true,
        ..one_plural
    };
    assert!(
        one_plural
            .realization(source)
            .same_speech(coalesced.realization(source))
    );
}
