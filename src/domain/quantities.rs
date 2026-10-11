//! Validated money, measurements, percentages and contextual ranges.
mod percentage;
mod range;

pub(crate) use percentage::{PercentTail, percent, percent_body};
pub(crate) use range::NumericRange;

use super::lexicon::{self, Currency, Lexeme};
use crate::{
    IssueCategory,
    morphology::{Realization, Spoken, TailInflection, Word, case_inflection},
    notation::split_suffix,
    numerals::{self, Amount, Number},
};

#[derive(Clone, Debug)]
enum QuantityValue {
    Number(Number),
    Money(Amount, Currency),
}

#[derive(Clone, Debug)]
pub(crate) struct Quantity {
    value: QuantityValue,
    lexeme: Lexeme,
    prefix: Option<Lexeme>,
    tail: Option<TailInflection>,
}

impl Quantity {
    pub(crate) fn parse(number: &str, label: &str) -> Result<Self, IssueCategory> {
        let (base, suffix) = split_suffix(label).ok_or(IssueCategory::InvalidExpression)?;
        Self::parse_with_suffix(number, base, suffix)
    }
    pub(crate) fn parse_with_suffix(
        number: &str,
        base: &str,
        suffix: Option<&str>,
    ) -> Result<Self, IssueCategory> {
        let invalid = IssueCategory::InvalidExpression;
        let (value, lexeme, prefix) = if let Some(currency) = Currency::parse(base) {
            (
                QuantityValue::Money(Amount::parse(number).ok_or(invalid)?, currency),
                currency.lexeme(base),
                None,
            )
        } else if let Some((prefix, lexeme)) = lexicon::rate(base) {
            (
                QuantityValue::Number(Number::parse(number).ok_or(invalid)?),
                lexeme,
                Some(prefix),
            )
        } else {
            (
                QuantityValue::Number(Number::parse(number).ok_or(invalid)?),
                lexicon::unit(base).ok_or(IssueCategory::Unsupported)?,
                None,
            )
        };
        let mut quantity = Self {
            value,
            lexeme,
            prefix,
            tail: None,
        };
        quantity.tail = suffix
            .map(|suffix| quantity.select_tail(suffix))
            .transpose()?;
        Ok(quantity)
    }

    /// Validates the complete isolated leaf, without claiming arithmetic continuations.
    pub(crate) fn parse_operand(source: &str) -> Result<Self, IssueCategory> {
        let invalid = IssueCategory::InvalidExpression;
        if source.trim() != source {
            return Err(invalid);
        }
        if let Some(offset) = source.find(char::is_whitespace) {
            let label = &source[..offset];
            if crate::notation::label(label) {
                let number = source[offset..].trim();
                if matches!(label, "₺" | "$" | "€" | "£") {
                    let (body, suffix) = split_suffix(number).ok_or(invalid)?;
                    return Self::parse_with_suffix(body, label, suffix);
                }
                return Self::parse(number, label);
            }
        }
        let (body, suffix) = split_suffix(source).ok_or(invalid)?;
        if let Some((number, label)) = crate::notation::attached_quantity(body)
            && crate::notation::label(label)
        {
            return Self::parse_with_suffix(number.trim(), label, suffix);
        }
        if let Some(offset) = body.rfind(char::is_whitespace) {
            let label = &body[offset..].trim();
            if crate::notation::label(label) {
                return Self::parse_with_suffix(body[..offset].trim(), label, suffix);
            }
        }
        Err(invalid)
    }

    fn target_tail(&self, form: TailInflection) -> (&'static str, Word, TailInflection) {
        if let QuantityValue::Money(amount, currency) = &self.value {
            if amount.minor() != 0 {
                let minor = currency.minor();
                return (minor.output, minor.target, form);
            }
            if let Some(head) = currency.compound_head()
                && (form.has_possession() || form.has_plural())
            {
                let form = if form.has_possession() {
                    form
                } else {
                    form.with_compound_possession()
                };
                return (head.output, head.target, form);
            }
        }
        (self.lexeme.output, self.lexeme.target, form)
    }

    fn select_tail(&self, suffix: &str) -> Result<TailInflection, IssueCategory> {
        if let Some(case) = case_inflection(self.lexeme.source, suffix) {
            return Ok(TailInflection::case(case));
        }
        let mut selected = None;
        let mut realization: Option<(&str, Realization)> = None;
        let mut ambiguous = false;
        TailInflection::analyses(self.lexeme.source, suffix, |form| {
            let (head, target, target_form) = self.target_tail(form);
            let candidate = target_form.realization(target);
            if realization
                .is_some_and(|(old_head, old)| old_head != head || !old.same_speech(candidate))
            {
                ambiguous = true;
            } else {
                selected.get_or_insert(form);
                realization = Some((head, candidate));
            }
        });
        if ambiguous {
            Err(IssueCategory::Ambiguous)
        } else {
            selected.ok_or(IssueCategory::InvalidExpression)
        }
    }
    pub(crate) fn render(&self) -> Spoken {
        let mut spoken = match &self.value {
            QuantityValue::Number(number) => {
                let number = numerals::number(number).into_text();
                let text = if let Some(numerator) = self.prefix {
                    format!("{number} {} {}", numerator.output, self.lexeme.output)
                } else {
                    format!("{number} {}", self.lexeme.output)
                };
                Spoken::lexical(&text, self.lexeme.target)
            }
            QuantityValue::Money(amount, currency) => {
                let (output, target, _) = self.target_tail(self.tail.unwrap_or_default());
                let (major, major_tail) = if amount.minor() == 0 {
                    (output, target)
                } else {
                    (self.lexeme.output, self.lexeme.target)
                };
                numerals::amount(amount, major, major_tail, currency.minor().target)
            }
        };
        if let Some(tail) = self.tail {
            self.target_tail(tail).2.apply(&mut spoken);
        }
        spoken
    }
    pub(crate) fn is_money(&self) -> bool {
        matches!(self.value, QuantityValue::Money(..))
    }
}

#[cfg(test)]
mod tests;
