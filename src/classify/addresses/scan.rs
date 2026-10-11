//! Source tokenization, protected electronic spans and address frame boundaries.
use crate::{NormalizeError, SourceRange, WorkControl};

use super::lexicon::{Alias, alias};

#[derive(Clone, Copy)]
pub(super) struct Token {
    pub(super) range: SourceRange,
    pub(super) protected: bool,
    before: bool,
    after: bool,
    pub(super) numeric_period: bool,
    pub(super) spaced: bool,
    pub(super) frame: usize,
}

impl Token {
    pub(super) fn text(self, text: &str) -> &str {
        &text[self.range.start..self.range.end]
    }
}

pub(super) fn whitespace(text: &str, control: &WorkControl) -> Result<bool, NormalizeError> {
    for ch in text.chars() {
        control.check()?;
        if !ch.is_whitespace() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn protected(text: &str, control: &WorkControl) -> Result<bool, NormalizeError> {
    let mut previous = '\0';
    let mut before_previous = '\0';
    let mut scheme = false;
    for ch in text.chars() {
        control.check()?;
        if ch == '@' || ch == '\\' {
            return Ok(true);
        }
        if before_previous == ':' && previous == '/' && ch == '/' {
            scheme = true;
        }
        before_previous = previous;
        previous = ch;
    }
    Ok(scheme
        || text
            .get(..3)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("D:/"))
        || text
            .get(..4)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("www.")))
}

fn append_token(
    text: &str,
    mut range: SourceRange,
    protected: bool,
    before: &mut bool,
    tokens: &mut Vec<Token>,
    control: &WorkControl,
) -> Result<(), NormalizeError> {
    if range.start == range.end {
        return Ok(());
    }
    let raw = &text[range.start..range.end];
    let mut after = false;
    let mut numeric_period = false;
    if let Some(body) = raw.strip_suffix('.')
        && alias(raw).is_none()
        && !crate::domain::lexicon::plain_abbreviation(raw)
    {
        let mut digits = !body.is_empty();
        for byte in body.bytes() {
            control.check()?;
            digits &= byte.is_ascii_digit();
        }
        numeric_period = digits && !protected;
        if !numeric_period {
            range.end -= 1;
            after = true;
        }
    }
    if range.start < range.end {
        let previous_end = tokens.last().map_or(0, |token| token.range.end);
        let spaced = whitespace(&text[previous_end..range.start], control)?;
        tokens.push(Token {
            range,
            protected,
            before: *before,
            after,
            numeric_period,
            spaced,
            frame: 0,
        });
        *before = after;
    } else {
        *before = true;
    }
    Ok(())
}

fn chunk(
    text: &str,
    range: SourceRange,
    before: &mut bool,
    tokens: &mut Vec<Token>,
    control: &WorkControl,
) -> Result<(), NormalizeError> {
    let raw = &text[range.start..range.end];
    if protected(raw, control)? {
        let mut end = range.end;
        for (offset, ch) in raw.char_indices().rev() {
            control.check()?;
            if !matches!(ch, ',' | ';' | '!' | '?' | ')' | ']' | '}' | '"' | '”') {
                break;
            }
            end = range.start + offset;
        }
        let terminal_boundary = raw[end - range.start..].contains([';', '!', '?']);
        append_token(
            text,
            SourceRange::new(range.start, end),
            true,
            before,
            tokens,
            control,
        )?;
        if terminal_boundary {
            *before = true;
        }
        return Ok(());
    }
    // The shared scanner keeps compact identifier punctuation in one token.
    // Do not produce a second field whose start would be inside that token.
    if crate::classify::scan::identifier(raw) {
        let mut start = range.start;
        let mut end = range.end;
        for (offset, ch) in raw.char_indices() {
            control.check()?;
            if !matches!(ch, '(' | '[' | '{' | '"' | '“' | '«' | ',') {
                break;
            }
            start = range.start + offset + ch.len_utf8();
        }
        let mut terminal_boundary = false;
        for (offset, ch) in text[start..end].char_indices().rev() {
            control.check()?;
            if !matches!(
                ch,
                ',' | ';' | '!' | '?' | ')' | ']' | '}' | '"' | '”' | '»'
            ) {
                break;
            }
            terminal_boundary |= matches!(ch, ';' | '!' | '?');
            end = start + offset;
        }
        append_token(
            text,
            SourceRange::new(start, end),
            false,
            before,
            tokens,
            control,
        )?;
        *before |= terminal_boundary;
        return Ok(());
    }
    let mut start = range.start;
    for (offset, ch) in raw.char_indices() {
        control.check()?;
        let absolute = range.start + offset;
        let decimal_comma = ch == ','
            && absolute > start
            && text.as_bytes()[absolute - 1].is_ascii_digit()
            && text
                .as_bytes()
                .get(absolute + 1)
                .is_some_and(u8::is_ascii_digit);
        if !matches!(
            ch,
            ',' | ';' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '“' | '”' | '«' | '»'
        ) || decimal_comma
        {
            continue;
        }
        append_token(
            text,
            SourceRange::new(start, absolute),
            false,
            before,
            tokens,
            control,
        )?;
        if matches!(ch, ';' | '!' | '?') {
            *before = true;
        }
        start = absolute + ch.len_utf8();
    }
    append_token(
        text,
        SourceRange::new(start, range.end),
        false,
        before,
        tokens,
        control,
    )
}

pub(super) fn tokens(text: &str, control: &WorkControl) -> Result<Vec<Token>, NormalizeError> {
    let quoted_emails = quoted_emails(text, control)?;
    let mut quoted_index = 0;
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut before = false;
    for (offset, ch) in text.char_indices() {
        control.check()?;
        if quoted_emails
            .get(quoted_index)
            .is_some_and(|range| range.end <= offset)
        {
            quoted_index += 1;
        }
        let quoted = quoted_emails
            .get(quoted_index)
            .is_some_and(|range| range.start <= offset && offset < range.end);
        if ch.is_whitespace() && !quoted {
            chunk(
                text,
                SourceRange::new(start, offset),
                &mut before,
                &mut tokens,
                control,
            )?;
            if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                before = true;
            }
            start = offset + ch.len_utf8();
        }
    }
    chunk(
        text,
        SourceRange::new(start, text.len()),
        &mut before,
        &mut tokens,
        control,
    )?;
    for index in 0..tokens.len() {
        control.check()?;
        if tokens[index].numeric_period
            && !tokens.get(index + 1).is_some_and(|next| {
                !next.before
                    && !next.protected
                    && next.spaced
                    && alias(next.text(text)) == Some(Alias::Street)
            })
        {
            tokens[index].range.end -= 1;
            tokens[index].after = true;
        }
    }
    let mut frame = 0;
    let mut previous_after = false;
    for token in &mut tokens {
        control.check()?;
        if previous_after || token.before {
            frame += 1;
        }
        token.frame = frame;
        previous_after = token.after;
    }
    Ok(tokens)
}

fn quoted_emails(text: &str, control: &WorkControl) -> Result<Vec<SourceRange>, NormalizeError> {
    let mut ranges = Vec::new();
    let mut opening = None;
    let mut address = None;
    let mut previous_space = true;
    for (offset, ch) in text.char_indices() {
        control.check()?;
        if let Some(start) = address {
            if ch.is_whitespace() {
                ranges.push(SourceRange::new(start, offset));
                address = None;
            }
        } else if ch == '"' {
            if let Some(start) = opening.take() {
                if text.as_bytes().get(offset + 1) == Some(&b'@') {
                    address = Some(start);
                }
            } else if previous_space {
                opening = Some(offset);
            }
        } else if matches!(ch, '\r' | '\n') {
            opening = None;
        }
        previous_space = ch.is_whitespace();
    }
    if let Some(start) = address {
        ranges.push(SourceRange::new(start, text.len()));
    }
    Ok(ranges)
}
