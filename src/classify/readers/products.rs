use super::{Context, scan};
use crate::{
    FallbackClass, IssueCategory, LimitKind, MAX_CANDIDATES, NormalizeError, SegmentKind,
    SourceRange, WorkControl,
    domain::{
        composite::{Component, ComponentValue, Composite},
        identifiers::Telephone,
        numeric::Numeric,
        quantities::Quantity,
    },
    interpretation::{Reading, UnresolvedFinding, Value},
    notation,
};

fn operator(text: &str) -> bool {
    matches!(text, "x" | "*" | "×")
}

fn numeric_piece(text: &str) -> bool {
    text.chars().any(|ch| ch.is_ascii_digit())
        || notation::label(text)
        || notation::unsupported_label(text)
}

/// Product punctuation is scanned locally; electronic interiors are never split.
pub(super) fn read(
    ctx: &Context<'_>,
    index: usize,
    control: &WorkControl,
) -> Result<Option<(Reading, usize)>, NormalizeError> {
    let first = ctx.tokens[index];
    if !(numeric_piece(first.text) || operator(first.text))
        || crate::domain::electronic::looks_like(first.text)
    {
        return Ok(None);
    }
    let mut operand_end = ctx.number_end(index);
    if (notation::label(first.text) || first.text.starts_with(['₺', '$', '€', '£']))
        && let Some(number) = ctx.tokens.get(index + 1)
        && numeric_piece(number.text)
    {
        operand_end = ctx.number_end(index + 1);
    }
    if let Some(next) = ctx.tokens.get(operand_end + 1)
        && (notation::label(next.text) || notation::attached_money_tail(next.text).is_some())
    {
        operand_end += 1;
    }
    let explicit = operator(first.text)
        || first.text.contains(['*', '×'])
        || ctx
            .tokens
            .get(operand_end)
            .is_some_and(|last| last.text.contains(['*', '×']));
    if !explicit
        && !ctx
            .tokens
            .get(operand_end + 1)
            .is_some_and(|next| operator(next.text) || next.text.starts_with(['*', '×']))
    {
        return Ok(None);
    }
    let mut end = index;
    let mut found = operator(first.text) || first.text.contains(['*', '×']);
    let mut unsupported = false;
    let mut expecting_operand = operator(first.text) || first.text.ends_with(['*', '×']);
    while let Some(next) = ctx.tokens.get(end + 1) {
        control.check()?;
        if !scan::whitespace_between(ctx.text, ctx.tokens[end].range.end, next.range.start) {
            break;
        }
        if scan::math_operator(next.text) {
            found = true;
            unsupported |= !operator(next.text);
            end += 1;
            expecting_operand = true;
        } else if numeric_piece(next.text) || next.text.contains(['*', '×']) || expecting_operand {
            end += 1;
            found |= next.text.contains(['*', '×']);
            expecting_operand = next.text.ends_with(['*', '×']);
        } else {
            break;
        }
    }
    if !found {
        return Ok(None);
    }
    let owner = SourceRange::new(first.range.start, ctx.tokens[end].range.end);
    let input = &ctx.text[owner.start..owner.end];
    if unsupported {
        return Ok(Some((
            Reading::Unresolved(UnresolvedFinding::unresolved(
                input,
                FallbackClass::Expression,
                IssueCategory::Unsupported,
            )),
            end,
        )));
    }
    let mut operators = Vec::new();
    for (offset, ch) in input.char_indices() {
        control.check()?;
        if matches!(ch, '*' | '×')
            || (ch == 'x'
                && (offset == 0
                    || input[..offset]
                        .chars()
                        .next_back()
                        .is_some_and(char::is_whitespace))
                && input[offset + 1..]
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace))
        {
            operators.push(SourceRange::new(
                owner.start + offset,
                owner.start + offset + ch.len_utf8(),
            ));
        }
    }
    if operators.is_empty() {
        return Ok(None);
    }
    if operators.len() > (MAX_CANDIDATES - 1) / 2 {
        return Err(NormalizeError::LimitExceeded(LimitKind::Candidates));
    }
    let failure = |category| {
        (
            Reading::Unresolved(UnresolvedFinding::unresolved(
                input,
                FallbackClass::Expression,
                category,
            )),
            end,
        )
    };
    let mut components = Vec::new();
    let mut quantities = 0;
    let mut counts = true;
    let mut start = owner.start;
    for boundary in operators
        .iter()
        .copied()
        .chain(std::iter::once(SourceRange::new(owner.end, owner.end)))
    {
        control.check()?;
        let raw = &ctx.text[start..boundary.start];
        let body = raw.trim();
        let body = if boundary.start == owner.end {
            body.strip_suffix('.')
                .filter(|before| !before.ends_with('.'))
                .unwrap_or(body)
        } else {
            body
        };
        if body.is_empty() {
            return Ok(Some(failure(IssueCategory::InvalidExpression)));
        }
        if (body.starts_with("+90") || body.starts_with('0'))
            && Telephone::parse(body, false).is_some()
        {
            return Ok(Some(failure(IssueCategory::Unsupported)));
        }
        let leading = raw.len() - raw.trim_start().len();
        let part = SourceRange::new(start + leading, start + leading + body.len());
        if part.start != start {
            components.push(gap(ctx.text, start, part.start));
        }
        let value = if quantity_shaped(body) {
            match Quantity::parse_operand(body) {
                Ok(quantity) => {
                    quantities += 1;
                    ComponentValue::Quantity(quantity)
                }
                Err(category) => {
                    return Ok(Some(failure(if arithmetic_continuation(body) {
                        IssueCategory::Unsupported
                    } else {
                        category
                    })));
                }
            }
        } else {
            let Some(number) = Numeric::product_operand(body) else {
                return Ok(Some(failure(if arithmetic_continuation(body) {
                    IssueCategory::Unsupported
                } else {
                    IssueCategory::InvalidExpression
                })));
            };
            counts &= number.is_count();
            ComponentValue::Numeric(number)
        };
        components.push(Component { range: part, value });
        if part.end != boundary.start {
            components.push(gap(ctx.text, part.end, boundary.start));
        }
        if boundary.start != owner.end {
            components.push(Component {
                range: boundary,
                value: ComponentValue::Text {
                    kind: SegmentKind::Symbol,
                    rule_id: "expression.multiplication",
                    text: "çarpı".to_owned(),
                },
            });
        }
        start = boundary.end;
    }
    if quantities > 1 || (quantities == 1 && !counts) {
        return Ok(Some(failure(IssueCategory::Unsupported)));
    }
    // Compact symbols need spoken spacing without changing their source ownership.
    for index in 0..components.len() {
        if matches!(
            components[index].value,
            ComponentValue::Text {
                kind: SegmentKind::Symbol,
                ..
            }
        ) {
            let leading = index > 0
                && components[index - 1].range.end == components[index].range.start
                && !matches!(
                    components[index - 1].value,
                    ComponentValue::Text {
                        kind: SegmentKind::Verbatim,
                        ..
                    }
                );
            let trailing = components.get(index + 1).is_some_and(|next| {
                !matches!(
                    next.value,
                    ComponentValue::Text {
                        kind: SegmentKind::Verbatim,
                        ..
                    }
                )
            });
            if let ComponentValue::Text { text, .. } = &mut components[index].value {
                if leading {
                    text.insert(0, ' ');
                }
                if trailing {
                    text.push(' ');
                }
            }
        }
    }
    Ok(Some((
        Reading::Resolved(Value::Composite(Composite { components })),
        end,
    )))
}

fn gap(text: &str, start: usize, end: usize) -> Component {
    Component {
        range: SourceRange::new(start, end),
        value: ComponentValue::Text {
            kind: SegmentKind::Verbatim,
            rule_id: "source.verbatim",
            text: text[start..end].to_owned(),
        },
    }
}

fn quantity_shaped(text: &str) -> bool {
    text.split_whitespace().any(notation::label)
        || notation::attached_quantity(text).is_some()
        || text.chars().any(|ch| matches!(ch, '₺' | '$' | '€' | '£'))
}

fn arithmetic_continuation(text: &str) -> bool {
    text.char_indices().any(|(offset, ch)| {
        let token = &text[offset..offset + ch.len_utf8()];
        scan::math_operator(token) && !operator(token) && (offset > 0 || !matches!(ch, '+' | '-'))
    })
}
