//! Reviewed, frame-local address fields. Free address names remain independent prose.
mod emission;
mod fields;
mod lexicon;
mod scan;
#[cfg(test)]
mod tests;

use crate::{
    LimitKind, MAX_INPUT_BYTES, NormalizeError, SegmentKind, SourceRange, WorkControl,
    domain::numeric::Numeric,
};

use super::Candidate;
use emission::{emit, push, resolved, text_component};
use fields::{Field, body, header, integer, locality, postcode, proper_name};
use lexicon::{Alias, Label, alias, cue};
use scan::{Token, tokens};

fn frame(
    text: &str,
    tokens: &[Token],
    candidates: &mut Vec<Candidate>,
    control: &WorkControl,
) -> Result<(), NormalizeError> {
    let mut fields = Vec::new();
    let mut cue_start = None;
    let mut apartment = false;
    let mut structural = [false; 3];
    let mut named_aliases = [false; 3];
    let mut has_labeled = false;
    let mut actual_field = false;
    let mut index = 0;
    while index < tokens.len() {
        control.check()?;
        let token = tokens[index];
        if token.protected {
            index += 1;
            continue;
        }
        let word = token.text(text);
        if cue(word) {
            cue_start.get_or_insert(token.range.start);
        }
        if let Some(header) = header(text, tokens, index) {
            has_labeled = true;
            actual_field = true;
            apartment |= header.kind == Label::Apartment;
            let (body, next) = body(text, tokens, &header, control)?;
            let postal_valid = header.kind == Label::Postcode
                && body
                    .as_ref()
                    .is_some_and(|body| postcode(&text[body.range.start..body.range.end]));
            let values = if header.kind == Label::Postcode {
                None
            } else if let Some(body) = &body
                && body.valid_shape
            {
                let first = integer(
                    &text[body.first.start..body.first.end],
                    header.kind == Label::Floor,
                    control,
                )?;
                let second = match body.second {
                    Some(range) if header.kind == Label::Number => {
                        integer(&text[range.start..range.end], false, control)?
                    }
                    _ => None,
                };
                if body.second.is_some() && second.is_none() {
                    None
                } else {
                    first.map(|first| (first, second))
                }
            } else {
                None
            };
            if values.is_some() {
                structural[match header.kind {
                    Label::Number => 0,
                    Label::Floor => 1,
                    Label::Apartment => 2,
                    Label::Postcode => unreachable!(),
                }] = true;
            }
            fields.push(Field::Labeled {
                header,
                body,
                values,
                postal_valid,
            });
            index = next;
            continue;
        }
        if token.numeric_period
            && let Some(next) = tokens.get(index + 1)
            && alias(next.text(text)) == Some(Alias::Street)
        {
            let value = integer(word.trim_end_matches('.'), false, control)?
                .and_then(|_| Numeric::parse(word, true));
            named_aliases[2] |= value.is_some();
            actual_field = true;
            fields.push(Field::Street {
                number: token.range,
                alias: next.range,
                value,
            });
            index += 2;
            continue;
        }
        if let Some(kind) = alias(word) {
            let named = if let Some(previous) = index.checked_sub(1).map(|i| tokens[i]) {
                !previous.protected
                    && !cue(previous.text(text))
                    && token.spaced
                    && proper_name(previous.text(text), control)?
            } else {
                false
            };
            if named {
                named_aliases[kind as usize] = true;
                actual_field = true;
            }
            fields.push(Field::Alias {
                kind,
                range: token.range,
                named,
            });
        }
        index += 1;
    }
    let named_count = named_aliases.into_iter().filter(|named| *named).count();
    let structural_count = structural.into_iter().filter(|valid| *valid).count();
    let explicit = cue_start.is_some() && actual_field;
    let inferred = named_count >= 2 || (named_count > 0 && has_labeled) || structural_count >= 2;
    if !explicit && !inferred {
        for field in fields {
            control.check()?;
            if matches!(&field, Field::Labeled { header, .. } if header.kind == Label::Postcode) {
                emit(text, field, apartment, candidates)?;
            }
        }
        return Ok(());
    }
    let start = if explicit {
        cue_start.unwrap()
    } else {
        tokens[0].range.start
    };
    let mut tail = vec![false; tokens.len() + 1];
    tail[tokens.len()] = true;
    for index in (0..tokens.len()).rev() {
        control.check()?;
        tail[index] = !tokens[index].protected
            && (proper_name(tokens[index].text(text), control)?
                || locality(tokens[index].text(text), control)?.is_some())
            && tail[index + 1];
    }
    let mut fields = fields.into_iter().peekable();
    let mut previous_field = false;
    let mut index = 0;
    while index < tokens.len() {
        control.check()?;
        let token = tokens[index];
        if token.range.start < start {
            index += 1;
            continue;
        }
        if fields
            .peek()
            .is_some_and(|field| field.range().start < start)
        {
            fields.next();
            continue;
        }
        if fields
            .peek()
            .is_some_and(|field| field.range().start == token.range.start)
        {
            let field = fields.next().unwrap();
            let range = field.range();
            let eligible = !matches!(&field, Field::Alias { named: false, .. }) || previous_field;
            if eligible {
                emit(text, field, apartment, candidates)?;
                previous_field = true;
            }
            while index < tokens.len() && tokens[index].range.start < range.end {
                control.check()?;
                index += 1;
            }
            continue;
        }
        if cue(token.text(text)) && token.text(text).ends_with(':') {
            push(
                candidates,
                resolved(
                    token.range,
                    vec![text_component(
                        token.range,
                        SegmentKind::Verbatim,
                        token.text(text),
                    )],
                ),
            )?;
        }
        if previous_field && !token.protected {
            let word = token.text(text);
            if postcode(word) && index + 1 < tokens.len() && tail[index + 1] {
                push(
                    candidates,
                    resolved(
                        token.range,
                        vec![text_component(
                            token.range,
                            SegmentKind::Digits,
                            format!("posta kodu {}", crate::numerals::digits(word)),
                        )],
                    ),
                )?;
            } else if let Some(slash) = locality(word, control)?
                && tail[index + 1]
            {
                let separator = token.range.start + slash;
                push(
                    candidates,
                    resolved(
                        token.range,
                        vec![
                            text_component(
                                SourceRange::new(token.range.start, separator),
                                SegmentKind::Verbatim,
                                &word[..slash],
                            ),
                            text_component(
                                SourceRange::new(separator, separator + 1),
                                SegmentKind::Symbol,
                                ", ",
                            ),
                            text_component(
                                SourceRange::new(separator + 1, token.range.end),
                                SegmentKind::Verbatim,
                                &word[slash + 1..],
                            ),
                        ],
                    ),
                )?;
            }
        }
        index += 1;
    }
    Ok(())
}

pub(crate) fn collect(text: &str, control: &WorkControl) -> Result<Vec<Candidate>, NormalizeError> {
    control.check()?;
    if text.len() > MAX_INPUT_BYTES {
        return Err(NormalizeError::LimitExceeded(LimitKind::Input));
    }
    // Every field needs punctuation, except an explicit unpunctuated postal label.
    if !text.contains([':', '.'])
        && !text.contains("posta")
        && !text.contains("Posta")
        && !text.contains("POSTA")
    {
        return Ok(Vec::new());
    }
    let tokens = tokens(text, control)?;
    let mut candidates = Vec::new();
    let mut start = 0;
    while start < tokens.len() {
        control.check()?;
        let mut end = start + 1;
        while end < tokens.len() && tokens[end].frame == tokens[start].frame {
            control.check()?;
            end += 1;
        }
        frame(text, &tokens[start..end], &mut candidates, control)?;
        start = end;
    }
    Ok(candidates)
}
