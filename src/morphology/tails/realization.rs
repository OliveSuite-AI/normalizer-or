//! Borrowed suffix realization, spoken-stem mutation and shared terminal transitions.
use super::{EMPTY, Parts, TailInflection};
use crate::morphology::{
    Harmony, Inflection, Spoken, Word, WordEnd,
    nominal::{plural_stem, possessed_stem},
};

const STAGES: usize = 5;

/// A bounded suffix and original-head mutation; comparing it never renders a sentence.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Realization {
    parts: [Parts; STAGES],
    mutation: Option<char>,
    tail: Word,
}

impl Realization {
    #[cfg(test)]
    pub(super) fn matches(self, mut suffix: &str) -> bool {
        for parts in self.parts {
            for part in parts {
                let Some(rest) = suffix.strip_prefix(part) else {
                    return false;
                };
                suffix = rest;
            }
        }
        suffix.is_empty()
    }

    pub(crate) fn same_speech(self, other: Self) -> bool {
        self.mutation == other.mutation
            && self
                .parts
                .into_iter()
                .flatten()
                .flat_map(str::bytes)
                .eq(other.parts.into_iter().flatten().flat_map(str::bytes))
    }

    fn apply(self, spoken: &mut Spoken) {
        if let Some(replacement) = self.mutation {
            spoken.text.pop();
            spoken.text.push(replacement);
        }
        for part in self.parts.into_iter().flatten() {
            spoken.text.push_str(part);
        }
        spoken.tail = self.tail;
    }
}

impl TailInflection {
    pub(crate) fn realization(self, source: Word) -> Realization {
        let (mut stem, plural) = plural_stem(source, self.plural);
        let (possessed, possession, possession_vowel) =
            possessed_stem(stem, self.plural, self.possessor);
        stem = possessed;
        let case = self.case.map_or(EMPTY, |case| stem.suffix_parts(case));
        if let Some(case) = self.case {
            stem = stem.inflected(case);
        }
        let (ki_stem, ki) = ki_stem(stem, self.ki);
        let (stem, copula) = copula_stem(ki_stem, self.copula);
        let vowel_start = !self.plural
            && if self.possessor.is_some() {
                possession_vowel
            } else {
                self.case.is_some_and(|case| {
                    matches!(
                        case,
                        Inflection::Accusative | Inflection::Dative | Inflection::Genitive
                    )
                })
            };
        let mutation = if vowel_start {
            match source.end {
                WordEnd::Softens => Some('d'),
                WordEnd::SoftensP => Some('b'),
                _ => None,
            }
        } else {
            None
        };
        Realization {
            parts: [plural, possession, case, ki, copula],
            mutation,
            tail: stem,
        }
    }

    pub(crate) fn apply(self, spoken: &mut Spoken) {
        self.realization(spoken.tail).apply(spoken);
    }
}

pub(super) fn ki_stem(mut source: Word, ki: bool) -> (Word, Parts) {
    if !ki {
        return (source, EMPTY);
    }
    source.harmony = Harmony::FrontFlat;
    source.end = WordEnd::Vowel;
    (source, ["ki", "", "", ""])
}

pub(super) fn copula_stem(mut source: Word, copula: bool) -> (Word, Parts) {
    if !copula {
        return (source, EMPTY);
    }
    let stop = if matches!(
        source.end,
        WordEnd::Voiceless | WordEnd::Softens | WordEnd::SoftensP
    ) {
        "t"
    } else {
        "d"
    };
    let parts = [stop, source.harmony.high(), "r", ""];
    source.end = WordEnd::Voiced;
    (source, parts)
}
