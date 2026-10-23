//! Original-source boundaries and whole-expression ownership, before rendering.
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
use unicode_segmentation::UnicodeSegmentation;

use super::{
    AmbiguityPolicy, NormalizeError, SourceRange, WorkControl, electronic, has_digit, label,
    numerals, read,
};

#[derive(Clone, Copy)]
pub(super) struct Atom {
    pub(super) range: SourceRange,
    electronic: bool,
    wrapper_error: bool,
    numeric_end: usize,
}

pub(super) struct Claim {
    pub(super) range: SourceRange,
    pub(super) next: usize,
    pub(super) protected: bool,
    pub(super) invalid: bool,
}

fn whitespace(g: &str) -> bool {
    g.chars().all(char::is_whitespace)
}
fn horizontal(gap: &str) -> bool {
    !gap.is_empty()
        && gap
            .chars()
            .all(|c| c.is_whitespace() && !matches!(c, '\r' | '\n' | '\u{2028}' | '\u{2029}'))
}

fn wrapper(g: &str) -> bool {
    matches!(
        g,
        "(" | ")" | "[" | "]" | "{" | "}" | "\"" | "'" | "“" | "”" | "‘" | "’"
    )
}
fn closer(g: &str) -> Option<&'static str> {
    Some(match g {
        "(" => ")",
        "[" => "]",
        "{" => "}",
        "“" => "”",
        "‘" => "’",
        "\"" => "\"",
        "'" => "'",
        _ => return None,
    })
}
fn external(g: &str) -> bool {
    whitespace(g) || wrapper(g) || matches!(g, "," | ";" | "." | "!" | "?" | "।" | "॥")
}

fn phone_cue(s: &str) -> bool {
    matches!(
        s.trim_end_matches(':').to_ascii_lowercase().as_str(),
        "phone" | "tel" | "mobile" | "ଫୋନ" | "ମୋବାଇଲ"
    )
}

// Trimming here only recognizes a malformed label/number's ownership. The full
// source, including unmatched wrappers/marks, still goes through strict parsing.
fn label_like(s: &str) -> bool {
    let s = s.trim_matches(|c: char| "()[]{}\"'“”‘’".contains(c));
    let base: String = s
        .nfd()
        .filter(|c| !is_combining_mark(*c) && !super::is_mark_or_joiner(*c))
        .collect();
    label(&base)
        || base.starts_with("INR")
        || matches!(base.to_ascii_lowercase().as_str(), "g" | "m" | "l" | "°c")
        || (!base.is_empty() && base.chars().all(|c| matches!(c, '%' | '₹')))
}
fn money(s: &str) -> bool {
    let s = s.trim_matches(|c: char| "()[]{}\"'“”‘’".contains(c));
    s.starts_with(['₹', '$', '€', '£'])
        || s.ends_with(['₹', '$', '€', '£'])
        || ["INR", "USD", "EUR", "GBP"]
            .iter()
            .any(|m| s.starts_with(m) || s.ends_with(m))
}
fn numeric(s: &str) -> bool {
    let s = s.trim_matches(|c: char| "()[]{}\"'“”‘’".contains(c));
    has_digit(s)
        && s.chars()
            .all(|c| numerals::digit(c).is_some() || matches!(c, '+' | '-' | '.' | ','))
}

pub(super) fn scan(text: &str, control: &WorkControl) -> Result<Vec<Atom>, NormalizeError> {
    let gs: Vec<_> = text.grapheme_indices(true).collect();
    let mut protected = vec![false; gs.len()];
    // Electronic source owns punctuation in its paths/queries before prose splitting.
    let mut i = 0;
    while i < gs.len() {
        control.check()?;
        if whitespace(gs[i].1) {
            i += 1;
            continue;
        }
        let start = i;
        while i < gs.len() && !whitespace(gs[i].1) {
            i += 1;
        }
        let end = gs.get(i).map_or(text.len(), |(p, _)| *p);
        let raw = &text[gs[start].0..end];
        let core = raw.trim_matches(|c: char| "()[]{}\"'“”‘’.,;!?।॥".contains(c));
        if electronic(core) {
            protected[start..i].fill(true);
        }
    }
    // Certify global wrapper pairs once, including pairs across whitespace.
    let mut transparent = vec![false; gs.len()];
    let mut stack: Vec<(usize, &str)> = Vec::new();
    for (i, (_, g)) in gs.iter().enumerate() {
        control.check()?;
        if protected[i] {
            continue;
        }
        if let Some(&(open, close)) = stack.last()
            && *g == close
        {
            stack.pop();
            if gs.get(i + 1).is_none_or(|(_, g)| external(g)) {
                transparent[open] = true;
                transparent[i] = true;
            }
        } else if let Some(close) = closer(g)
            && (i == 0 || external(gs[i - 1].1))
        {
            stack.push((i, close));
        }
    }
    let mut result = Vec::new();
    let mut start = None;
    let mut opaque_comma = false;
    for (i, &(offset, g)) in gs.iter().enumerate() {
        control.check()?;
        let mut boundary = whitespace(g) || transparent[i];
        if !protected[i] {
            boundary |= matches!(g, ";" | "।" | "॥" | "!" | "?");
            if g == "," {
                let prev = if i > 0 { gs[i - 1].1 } else { "" };
                let next = gs.get(i + 1).map_or("", |(_, g)| *g);
                let digit_g = |g: &str| {
                    g.chars().count() == 1
                        && g.chars()
                            .next()
                            .is_some_and(|c| numerals::digit(c).is_some())
                };
                if prev == "," || next == "," {
                    // A repeated grouping separator is malformed numeric source,
                    // never permission to release the preceding valid prefix.
                    opaque_comma = true;
                } else if !(digit_g(prev) && digit_g(next)) {
                    if next.is_empty()
                        || whitespace(next)
                        || (i + 1 < gs.len() && transparent[i + 1])
                    {
                        boundary = true;
                    } else if !opaque_comma {
                        let left = &text[start.unwrap_or(offset)..offset];
                        boundary = next.starts_with(['₹', '%', '$', '€', '£'])
                            || label_like(left)
                            || read(left, AmbiguityPolicy::Preserve).is_some_and(|r| r.is_ok());
                        // Do not repeatedly parse growing malformed prefixes.
                        opaque_comma = !boundary;
                    }
                }
            }
        } else {
            boundary = false;
        }
        if boundary {
            if let Some(begin) = start.take() {
                push(text, begin, offset, &mut result);
            }
            opaque_comma = false;
        } else {
            start.get_or_insert(offset);
            // Compact phone cues separate the label from its numeric source.
            if g == ":"
                && phone_cue(&text[start.unwrap()..offset])
                && gs
                    .get(i + 1)
                    .is_some_and(|(_, g)| g.starts_with('+') || has_digit(g))
            {
                push(text, start.take().unwrap(), offset + g.len(), &mut result);
            }
        }
    }
    if let Some(start) = start {
        push(text, start, text.len(), &mut result);
    }
    for i in (0..result.len()).rev() {
        result[i].numeric_end = i;
        if numeric(&text[result[i].range.start..result[i].range.end])
            && i + 1 < result.len()
            && numeric(&text[result[i + 1].range.start..result[i + 1].range.end])
            && horizontal(&text[result[i].range.end..result[i + 1].range.start])
        {
            result[i].numeric_end = result[i + 1].numeric_end;
        }
    }
    Ok(result)
}

fn push(text: &str, start: usize, mut end: usize, out: &mut Vec<Atom>) {
    if start == end {
        return;
    }
    // One terminal full stop is sentence punctuation, including after a wrapper.
    let mut gs = text[start..end].graphemes(true).rev();
    if gs.next() == Some(".") && gs.next().is_some_and(|g| g != ".") {
        end -= 1;
    }
    if start < end {
        let body = &text[start..end];
        let core = body.trim_matches(|c: char| "()[]{}\"'“”‘’.,;!?।॥".contains(c));
        out.push(Atom {
            range: SourceRange::new(start, end),
            electronic: electronic(core),
            wrapper_error: body.graphemes(true).any(wrapper),
            numeric_end: 0,
        });
    }
}

fn piece<'a>(text: &'a str, atoms: &[Atom], i: usize) -> &'a str {
    &text[atoms[i].range.start..atoms[i].range.end]
}
fn phone_piece(s: &str) -> bool {
    has_digit(s)
        && s.chars()
            .all(|c| numerals::digit(c).is_some() || matches!(c, '+' | '-' | '(' | ')' | '.'))
}
fn digits(s: &str) -> usize {
    s.chars().filter(|c| numerals::digit(*c).is_some()).count()
}

fn phone_end(text: &str, atoms: &[Atom], i: usize) -> Option<usize> {
    let first = piece(text, atoms, i);
    let ascii = numerals::ascii(first);
    let country = ascii.strip_prefix("+91").is_some_and(|tail| {
        tail.is_empty() || tail.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '-' | '('))
    });
    let cue = if i > 1 && piece(text, atoms, i - 1) == ":" {
        Some(i - 2)
    } else {
        i.checked_sub(1)
    };
    let cued = cue.is_some_and(|cue| {
        phone_cue(piece(text, atoms, cue))
            && text[atoms[cue].range.end..atoms[i].range.start]
                .chars()
                .all(|c| c.is_whitespace() || ":([{\"'“‘".contains(c))
    });
    if !phone_piece(first) || !(country || cued) {
        return None;
    }
    // A following explicit quantity label is stronger than a generic phone cue.
    if let Some(neighbor) = atoms.get(i + 1)
        && horizontal(&text[atoms[i].range.end..neighbor.range.start])
        && label_like(piece(text, atoms, i + 1))
    {
        return None;
    }
    let target = if country { 12 } else { 10 };
    let mut count = digits(first);
    let mut next = i + 1;
    while count < target && next < atoms.len() && phone_piece(piece(text, atoms, next)) {
        let gap = &text[atoms[next - 1].range.end..atoms[next].range.start];
        if gap.is_empty()
            || !gap
                .chars()
                .all(|c| c.is_whitespace() || matches!(c, '(' | ')' | '-'))
        {
            break;
        }
        count += digits(piece(text, atoms, next));
        next += 1;
    }
    Some(next)
}

pub(super) fn claim(text: &str, atoms: &[Atom], i: usize) -> Claim {
    let first = atoms[i];
    let mut claim = Claim {
        range: first.range,
        next: i + 1,
        protected: first.electronic,
        invalid: false,
    };
    if first.electronic {
        return claim;
    }
    if let Some(next) = phone_end(text, atoms, i) {
        claim.next = next;
        claim.range.end = atoms[next - 1].range.end;
        claim.protected = true;
        // Phone-local parentheses must stay with the phone, including closers
        // certified by the prose scanner. An outer wrapper remains outside.
        let owned = &text[claim.range.start..claim.range.end];
        let mut balance = owned
            .chars()
            .filter(|c| *c == '(')
            .count()
            .saturating_sub(owned.chars().filter(|c| *c == ')').count());
        let original_end = claim.range.end;
        for (offset, g) in text[original_end..].grapheme_indices(true) {
            if balance == 0 || !(whitespace(g) || g == ")") {
                break;
            }
            if g == ")" {
                balance -= 1;
                claim.range.end = original_end + offset + g.len();
            }
        }
        return claim;
    }
    let mut has_number = has_digit(piece(text, atoms, i));
    let mut monetary = money(piece(text, atoms, i));
    let mut started = has_number || label_like(piece(text, atoms, i));
    let mut wrapper_error = first.wrapper_error;
    // An unmatched standalone opener must not release an adjacent quantity.
    if !started && piece(text, atoms, i).graphemes(true).all(wrapper) {
        started = true;
    }
    // Indexing numeric runs makes suffix-currency ownership linear, even for
    // long ordinary lists of numbers with no currency at all.
    if numeric(piece(text, atoms, i)) {
        let end = first.numeric_end;
        if let Some(next) = atoms.get(end + 1)
            && horizontal(&text[atoms[end].range.end..next.range.start])
            && money(piece(text, atoms, end + 1))
            && label_like(piece(text, atoms, end + 1))
        {
            claim.next = end + 2;
            claim.range.end = next.range.end;
            monetary = true;
            wrapper_error |= atoms[i..claim.next].iter().any(|a| a.wrapper_error);
        }
    }
    if started {
        while claim.next < atoms.len() {
            let next = claim.next;
            let atom = atoms[next];
            let gap = &text[claim.range.end..atom.range.start];
            if !horizontal(gap) || atom.electronic {
                break;
            }
            let s = piece(text, atoms, next);
            let next_number = numeric(s);
            let tail_label = label_like(s);
            let dangling_wrapper = s.graphemes(true).all(wrapper);
            if !(tail_label
                || (next_number && (!has_number || monetary))
                || (dangling_wrapper && has_number))
            {
                break;
            }
            if next_number && monetary && has_number {
                let end = atom.numeric_end;
                let independently_labeled = atoms.get(end + 1).is_some_and(|a| {
                    horizontal(&text[atoms[end].range.end..a.range.start])
                        && label_like(piece(text, atoms, end + 1))
                        && !money(piece(text, atoms, end + 1))
                });
                if independently_labeled && end == next {
                    break;
                }
            }
            claim.range.end = atom.range.end;
            claim.next += 1;
            monetary |= money(s);
            has_number |= has_digit(s);
            wrapper_error |= atom.wrapper_error;
        }
    }
    claim.invalid = has_number
        && (wrapper_error
            || text[claim.range.start..claim.range.end]
                .chars()
                .any(|c| c.is_whitespace() && !matches!(c, ' ' | '\u{a0}' | '\u{202f}')));
    claim
}
