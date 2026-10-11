//! Address field boundaries and structural value validation.
use super::{
    lexicon::{Alias, Label, alias, cue, label},
    scan::{Token, whitespace},
};
use crate::{NormalizeError, SourceRange, WorkControl, domain::numeric::Numeric};

pub(super) struct Header {
    pub(super) kind: Label,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) next: usize,
    pub(super) attached: Option<SourceRange>,
}

pub(super) fn header(text: &str, tokens: &[Token], index: usize) -> Option<Header> {
    let token = *tokens.get(index)?;
    if token.protected {
        return None;
    }
    let word = token.text(text);
    let (kind, label_token, label_index) = if matches!(word, "posta" | "Posta" | "POSTA") {
        let next = *tokens.get(index + 1)?;
        let code_word = next.text(text).split(':').next()?;
        if !matches!(
            (word, code_word),
            ("posta", "kodu") | ("Posta", "Kodu") | ("Posta", "kodu") | ("POSTA", "KODU")
        ) || next.protected
            || !next.spaced
        {
            return None;
        }
        (Label::Postcode, next, index + 1)
    } else {
        (label(word.split(':').next()?)?, token, index)
    };
    let label_word = label_token.text(text);
    if let Some(colon) = label_word.find(':') {
        let end = label_token.range.start + colon + 1;
        return Some(Header {
            kind,
            start: token.range.start,
            end,
            next: label_index + 1,
            attached: (end < label_token.range.end)
                .then_some(SourceRange::new(end, label_token.range.end)),
        });
    }
    if kind == Label::Postcode
        && !tokens
            .get(label_index + 1)
            .is_some_and(|token| token.text(text).starts_with(':'))
    {
        let value = tokens.get(label_index + 1)?;
        if value.protected
            || !value.spaced
            || !value
                .text(text)
                .starts_with(|ch: char| ch.is_ascii_digit() || matches!(ch, '-' | '+'))
        {
            return None;
        }
        return Some(Header {
            kind,
            start: token.range.start,
            end: label_token.range.end,
            next: label_index + 1,
            attached: None,
        });
    }
    let colon = *tokens.get(label_index + 1)?;
    if colon.protected || !colon.spaced {
        return None;
    }
    let body = colon.text(text).strip_prefix(':')?;
    let end = colon.range.start + 1;
    Some(Header {
        kind,
        start: token.range.start,
        end,
        next: label_index + 2,
        attached: (!body.is_empty()).then_some(SourceRange::new(end, colon.range.end)),
    })
}

pub(super) fn proper_name(word: &str, control: &WorkControl) -> Result<bool, NormalizeError> {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return Ok(false);
    };
    if !first.is_uppercase() {
        return Ok(false);
    }
    for ch in chars {
        control.check()?;
        if !ch.is_alphabetic() && !matches!(ch, '-' | '\'' | '’') {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn locality(word: &str, control: &WorkControl) -> Result<Option<usize>, NormalizeError> {
    let Some(slash) = word.find('/') else {
        return Ok(None);
    };
    Ok(
        (proper_name(&word[..slash], control)? && proper_name(&word[slash + 1..], control)?)
            .then_some(slash),
    )
}

pub(super) struct Body {
    pub(super) range: SourceRange,
    pub(super) first: SourceRange,
    pub(super) second: Option<SourceRange>,
    pub(super) valid_shape: bool,
}

pub(super) fn body(
    text: &str,
    tokens: &[Token],
    header: &Header,
    control: &WorkControl,
) -> Result<(Option<Body>, usize), NormalizeError> {
    let mut next = header.next;
    let range = if let Some(attached) = header.attached {
        attached
    } else if let Some(token) = tokens.get(next) {
        if token.protected
            || self::header(text, tokens, next).is_some()
            || alias(token.text(text)).is_some()
            || cue(token.text(text))
            || locality(token.text(text), control)?.is_some()
        {
            return Ok((None, next));
        }
        next += 1;
        token.range
    } else {
        return Ok((None, next));
    };
    let mut end = range.end;
    while tokens
        .get(next)
        .is_some_and(|token| !token.protected && token.text(text).starts_with('/'))
    {
        control.check()?;
        let separator = tokens[next];
        end = separator.range.end;
        next += 1;
        if separator.text(text) == "/"
            && let Some(token) = tokens.get(next)
            && !token.protected
            && self::header(text, tokens, next).is_none()
            && alias(token.text(text)).is_none()
        {
            end = token.range.end;
            next += 1;
        }
    }
    let range = SourceRange::new(range.start, end);
    let mut slash = None;
    let mut valid_shape = whitespace(&text[header.end..range.start], control)?;
    for (offset, ch) in text[range.start..range.end].char_indices() {
        control.check()?;
        if ch == '/' && slash.replace(range.start + offset).is_some() {
            valid_shape = false;
        }
    }
    let trimmed = |start: usize, end: usize| {
        let raw = &text[start..end];
        let body = raw.trim();
        let start = start + raw.len() - raw.trim_start().len();
        SourceRange::new(start, start + body.len())
    };
    let (first, second) = match slash {
        Some(slash) => (
            trimmed(range.start, slash),
            Some(trimmed(slash + 1, range.end)),
        ),
        None => (range, None),
    };
    Ok((
        Some(Body {
            range,
            first,
            second,
            valid_shape,
        }),
        next,
    ))
}

pub(super) fn integer(
    text: &str,
    signed: bool,
    control: &WorkControl,
) -> Result<Option<Numeric>, NormalizeError> {
    let digits = if signed {
        text.strip_prefix(['-', '+']).unwrap_or(text)
    } else {
        text
    };
    if digits.is_empty() {
        return Ok(None);
    }
    let mut nonzero = false;
    for byte in digits.bytes() {
        control.check()?;
        if !byte.is_ascii_digit() {
            return Ok(None);
        }
        nonzero |= byte != b'0';
    }
    if !signed && !nonzero {
        return Ok(None);
    }
    Ok(Numeric::parse(text, false))
}

pub(super) fn postcode(text: &str) -> bool {
    text.len() == 5 && text.bytes().all(|byte| byte.is_ascii_digit())
}

pub(super) enum Field {
    Alias {
        kind: Alias,
        range: SourceRange,
        named: bool,
    },
    Street {
        number: SourceRange,
        alias: SourceRange,
        value: Option<Numeric>,
    },
    Labeled {
        header: Header,
        body: Option<Body>,
        values: Option<(Numeric, Option<Numeric>)>,
        postal_valid: bool,
    },
}

impl Field {
    pub(super) fn range(&self) -> SourceRange {
        match self {
            Self::Alias { range, .. } => *range,
            Self::Street { number, alias, .. } => SourceRange::new(number.start, alias.end),
            Self::Labeled { header, body, .. } => SourceRange::new(
                header.start,
                body.as_ref().map_or(header.end, |body| body.range.end),
            ),
        }
    }
}
