//! Fixed-order numeric/quantity tails, distinct from the sealed name profile.
mod analysis;
mod realization;
#[cfg(test)]
mod tests;

use super::{Inflection, nominal::Possessor};
pub(crate) use realization::Realization;

type Parts = [&'static str; 4];
const EMPTY: Parts = ["", "", "", ""];
const HIGH_BYTES: usize = "ü".len();
const MAX_BYTES: usize = "lar".len()
    + 2 * HIGH_BYTES
    + "mz".len()
    + HIGH_BYTES
    + "nn".len()
    + "ki".len()
    + HIGH_BYTES
    + "dr".len();

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TailInflection {
    plural: bool,
    possessor: Option<Possessor>,
    case: Option<Inflection>,
    ki: bool,
    copula: bool,
}

impl TailInflection {
    pub(crate) const fn case(case: Inflection) -> Self {
        Self {
            plural: false,
            possessor: None,
            case: Some(case),
            ki: false,
            copula: false,
        }
    }

    pub(crate) fn is_locative(self) -> bool {
        self == Self::case(Inflection::Locative)
    }

    pub(crate) fn has_possession(self) -> bool {
        self.possessor.is_some()
    }
    pub(crate) fn has_plural(self) -> bool {
        self.plural
    }

    /// Authored compounds insert their default possession after a newly added plural.
    pub(crate) fn with_compound_possession(mut self) -> Self {
        self.possessor = Some(Possessor::ThirdSingular);
        self
    }
}
