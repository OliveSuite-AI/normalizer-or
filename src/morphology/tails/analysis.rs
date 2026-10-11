//! Finite suffix admission and explicit selection between complete analyses.
use super::{
    EMPTY, MAX_BYTES, Parts, TailInflection,
    realization::{copula_stem, ki_stem},
};
use crate::morphology::{
    Inflection, Word, case_inflection,
    nominal::{CASES, POSSESSORS, plural_stem, possessed_stem},
};

impl TailInflection {
    pub(crate) fn analyses(source: Word, suffix: &str, mut visit: impl FnMut(Self)) {
        if suffix.is_empty() || suffix.len() > MAX_BYTES {
            return;
        }
        for plural in [false, true] {
            let (stem, parts) = plural_stem(source, plural);
            let Some(remainder) = strip_parts(suffix, parts) else {
                continue;
            };
            for possessor in POSSESSORS {
                let (possessed, parts, _) = possessed_stem(stem, plural, possessor);
                let Some(remainder) = strip_parts(remainder, parts) else {
                    continue;
                };
                for case in CASES {
                    let parts = case.map_or(EMPTY, |case| possessed.suffix_parts(case));
                    let Some(remainder) = strip_parts(remainder, parts) else {
                        continue;
                    };
                    let stem = case.map_or(possessed, |case| possessed.inflected(case));
                    Self::finish(
                        stem,
                        remainder,
                        Self {
                            plural,
                            possessor,
                            case,
                            ki: false,
                            copula: false,
                        },
                        &mut visit,
                    );
                }
            }
        }
    }

    pub(crate) fn date(source: Word, suffix: &str) -> Option<Self> {
        if suffix.is_empty() || suffix.len() > MAX_BYTES {
            return None;
        }
        let mut selected = None;
        for case in CASES {
            let parts = case.map_or(EMPTY, |case| source.suffix_parts(case));
            let Some(remainder) = strip_parts(suffix, parts) else {
                continue;
            };
            let stem = case.map_or(source, |case| source.inflected(case));
            Self::finish(
                stem,
                remainder,
                Self {
                    case,
                    ..Default::default()
                },
                &mut |form| selected = Some(form),
            );
        }
        selected
    }

    fn finish(stem: Word, suffix: &str, form: Self, visit: &mut impl FnMut(Self)) {
        for ki in [false, true] {
            if ki && form.case != Some(Inflection::Locative) {
                continue;
            }
            let (stem, parts) = ki_stem(stem, ki);
            let Some(remainder) = strip_parts(suffix, parts) else {
                continue;
            };
            for copula in [false, true] {
                let (_, parts) = copula_stem(stem, copula);
                let form = Self { ki, copula, ..form };
                if form != Self::default() && strip_parts(remainder, parts) == Some("") {
                    visit(form);
                }
            }
        }
    }

    pub(crate) fn select(
        source: Word,
        suffix: &str,
        target: Word,
    ) -> Result<Self, crate::IssueCategory> {
        // Existing successful cases are a specified precedence, not enumeration order.
        if let Some(case) = case_inflection(source, suffix) {
            return Ok(Self::case(case));
        }
        let mut selected = None;
        let mut ambiguous = false;
        Self::analyses(source, suffix, |form| {
            if selected.is_some_and(|old: Self| {
                !old.realization(target)
                    .same_speech(form.realization(target))
            }) {
                ambiguous = true;
            } else {
                selected.get_or_insert(form);
            }
        });
        if ambiguous {
            Err(crate::IssueCategory::Ambiguous)
        } else {
            selected.ok_or(crate::IssueCategory::InvalidExpression)
        }
    }
}

fn strip_parts(mut text: &str, parts: Parts) -> Option<&str> {
    for part in parts {
        text = text.strip_prefix(part)?;
    }
    Some(text)
}
