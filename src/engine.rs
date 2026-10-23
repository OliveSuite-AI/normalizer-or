//! Bounded Odia recognition, rendering, policy selection, and source composition.
use std::mem::size_of;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    AmbiguityPolicy, CleanOptions, DigitStyle, FallbackClass, FallbackDiagnostic, FallbackReason,
    FallbackStrategy, HintKind, Issue, IssueCategory, LimitKind, MAX_CANDIDATES, MAX_HINTS,
    MAX_INPUT_BYTES, MAX_RESULT_BYTES, NORMALIZER_ID, NormalizeError, NormalizeOptions,
    NormalizeResult, Segment, SegmentKind, SourceRange, WhitespacePolicy, WorkControl,
    numerals::{self, Number},
    source_map::SourceMap,
    vocabulary,
};

mod boundaries;
mod formula;

type Reading = Result<(SegmentKind, String, &'static str), IssueCategory>;

fn validate(input: &str, control: &WorkControl) -> Result<(), NormalizeError> {
    control.check()?;
    if input.len() > MAX_INPUT_BYTES {
        return Err(NormalizeError::LimitExceeded(LimitKind::Input));
    }
    if input.trim().is_empty() || input.chars().any(|c| {
        (c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
    }) { return Err(NormalizeError::InvalidInput); }
    Ok(())
}

struct Output {
    result: NormalizeResult,
    bytes: usize,
    work: usize,
}

impl Output {
    fn new() -> Self {
        Self {
            result: NormalizeResult {
                normalized_text: String::new(),
                locale: "or-IN",
                normalizer_id: NORMALIZER_ID,
                complete: true,
                segments: Vec::new(),
                issues: Vec::new(),
                fallbacks: Vec::new(),
            },
            bytes: size_of::<NormalizeResult>(),
            work: 0,
        }
    }
    fn charge(&mut self, bytes: usize) -> Result<(), NormalizeError> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .filter(|n| *n <= MAX_RESULT_BYTES)
            .ok_or(NormalizeError::LimitExceeded(LimitKind::Result))?;
        Ok(())
    }
    fn push(
        &mut self,
        range: SourceRange,
        kind: SegmentKind,
        text: String,
        rule_id: &'static str,
    ) -> Result<(), NormalizeError> {
        if range.start == range.end {
            return Ok(());
        }
        if range.start != self.result.segments.last().map_or(0, |s| s.range.end) {
            return Err(NormalizeError::Internal);
        }
        let merge = self.result.segments.last().is_some_and(|s| {
            s.kind == kind
                && s.rule_id == rule_id
                && matches!(kind, SegmentKind::Verbatim | SegmentKind::Normalization)
        });
        self.charge(
            text.len()
                .checked_mul(2)
                .ok_or(NormalizeError::LimitExceeded(LimitKind::Result))?,
        )?;
        if merge {
            let last = self
                .result
                .segments
                .last_mut()
                .ok_or(NormalizeError::Internal)?;
            last.range.end = range.end;
            last.text.push_str(&text);
        } else {
            if kind != SegmentKind::Verbatim {
                self.work += 1;
                if self.work > MAX_CANDIDATES {
                    return Err(NormalizeError::LimitExceeded(LimitKind::Candidates));
                }
            }
            self.charge(size_of::<Segment>())?;
            self.result.segments.push(Segment {
                range,
                kind,
                text: text.clone(),
                rule_id,
            });
        }
        self.result.normalized_text.push_str(&text);
        Ok(())
    }
    fn verbatim(&mut self, input: &str, start: usize, end: usize) -> Result<(), NormalizeError> {
        self.push(
            SourceRange::new(start, end),
            SegmentKind::Verbatim,
            input[start..end].to_owned(),
            "source.verbatim",
        )
    }
    fn emit(
        &mut self,
        input: &str,
        range: SourceRange,
        reading: Reading,
        policy: AmbiguityPolicy,
        control: &WorkControl,
    ) -> Result<(), NormalizeError> {
        match reading {
            Ok((kind, text, rule)) => self.push(range, kind, text, rule),
            Err(category) if policy == AmbiguityPolicy::Fallback => {
                self.charge(size_of::<FallbackDiagnostic>())?;
                let (text, strategy) = literal(&input[range.start..range.end], control)?;
                self.result.fallbacks.push(FallbackDiagnostic {
                    range,
                    attempted_class: if category == IssueCategory::ProtectedIdentifier {
                        FallbackClass::Identifier
                    } else {
                        FallbackClass::Expression
                    },
                    reason: if category == IssueCategory::ProtectedIdentifier {
                        FallbackReason::ProtectedIdentifier
                    } else if category == IssueCategory::Unsupported {
                        FallbackReason::UnsupportedForm
                    } else {
                        FallbackReason::InvalidForm
                    },
                    original_category: Some(category),
                    strategy,
                });
                self.push(range, SegmentKind::Fallback, text, "source.fallback")
            }
            Err(category) => {
                self.charge(size_of::<Issue>())?;
                self.result.issues.push(Issue {
                    range,
                    category,
                    explanation: match category {
                        IssueCategory::ProtectedIdentifier => {
                            "structured source is protected from partial normalization"
                        }
                        IssueCategory::Unsupported => {
                            "expression is outside the supported Odia core"
                        }
                        _ => "expression has an invalid supported format or value",
                    },
                });
                self.result.complete = false;
                self.push(
                    range,
                    SegmentKind::Unresolved,
                    input[range.start..range.end].to_owned(),
                    "source.unresolved",
                )
            }
        }
    }
}

pub(crate) fn clean(
    input: &str,
    options: &CleanOptions,
    control: &WorkControl,
) -> Result<NormalizeResult, NormalizeError> {
    validate(input, control)?;
    let mut output = Output::new();
    let mut graphemes = input.grapheme_indices(true).peekable();
    while let Some((start, grapheme)) = graphemes.next() {
        control.check()?;
        let mut end = start + grapheme.len();
        let text: String = if options.whitespace == WhitespacePolicy::Collapse
            && grapheme.chars().all(char::is_whitespace)
        {
            while let Some(&(next, g)) = graphemes.peek() {
                if !g.chars().all(char::is_whitespace) {
                    break;
                }
                end = next + g.len();
                graphemes.next();
            }
            " ".to_owned()
        } else {
            grapheme
                .nfc()
                .map(|ch| match (options.digits, numerals::digit(ch)) {
                    (DigitStyle::Ascii, Some(d)) => char::from(b'0' + d),
                    (DigitStyle::Odia, Some(d)) => {
                        char::from_u32('୦' as u32 + u32::from(d)).expect("Odia digit")
                    }
                    _ => ch,
                })
                .collect()
        };
        let changed = text != input[start..end];
        output.push(
            SourceRange::new(start, end),
            if changed {
                SegmentKind::Normalization
            } else {
                SegmentKind::Verbatim
            },
            text,
            if changed {
                "text.cleanup"
            } else {
                "source.verbatim"
            },
        )?;
    }
    control.check()?;
    Ok(output.result)
}

fn electronic(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.starts_with("http:")
        || lower.starts_with("https:")
        || lower.starts_with("www.")
        || text.contains('@')
    {
        return true;
    }
    let Some((_, suffix)) = lower.rsplit_once('.') else {
        return false;
    };
    if !matches!(
        suffix,
        "com" | "net" | "org" | "in" | "gov" | "edu" | "io" | "ai" | "app" | "dev" | "co"
    ) {
        return false;
    }
    lower.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    })
}

fn currency(s: &str) -> bool {
    matches!(s, "₹" | "INR")
}
fn label(s: &str) -> bool {
    currency(s)
        || vocabulary::unit(s).is_some()
        || s == "%"
        || matches!(s, "USD" | "EUR" | "GBP" | "$" | "€" | "£")
        || matches!(
            s.to_ascii_lowercase().as_str(),
            "kg" | "mg" | "km" | "cm" | "mm" | "ml" | "inr"
        )
}

fn has_digit(s: &str) -> bool {
    s.chars().any(|ch| numerals::digit(ch).is_some())
}

pub(crate) fn normalize(
    input: &str,
    options: &NormalizeOptions,
    control: &WorkControl,
) -> Result<NormalizeResult, NormalizeError> {
    validate(input, control)?;
    if options.hints.len() > MAX_HINTS {
        return Err(NormalizeError::LimitExceeded(LimitKind::Hints));
    }
    let source = SourceMap::new(input, control)?;
    let hints = source.hints(&options.hints)?;
    let text = source.text();
    let atoms = boundaries::scan(text, control)?;
    let mut output = Output::new();
    let mut index = 0;
    let mut hint_index = 0;
    let mut cursor = 0;
    while index < atoms.len() {
        control.check()?;
        let mut detected = boundaries::claim(text, &atoms, index);
        let mut range = detected.range;
        let mut next = detected.next;
        let hint = hints.get(hint_index);
        if hint.is_none() && !detected.protected && !detected.invalid {
            let mut best = formula::read(&text[range.start..range.end]);
            if best.is_some() {
                let mut end_index = index + 1;
                while end_index < atoms.len() && end_index - index < 48 {
                    let gap = &text[atoms[end_index - 1].range.end..atoms[end_index].range.start];
                    if gap.contains(['\n', '\r', '!', '?', ';', '।', '॥']) {
                        break;
                    }
                    let candidate_end = atoms[end_index].range.end;
                    if let Some(reading) = formula::read(&text[range.start..candidate_end]) {
                        best = Some(reading);
                        range.end = candidate_end;
                        next = end_index + 1;
                    }
                    end_index += 1;
                }
                if let Some(reading) = best {
                    detected.range = range;
                    detected.next = next;
                    detected.invalid = false;
                    let mapped = source
                        .original(source.cover(range))
                        .map_err(|_| NormalizeError::Internal)?;
                    output.verbatim(input, cursor, mapped.start)?;
                    output.emit(
                        input,
                        mapped,
                        Ok(reading),
                        options.ambiguity_policy,
                        control,
                    )?;
                    cursor = mapped.end;
                    index = next;
                    continue;
                }
            }
        }
        let reading = if let Some(hint) = hint.filter(|h| h.range.start == range.start) {
            if hint.range.end < range.end {
                return Err(NormalizeError::InvalidHint);
            }
            while range.end < hint.range.end {
                let part = atoms.get(next).ok_or(NormalizeError::InvalidHint)?;
                range.end = part.range.end;
                next += 1;
            }
            if range.end != hint.range.end {
                return Err(NormalizeError::InvalidHint);
            }
            if detected.protected && hint.kind != HintKind::Digits {
                return Err(NormalizeError::InvalidHint);
            }
            hint_index += 1;
            Some(
                read_hint(&text[range.start..range.end], hint.kind)
                    .ok_or(NormalizeError::InvalidHint),
            )
        } else {
            if hint.is_some_and(|h| h.range.start < range.end) {
                return Err(NormalizeError::InvalidHint);
            }
            if detected.protected {
                Some(Ok(Err(IssueCategory::ProtectedIdentifier)))
            } else if detected.invalid {
                Some(Ok(formula::read(&text[range.start..range.end])
                    .map(Ok)
                    .unwrap_or(Err(IssueCategory::InvalidExpression))))
            } else {
                let candidate = &text[range.start..range.end];
                let isolated_coefficient = candidate.len() >= 2
                    && candidate.as_bytes()[..candidate.len() - 1]
                        .iter()
                        .all(u8::is_ascii_digit)
                    && candidate.as_bytes()[candidate.len() - 1].is_ascii_alphabetic()
                    && vocabulary::unit(&candidate[candidate.len() - 1..]).is_none();
                if isolated_coefficient {
                    None
                } else {
                    formula::read(candidate)
                        .map(|value| Ok(Ok(value)))
                        .or_else(|| read(candidate, options.ambiguity_policy).map(Ok))
                }
            }
        };
        if let Some(reading) = reading {
            let reading = reading?;
            // Whole original graphemes only; no substring is extracted from an identifier.
            let range = source
                .original(source.cover(range))
                .map_err(|_| NormalizeError::Internal)?;
            output.verbatim(input, cursor, range.start)?;
            output.emit(input, range, reading, options.ambiguity_policy, control)?;
            cursor = range.end;
        }
        index = next;
    }
    if hint_index != hints.len() {
        return Err(NormalizeError::InvalidHint);
    }
    output.verbatim(input, cursor, input.len())?;
    control.check()?;
    if options.ambiguity_policy == AmbiguityPolicy::Reject && !output.result.complete {
        return Err(NormalizeError::Unresolved(output.result.issues));
    }
    Ok(output.result)
}

fn read_hint(text: &str, hint: HintKind) -> Option<Reading> {
    let text = numerals::ascii(text);
    match hint {
        HintKind::Cardinal => {
            let number = Number::parse(&text, true)?;
            if !number.fraction.is_empty() {
                return None;
            }
            Some(Ok((
                SegmentKind::Cardinal,
                number.render(true),
                "number.cardinal.hint",
            )))
        }
        HintKind::Digits => {
            let body = text.strip_prefix('+').unwrap_or(&text);
            if !body.bytes().any(|b| b.is_ascii_digit())
                || !body
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, ' ' | '.' | '/' | '-' | '(' | ')'))
            {
                return None;
            }
            Some(Ok((
                SegmentKind::Digits,
                format!(
                    "{}{}",
                    if text.starts_with('+') {
                        "ଯୁକ୍ତ "
                    } else {
                        ""
                    },
                    numerals::digits(body)
                ),
                "number.digits.hint",
            )))
        }
        HintKind::Date => Some(Ok((SegmentKind::Date, date(&text)?, "date.gregorian"))),
        HintKind::Time => Some(Ok((SegmentKind::Time, time(&text)?, "time.digital"))),
    }
}

fn read(text: &str, policy: AmbiguityPolicy) -> Option<Reading> {
    if electronic(text) {
        return Some(Err(IssueCategory::ProtectedIdentifier));
    }
    let ascii = numerals::ascii(text);
    let s = ascii.as_str();
    let invalid = || Some(Err(IssueCategory::InvalidExpression));
    if has_digit(s) {
        if let Some(number) = s
            .strip_suffix(':')
            .and_then(|body| Number::parse(body, true))
            && number.fraction.is_empty()
        {
            return Some(Ok((
                SegmentKind::Cardinal,
                format!("{}:", number.render(false)),
                "number",
            )));
        }
        // Calendar/clock recognition owns invalid values as well as valid values.
        if s.split(['/', '-']).count() == 3 && !s.starts_with(['+', '-']) {
            return Some(
                date(s)
                    .map(|v| (SegmentKind::Date, v, "date.gregorian"))
                    .ok_or(IssueCategory::InvalidExpression),
            );
        }
        if s.contains(':') && s.starts_with(|c: char| c.is_ascii_digit()) {
            return Some(
                time(s)
                    .map(|v| (SegmentKind::Time, v, "time.digital"))
                    .ok_or(IssueCategory::InvalidExpression),
            );
        }
        let mut labels = 0;
        let mut body = s.trim();
        for marker in ["₹", "INR"] {
            if let Some(rest) = body.strip_prefix(marker) {
                labels += 1;
                body = rest.trim_start();
            }
            if let Some(rest) = body.strip_suffix(marker) {
                labels += 1;
                body = rest.trim_end();
            }
        }
        if labels > 0 {
            if labels != 1 {
                return invalid();
            }
            return Some(
                Number::parse(body, false)
                    .and_then(|n| n.money())
                    .map(|v| (SegmentKind::Money, v, "money.inr"))
                    .ok_or(IssueCategory::InvalidExpression),
            );
        }
        if s.contains('%') {
            let Some(body) = s.strip_prefix('%').or_else(|| s.strip_suffix('%')) else {
                return invalid();
            };
            let body = body.trim();
            return Some(
                Number::parse(body, false)
                    .map(|n| {
                        (
                            SegmentKind::Percent,
                            format!("{} ପ୍ରତିଶତ", n.render(false)),
                            "percent",
                        )
                    })
                    .ok_or(IssueCategory::InvalidExpression),
            );
        }
        let numeric_end = s
            .char_indices()
            .find(|(_, c)| !c.is_ascii_digit() && !matches!(c, '+' | '-' | '.' | ','))
            .map_or(s.len(), |(i, _)| i);
        if numeric_end > 0 && numeric_end < s.len() {
            let unit = s[numeric_end..].trim();
            if let Some(name) = vocabulary::unit(unit) {
                return Some(
                    Number::parse(&s[..numeric_end], false)
                        .map(|n| {
                            (
                                SegmentKind::Unit,
                                format!("{} {}", n.render(false), name),
                                "quantity.unit",
                            )
                        })
                        .ok_or(IssueCategory::InvalidExpression),
                );
            }
            return Some(Err(IssueCategory::Unsupported));
        }
        if let Some(number) = Number::parse(s, true) {
            return Some(Ok((
                if number.padded {
                    SegmentKind::Digits
                } else if number.fraction.is_empty() {
                    SegmentKind::Cardinal
                } else {
                    SegmentKind::Decimal
                },
                number.render(false),
                "number",
            )));
        }
        return Some(Err(if s.starts_with(|c: char| c.is_alphabetic()) {
            IssueCategory::ProtectedIdentifier
        } else {
            IssueCategory::InvalidExpression
        }));
    }
    if currency(s) {
        return invalid();
    }
    if policy == AmbiguityPolicy::Fallback
        && s.chars()
            .any(|c| !c.is_alphanumeric() && !prose(c) && !is_mark_or_joiner(c))
    {
        return Some(Err(IssueCategory::Unsupported));
    }
    None
}

fn date(s: &str) -> Option<String> {
    let separator = if s.contains('/') { '/' } else { '-' };
    let parts: Vec<_> = s.split(separator).collect();
    if parts.len() != 3 || parts.iter().any(|p| !p.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    let (day, month, year) =
        if separator == '-' && parts[0].len() == 4 && parts[1].len() == 2 && parts[2].len() == 2 {
            (parts[2], parts[1], parts[0])
        } else if parts[0].len() == 2 && parts[1].len() == 2 && parts[2].len() == 4 {
            (parts[0], parts[1], parts[2])
        } else {
            return None;
        };
    let day: u64 = day.parse().ok()?;
    let month: usize = month.parse().ok()?;
    let year: u64 = year.parse().ok()?;
    if year == 0 || !(1..=12).contains(&month) {
        return None;
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if day == 0 || day > days {
        return None;
    }
    Some(format!(
        "{} {} {}",
        numerals::cardinal(day),
        vocabulary::MONTHS[month - 1],
        numerals::cardinal(year)
    ))
}

fn time(s: &str) -> Option<String> {
    let (hour, minute) = s.split_once(':')?;
    if hour.len() != 2
        || minute.len() != 2
        || !hour
            .bytes()
            .chain(minute.bytes())
            .all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let hour: u64 = hour.parse().ok()?;
    let minute: u64 = minute.parse().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some(format!(
        "{} ଘଣ୍ଟା {} ମିନିଟ",
        numerals::cardinal(hour),
        numerals::cardinal(minute)
    ))
}

fn prose(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '.' | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | '।'
                | '॥'
                | '\''
                | '’'
                | '‘'
                | '"'
                | '“'
                | '”'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '…'
        )
}
fn is_mark_or_joiner(c: char) -> bool {
    matches!(c,'\u{200c}'|'\u{200d}'|'\u{0300}'..='\u{036f}'|'\u{0b01}'..='\u{0b03}'|'\u{0b3c}'..='\u{0b57}'|'\u{0b62}'..='\u{0b63}')
}

fn literal(
    text: &str,
    control: &WorkControl,
) -> Result<(String, FallbackStrategy), NormalizeError> {
    let mut result = String::new();
    let mut coded = false;
    let mut plain = false;
    for grapheme in text.graphemes(true) {
        control.check()?;
        if grapheme.chars().all(char::is_whitespace) {
            result.push_str(grapheme);
            plain = false;
        } else if grapheme
            .chars()
            .all(|c| c.is_alphabetic() || is_mark_or_joiner(c))
        {
            if !plain && !result.is_empty() && !result.ends_with(char::is_whitespace) {
                result.push(' ');
            }
            result.push_str(grapheme);
            plain = true;
        } else {
            plain = false;
            for ch in grapheme.chars() {
                control.check()?;
                let part = if let Some(d) = numerals::digit(ch) {
                    vocabulary::SMALL[d as usize].to_owned()
                } else if let Some(name) = vocabulary::symbol(ch) {
                    name.to_owned()
                } else {
                    coded = true;
                    let hex = format!("{:X}", ch as u32);
                    format!(
                        "ୟୁନିକୋଡ୍ {}",
                        hex.chars()
                            .map(|c| match c {
                                'A' => "ଏ",
                                'B' => "ବି",
                                'C' => "ସି",
                                'D' => "ଡି",
                                'E' => "ଇ",
                                'F' => "ଏଫ୍",
                                _ => vocabulary::SMALL[(c as u8 - b'0') as usize],
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                };
                if !result.is_empty() && !result.ends_with(char::is_whitespace) {
                    result.push(' ');
                }
                result.push_str(&part);
                if result.len() > MAX_RESULT_BYTES / 2 {
                    return Err(NormalizeError::LimitExceeded(LimitKind::Result));
                }
            }
        }
        if result.len() > MAX_RESULT_BYTES / 2 {
            return Err(NormalizeError::LimitExceeded(LimitKind::Result));
        }
    }
    Ok((
        result,
        if coded {
            FallbackStrategy::UnicodeCodePoint
        } else {
            FallbackStrategy::Literal
        },
    ))
}

#[cfg(test)]
mod budget_tests {
    use super::*;

    #[test]
    fn logical_result_budget_exact_boundary() {
        let mut output = Output::new();
        let remaining = MAX_RESULT_BYTES - output.bytes;
        output.charge(remaining - 1).unwrap();
        output.charge(1).unwrap();
        assert_eq!(
            output.charge(1),
            Err(NormalizeError::LimitExceeded(LimitKind::Result))
        );
    }

    #[test]
    fn work_counter_boundary_without_an_earlier_output_limit() {
        let mut output = Output::new();
        // Synthetic zero-output records isolate the work bound; real speech can
        // reach the independent output budget before this many records.
        for i in 0..MAX_CANDIDATES {
            output
                .push(
                    SourceRange::new(i, i + 1),
                    SegmentKind::Cardinal,
                    String::new(),
                    "test.counter",
                )
                .unwrap();
        }
        assert_eq!(
            output.push(
                SourceRange::new(MAX_CANDIDATES, MAX_CANDIDATES + 1),
                SegmentKind::Cardinal,
                String::new(),
                "test.counter"
            ),
            Err(NormalizeError::LimitExceeded(LimitKind::Candidates))
        );
    }
}
