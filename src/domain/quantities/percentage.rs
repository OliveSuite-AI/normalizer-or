//! Validated percentages with derived or bounded nominal tails.
use crate::{
    IssueCategory,
    morphology::{Inflection, Spoken, TailInflection},
    notation::suffix_parts,
    numerals::Number,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum PercentTail {
    Derivation,
    Nominal(TailInflection),
}

impl PercentTail {
    pub(crate) fn apply(self, spoken: &mut Spoken) {
        match self {
            Self::Derivation => spoken.inflect(Inflection::Derivation),
            Self::Nominal(tail) => tail.apply(spoken),
        }
    }
}

pub(crate) fn percent(text: &str) -> Option<Result<(Number, Option<PercentTail>), IssueCategory>> {
    let (base, suffixes) = suffix_parts(text)?;
    let body = base.strip_prefix('%').or_else(|| base.strip_suffix('%'))?;
    Some(percent_value(body, &suffixes))
}

pub(crate) fn percent_body(text: &str) -> Result<(Number, Option<PercentTail>), IssueCategory> {
    let (body, suffixes) = suffix_parts(text).ok_or(IssueCategory::Unsupported)?;
    percent_value(body, &suffixes)
}

fn percent_value(
    body: &str,
    suffixes: &[&str],
) -> Result<(Number, Option<PercentTail>), IssueCategory> {
    let number = match Number::parse(body) {
        Some(n) => n,
        None => return Err(IssueCategory::InvalidExpression),
    };
    if suffixes.len() > 1 {
        return Err(IssueCategory::Unsupported);
    }
    let source = number.tail();
    let inflection = match suffixes.first() {
        Some(s) if source.strip_suffix_prefix(Inflection::Derivation, s) == Some("") => {
            Some(PercentTail::Derivation)
        }
        Some(s) => Some(PercentTail::Nominal(TailInflection::select(
            source, s, source,
        )?)),
        None => None,
    };
    // Keep the validated number and grammatical family, not parsed emitted text.
    Ok((number, inflection))
}
