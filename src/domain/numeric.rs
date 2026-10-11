//! Exact number interpretations, ordinals and canonical Roman values.
use crate::{
    IssueCategory, SegmentKind,
    morphology::{Inflection, Spoken, TailInflection},
    notation::{split_suffix, suffix_parts},
    numerals::{self, Number},
};

#[derive(Clone, Debug)]
pub(crate) struct Numeric {
    number: Number,
    ordinal: bool,
    tail: Option<TailInflection>,
}

pub(crate) enum NumericPreference {
    Number(Numeric),
    SentenceNumber(Number),
}

pub(crate) struct NumericFailure {
    pub(crate) category: IssueCategory,
    pub(crate) preference: Option<NumericPreference>,
}

impl From<IssueCategory> for NumericFailure {
    fn from(category: IssueCategory) -> Self {
        Self {
            category,
            preference: None,
        }
    }
}

impl Numeric {
    pub(crate) fn automatic(text: &str) -> Result<Self, NumericFailure> {
        if text.chars().any(|c| c.is_numeric() && !c.is_ascii_digit()) {
            return Err(IssueCategory::Unsupported.into());
        }
        // Explicit ordinal syntax has its own bounded two-apostrophe profile.
        if text.matches(['\'', '’']).count() == 2 {
            return Self::parsed(text, false, false, false).map_err(Into::into);
        }
        let (base, suffix) = split_suffix(text).ok_or(IssueCategory::InvalidExpression)?;
        if base.ends_with('.') && suffix.is_none() {
            return Err(
                match Number::parse(&base[..base.len() - 1])
                    .filter(|number| number.fraction().is_empty() && !number.grouped())
                {
                    Some(number) => NumericFailure {
                        category: IssueCategory::Ambiguous,
                        preference: Some(NumericPreference::SentenceNumber(number)),
                    },
                    None => IssueCategory::InvalidExpression.into(),
                },
            );
        }
        let parsed = match Self::parsed(text, false, true, false) {
            Ok(parsed) => parsed,
            Err(_) if Number::parse(base).is_some_and(|number| number.grouped()) => {
                return Err(NumericFailure {
                    category: IssueCategory::Ambiguous,
                    preference: None,
                });
            }
            Err(category) => return Err(category.into()),
        };
        if parsed.number.grouped() {
            return Err(NumericFailure {
                category: IssueCategory::Ambiguous,
                preference: Some(NumericPreference::Number(parsed)),
            });
        }
        Ok(parsed)
    }
    pub(crate) fn cardinal_hint(text: &str) -> Option<Self> {
        let parsed = Self::parsed(text, false, true, true).ok()?;
        parsed.number.fraction().is_empty().then_some(parsed)
    }
    pub(crate) fn parse(text: &str, ordinal_hint: bool) -> Option<Self> {
        Self::parsed(text, ordinal_hint, false, false).ok()
    }
    pub(crate) fn product_operand(text: &str) -> Option<Self> {
        Self::parsed(text, false, true, false).ok()
    }
    pub(crate) fn is_count(&self) -> bool {
        self.number.sign() != numerals::Sign::Minus && self.number.fraction().is_empty()
    }
    fn parsed(
        text: &str,
        ordinal_hint: bool,
        allow_grouped: bool,
        padding: bool,
    ) -> Result<Self, IssueCategory> {
        let invalid = IssueCategory::InvalidExpression;
        let (base, suffixes) = suffix_parts(text).ok_or(invalid)?;
        let period = base.ends_with('.');
        if ordinal_hint && !period && suffixes.is_empty() {
            return Err(invalid);
        }
        let body = if period {
            &base[..base.len() - 1]
        } else {
            base
        };
        let number = if padding {
            Number::parse_cardinal_hint(body)
        } else {
            Number::parse(body)
        }
        .ok_or(invalid)?;
        if number.grouped() && !allow_grouped {
            return Err(invalid);
        }
        let source = number.tail();
        let mut ordinal = false;
        let tail;
        if period {
            if suffixes.is_empty() && !ordinal_hint {
                return Err(invalid);
            }
            if suffixes.len() > 1 || !number.fraction().is_empty() || number.grouped() {
                return Err(invalid);
            }
            ordinal = true;
            tail = suffixes
                .first()
                .map(|s| {
                    TailInflection::select(
                        source.inflected(Inflection::Ordinal),
                        s,
                        source.inflected(Inflection::Ordinal),
                    )
                })
                .transpose()?;
        } else if let Some(first) = suffixes.first() {
            if let Some(remaining) = number
                .fraction()
                .is_empty()
                .then(|| source.strip_suffix_prefix(Inflection::Ordinal, first))
                .flatten()
            {
                ordinal = true;
                let target = source.inflected(Inflection::Ordinal);
                tail = if remaining.is_empty() {
                    suffixes
                        .get(1)
                        .map(|s| TailInflection::select(target, s, target))
                        .transpose()?
                } else {
                    if suffixes.len() != 1 {
                        return Err(invalid);
                    }
                    Some(TailInflection::select(target, remaining, target)?)
                };
            } else {
                if ordinal_hint || suffixes.len() != 1 {
                    return Err(invalid);
                }
                tail = Some(TailInflection::select(source, first, source)?);
            }
        } else {
            ordinal = ordinal_hint;
            tail = None;
        }
        Ok(Self {
            number,
            ordinal,
            tail,
        })
    }
    pub(crate) fn roman(text: &str) -> Option<Self> {
        let (base, suffixes) = suffix_parts(text)?;
        let ordinal = base.ends_with('.');
        let roman = base.strip_suffix('.').unwrap_or(base);
        let value = roman_value(roman)?;
        let mut numeric = value.to_string();
        if ordinal {
            numeric.push('.');
        }
        for suffix in &suffixes {
            numeric.push('\'');
            numeric.push_str(suffix);
        }
        let parsed = Self::parse(&numeric, ordinal)?;
        if !suffixes.is_empty() && !parsed.ordinal {
            return None;
        }
        Some(parsed)
    }
    pub(crate) fn render(&self) -> Spoken {
        let mut spoken = numerals::number(&self.number);
        if self.ordinal {
            spoken.inflect(Inflection::Ordinal);
        }
        if let Some(tail) = self.tail {
            tail.apply(&mut spoken);
        }
        spoken
    }
    pub(crate) fn kind(&self) -> SegmentKind {
        if self.ordinal {
            SegmentKind::Ordinal
        } else if self.number.fraction().is_empty() {
            SegmentKind::Cardinal
        } else {
            SegmentKind::Decimal
        }
    }
}

fn roman_value(text: &str) -> Option<u16> {
    if text.is_empty() || text.len() > 15 {
        return None;
    }
    let value = |b| match b {
        b'I' => Some(1_u16),
        b'V' => Some(5),
        b'X' => Some(10),
        b'L' => Some(50),
        b'C' => Some(100),
        b'D' => Some(500),
        b'M' => Some(1000),
        _ => None,
    };
    let mut total = 0_i32;
    let mut previous = 0;
    for b in text.bytes().rev() {
        let n = value(b)?;
        total += if n < previous {
            -(i32::from(n))
        } else {
            i32::from(n)
        };
        previous = n;
    }
    if !(1..=3999).contains(&total) {
        return None;
    }
    let mut rest = total;
    let mut canonical = String::new();
    for (n, s) in [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while rest >= n {
            canonical.push_str(s);
            rest -= n;
        }
    }
    (canonical == text).then_some(total as u16)
}

#[cfg(test)]
mod tests;
