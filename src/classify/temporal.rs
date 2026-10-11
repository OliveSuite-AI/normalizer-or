use crate::{
    FallbackClass, IssueCategory,
    domain::temporal::{Clock, ClockRange, Date, DateTail},
    interpretation::{Reading, TemporalFailure, TemporalPreference, UnresolvedFinding, Value},
    morphology::Inflection,
};

use super::{
    context::{DATE_CUES, FOLLOWING_DATE_CUES, TIME_CUES, cue_key, is_clock_word},
    scan::{Token, whitespace_between},
};
use crate::notation::split_suffix;

pub(super) fn date(text: &str, permitted: bool) -> Result<Value, TemporalFailure> {
    let (base, suffix) = split_suffix(text).ok_or(IssueCategory::InvalidExpression)?;
    let date = Date::parse(base).ok_or(IssueCategory::InvalidExpression)?;
    let tail = DateTail::parse(date, suffix).ok_or(IssueCategory::InvalidExpression)?;
    if !permitted && !date.year_first_slash() {
        return Err(TemporalFailure {
            category: IssueCategory::Ambiguous,
            preference: (!date.slash()).then_some(TemporalPreference::Date(date, tail)),
        });
    }
    Ok(Value::Date(date, tail))
}

pub(super) fn time(text: &str, permitted: bool) -> Result<Value, TemporalFailure> {
    let (base, suffix) = split_suffix(text).ok_or(IssueCategory::InvalidExpression)?;
    let time = Clock::parse(base).ok_or(IssueCategory::InvalidExpression)?;
    if let Some(suffix) = suffix
        && time
            .tail()
            .strip_suffix_prefix(Inflection::Locative, suffix)
            != Some("")
    {
        return Err(IssueCategory::InvalidExpression.into());
    }
    if !permitted && !(base.contains('.') && suffix.is_some()) {
        return Err(TemporalFailure {
            category: IssueCategory::Ambiguous,
            preference: Some(TemporalPreference::Time(time, suffix.is_some())),
        });
    }
    Ok(Value::Time(time, suffix.is_some()))
}

fn cue(text: &str, tokens: &[Token<'_>], index: usize, allowed: &[&str]) -> bool {
    let Some(mut previous_index) = index.checked_sub(1) else {
        return false;
    };
    let adjacent = |start, end| start == end || whitespace_between(text, start, end);
    if !adjacent(tokens[previous_index].range.end, tokens[index].range.start) {
        return false;
    }
    if tokens[previous_index].text == ":" {
        let Some(word_index) = previous_index.checked_sub(1) else {
            return false;
        };
        if !whitespace_between(
            text,
            tokens[word_index].range.end,
            tokens[previous_index].range.start,
        ) || tokens[word_index].text.contains(':')
        {
            return false;
        }
        previous_index = word_index;
    }
    allowed.contains(&cue_key(tokens[previous_index].text).as_str())
}

fn frame(text: &str, tokens: &[Token<'_>], index: usize) -> bool {
    let (Some(date_token), Some(cue_token), Some(time_token)) = (
        tokens.get(index),
        tokens.get(index + 1),
        tokens.get(index + 2),
    ) else {
        return false;
    };
    matches!(date(date_token.text, true), Ok(Value::Date(_, tail)) if tail.is_locative())
        && is_clock_word(cue_token.text)
        && matches!(time(time_token.text, true), Ok(Value::Time(_, true)))
        && whitespace_between(text, date_token.range.end, cue_token.range.start)
        && whitespace_between(text, cue_token.range.end, time_token.range.start)
}

pub(super) fn recognize(
    text: &str,
    tokens: &[Token<'_>],
    index: usize,
) -> Option<(Result<Value, TemporalFailure>, FallbackClass)> {
    let token = tokens[index].text;
    let (base, suffix) = split_suffix(token)?;
    if !base.starts_with(char::is_numeric)
        || !base
            .chars()
            .all(|ch| ch.is_numeric() || matches!(ch, '.' | ':' | '-' | '/'))
    {
        return None;
    }
    let dots = base.bytes().filter(|b| *b == b'.').count();
    let hyphens = base.bytes().filter(|b| *b == b'-').count();
    let slashes = base.bytes().filter(|b| *b == b'/').count();
    if dots == 2 || ((hyphens == 2 || slashes == 2) && base.len() >= 8) {
        if dots == 2
            && crate::numerals::Number::parse(base).is_some_and(|n| n.grouped())
            && !cue(text, tokens, index, DATE_CUES)
        {
            return None;
        }
        return Some((
            date(
                token,
                frame(text, tokens, index)
                    || cue(text, tokens, index, DATE_CUES)
                    || tokens.get(index + 1).is_some_and(|following| {
                        whitespace_between(text, tokens[index].range.end, following.range.start)
                            && FOLLOWING_DATE_CUES.contains(&cue_key(following.text).as_str())
                    }),
            ),
            FallbackClass::Date,
        ));
    }
    let dotted_clock = base.split_once('.').is_some_and(|(hour, minute)| {
        (1..=2).contains(&hour.len())
            && minute.len() == 2
            && [hour, minute]
                .iter()
                .all(|part| part.bytes().all(|byte| byte.is_ascii_digit()))
    });
    if base.contains(':')
        || (dots == 1
            && (cue(text, tokens, index, TIME_CUES) || (suffix.is_some() && dotted_clock)))
    {
        return Some((
            time(token, cue(text, tokens, index, TIME_CUES)),
            FallbackClass::Time,
        ));
    }
    None
}

pub(super) fn range_hint(text: &str, following_ile: bool) -> Option<Value> {
    Some(Value::ClockRange(ClockRange::parse(text, following_ile)?))
}

pub(super) fn range(text: &str, tokens: &[Token<'_>], index: usize) -> Option<(Reading, usize)> {
    let first = tokens.get(index)?;
    let endpoint = first.text.split(['-', '–']).next()?;
    if !clock_shape(endpoint) {
        return None;
    }

    fn clock_shape(source: &str) -> bool {
        let base = source.split(['\'', '’']).next().unwrap_or(source);
        let separator = if base.contains(':') { ':' } else { '.' };
        let mut parts = base.split(separator);
        let Some(hour) = parts.next() else {
            return false;
        };
        let Some(minute) = parts.next() else {
            return false;
        };
        (1..=2).contains(&hour.len())
            && minute.len() == 2
            && (separator == ':' || parts.next().is_none())
            && [hour, minute]
                .iter()
                .all(|part| part.bytes().all(|byte| byte.is_ascii_digit()))
    }
    let mut end = index;
    let mut separator = first.text.contains(['-', '–']);
    if !separator {
        let next = tokens.get(index + 1)?;
        if !whitespace_between(text, first.range.end, next.range.start)
            || !next.text.starts_with(['-', '–'])
        {
            return None;
        }
        end += 1;
        separator = true;
    }
    if separator && tokens[end].text.ends_with(['-', '–']) {
        let next = tokens.get(end + 1)?;
        if !whitespace_between(text, tokens[end].range.end, next.range.start) {
            return None;
        }
        end += 1;
    }
    let source = &text[first.range.start..tokens[end].range.end];
    let ile = tokens.get(end + 1).is_some_and(|following| {
        whitespace_between(text, tokens[end].range.end, following.range.start)
            && cue_key(following.text) == "arasında"
    });
    Some((
        match range_hint(source, ile) {
            Some(value) => Reading::Resolved(value),
            None => Reading::Unresolved(UnresolvedFinding::unresolved(
                source,
                FallbackClass::Time,
                IssueCategory::InvalidExpression,
            )),
        },
        end,
    ))
}
