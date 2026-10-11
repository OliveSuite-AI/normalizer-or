//! Validated numeric endpoints with optional approved unit or noun context.
use crate::{
    domain::lexicon::{self, Lexeme},
    numerals::{self, Number},
};

#[derive(Clone, Debug)]
pub(crate) struct NumericRange {
    start: Number,
    end: Number,
    context: Option<Lexeme>,
    noun: Option<String>,
}

impl NumericRange {
    pub(crate) fn parse(text: &str, context: Option<&str>) -> Option<Self> {
        let mut found = None;
        for (offset, ch) in text.char_indices() {
            if !matches!(ch, '-' | '–') || offset == 0 {
                continue;
            }

            if let (Some(start), Some(end)) = (
                Number::parse(text[..offset].trim()),
                Number::parse(text[offset + ch.len_utf8()..].trim()),
            ) {
                if found.is_some() {
                    return None;
                }
                found = Some((start, end));
            }
        }
        let (start, end) = found?;
        let (unit, noun) = match context {
            Some(value)
                if ["kişi", "adet", "gün", "yaş"]
                    .contains(&lexicon::lookup_key(value).as_str()) =>
            {
                (None, Some(value.to_owned()))
            }
            Some(value) => (Some(lexicon::unit(value)?), None),
            None => (None, None),
        };
        Some(Self {
            start,
            end,
            context: unit,
            noun,
        })
    }
    pub(crate) fn render(&self) -> String {
        let mut result = format!(
            "{} ila {}",
            numerals::number(&self.start).into_text(),
            numerals::number(&self.end).into_text()
        );
        if let Some(unit) = self.context {
            result.push(' ');
            result.push_str(unit.output);
        }
        if let Some(noun) = &self.noun {
            result.push(' ');
            result.push_str(noun);
        }
        result
    }
}
