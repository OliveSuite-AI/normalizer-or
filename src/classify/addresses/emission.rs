//! Atomic field readings and source-partitioning constituent emission.
use super::{fields::Field, lexicon::Label};
use crate::{
    FallbackClass, IssueCategory, LimitKind, MAX_CANDIDATES, NormalizeError, SegmentKind,
    SourceRange,
    classify::Candidate,
    domain::{
        composite::{Component, ComponentValue, Composite},
        numeric::Numeric,
    },
    interpretation::{Reading, UnresolvedFinding, Value},
};

pub(super) fn text_component(
    range: SourceRange,
    kind: SegmentKind,
    text: impl Into<String>,
) -> Component {
    Component {
        range,
        value: ComponentValue::Text {
            kind,
            rule_id: if kind == SegmentKind::Verbatim {
                "source.verbatim"
            } else {
                "address.field"
            },
            text: text.into(),
        },
    }
}

fn number_component(range: SourceRange, value: Numeric) -> Component {
    Component {
        range,
        value: ComponentValue::Numeric(value),
    }
}

pub(super) fn resolved(range: SourceRange, components: Vec<Component>) -> Candidate {
    Candidate {
        range,
        reading: Reading::Resolved(Value::Composite(Composite { components })),
    }
}

pub(super) fn push(
    candidates: &mut Vec<Candidate>,
    candidate: Candidate,
) -> Result<(), NormalizeError> {
    if candidates.len() == MAX_CANDIDATES {
        return Err(NormalizeError::LimitExceeded(LimitKind::Candidates));
    }
    candidates.push(candidate);
    Ok(())
}

pub(super) fn emit(
    text: &str,
    field: Field,
    apartment: bool,
    candidates: &mut Vec<Candidate>,
) -> Result<(), NormalizeError> {
    let range = field.range();
    let mut components = Vec::new();
    match field {
        Field::Alias { kind, .. } => {
            components.push(text_component(
                range,
                SegmentKind::Abbreviation,
                kind.spoken(),
            ));
        }
        Field::Street {
            number,
            alias,
            value: Some(value),
        } => {
            components.push(number_component(number, value));
            if number.end < alias.start {
                components.push(text_component(
                    SourceRange::new(number.end, alias.start),
                    SegmentKind::Verbatim,
                    &text[number.end..alias.start],
                ));
            }
            components.push(text_component(alias, SegmentKind::Abbreviation, "sokak"));
        }
        Field::Labeled {
            header,
            body: Some(body),
            postal_valid: true,
            ..
        } if header.kind == Label::Postcode => {
            components.push(text_component(
                SourceRange::new(range.start, body.range.start),
                SegmentKind::Abbreviation,
                header.kind.spoken(),
            ));
            components.push(text_component(
                body.range,
                SegmentKind::Digits,
                crate::numerals::digits(&text[body.range.start..body.range.end]),
            ));
        }
        Field::Labeled {
            header,
            body: Some(body),
            values: Some((first, second)),
            ..
        } => {
            let infer_apartment = header.kind == Label::Number && second.is_some() && !apartment;
            components.push(text_component(
                SourceRange::new(range.start, body.first.start),
                SegmentKind::Abbreviation,
                if infer_apartment {
                    "bina numarası "
                } else {
                    header.kind.spoken()
                },
            ));
            components.push(number_component(body.first, first));
            if let (Some(second_range), Some(second)) = (body.second, second) {
                components.push(text_component(
                    SourceRange::new(body.first.end, second_range.start),
                    SegmentKind::Symbol,
                    if infer_apartment {
                        ", daire numarası "
                    } else {
                        " bölü "
                    },
                ));
                components.push(number_component(second_range, second));
            }
        }
        _ => {
            return push(
                candidates,
                Candidate {
                    range,
                    reading: Reading::Unresolved(UnresolvedFinding::unresolved(
                        &text[range.start..range.end],
                        FallbackClass::Number,
                        IssueCategory::InvalidExpression,
                    )),
                },
            );
        }
    }
    push(candidates, resolved(range, components))
}
