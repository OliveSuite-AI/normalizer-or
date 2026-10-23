//! Shared test-only canonical results and fixture assertions.
use normalizer_or::*;
use serde_json::{Value, json};

pub fn label(value: impl std::fmt::Debug) -> String {
    let source = format!("{value:?}");
    let chars: Vec<_> = source.chars().collect();
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase()
            && i > 0
            && (chars[i - 1].is_lowercase() || chars.get(i + 1).is_some_and(|c| c.is_lowercase()))
        {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

pub fn issues(records: &[Issue]) -> Value {
    json!(records.iter().map(|i|json!({"range":[i.range().start(),i.range().end()],"category":label(i.category()),"explanation":i.explanation()})).collect::<Vec<_>>())
}

pub fn canonical(result: Result<NormalizeResult, NormalizeError>) -> Value {
    match result {
        Ok(r) => {
            json!({"normalized_text":r.normalized_text(),"locale":r.locale(),"normalizer_id":r.normalizer_id(),"complete":r.complete(),"fallback_used":r.fallback_used(),
            "segments":r.segments().iter().map(|s|json!({"range":[s.range().start(),s.range().end()],"kind":label(s.kind()),"text":s.text(),"rule_id":s.rule_id()})).collect::<Vec<_>>(),
            "issues":issues(r.issues()),
            "fallbacks":r.fallbacks().iter().map(|f|json!({"range":[f.range().start(),f.range().end()],"attempted_class":label(f.attempted_class()),"reason":label(f.reason()),"original_category":f.original_category().map(label),"strategy":label(f.strategy())})).collect::<Vec<_>>() })
        }
        Err(e) => {
            let (code, limit, records) = match e {
                NormalizeError::InvalidInput => ("invalid_input", None, json!([])),
                NormalizeError::InvalidHint => ("invalid_hint", None, json!([])),
                NormalizeError::InvalidConfiguration => ("invalid_configuration", None, json!([])),
                NormalizeError::Cancelled => ("cancelled", None, json!([])),
                NormalizeError::Internal => ("internal", None, json!([])),
                NormalizeError::Unresolved(ref records) => ("unresolved", None, issues(records)),
                NormalizeError::LimitExceeded(kind) => {
                    ("limit_exceeded", Some(label(kind)), json!([]))
                }
            };
            json!({"error":code,"limit_kind":limit,"issues":records})
        }
    }
}

pub fn subset(expected: &Value, actual: &Value, context: &str) {
    match expected {
        Value::Object(fields) => {
            for (key, value) in fields {
                assert!(actual.get(key).is_some(), "{context}: missing {key}");
                subset(value, &actual[key], &format!("{context}.{key}"));
            }
        }
        Value::Array(values) => {
            let actual = actual
                .as_array()
                .unwrap_or_else(|| panic!("{context}: expected array, got {actual}"));
            assert_eq!(values.len(), actual.len(), "{context}: array length");
            for (i, (a, b)) in values.iter().zip(actual).enumerate() {
                subset(a, b, &format!("{context}[{i}]"));
            }
        }
        _ => assert_eq!(expected, actual, "{context}"),
    }
}

pub fn legacy_spans(case: &Value) -> Value {
    case.get("issue_spans")
        .cloned()
        .unwrap_or_else(|| json!([[0, case["input"].as_str().unwrap().len()]]))
}

pub fn check_case(n: &Normalizer, case: &Value) {
    let input = case["input"].as_str().unwrap();
    let id = case["id"].as_str().unwrap();
    let preserved = n.normalize(input, &Default::default());
    let preserved_issues = match &preserved {
        Ok(r) => issues(r.issues()),
        Err(_) => json!([]),
    };
    for (policy, name) in [
        (AmbiguityPolicy::Preserve, "preserve"),
        (AmbiguityPolicy::Reject, "reject"),
        (AmbiguityPolicy::Fallback, "fallback"),
    ] {
        let result = canonical(n.normalize(
            input,
            &NormalizeOptions {
                ambiguity_policy: policy,
                ..Default::default()
            },
        ));
        if let Some(expected) = case.get("expect") {
            subset(&expected[name], &result, &format!("{id}/{name}"));
        } else {
            let complete = case["complete"].as_bool().unwrap_or(true);
            if !complete && policy == AmbiguityPolicy::Reject {
                subset(
                    &json!({"error":"unresolved","issues":preserved_issues}),
                    &result,
                    id,
                );
            } else if complete {
                subset(
                    &json!({"normalized_text":case["output"],"complete":true,"issues":[],"fallbacks":[]}),
                    &result,
                    id,
                );
            } else {
                let field = if policy == AmbiguityPolicy::Fallback {
                    "fallbacks"
                } else {
                    "issues"
                };
                let spans = legacy_spans(case);
                let actual: Vec<_> = result[field]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| r["range"].clone())
                    .collect();
                assert_eq!(json!(actual), spans, "{id}/{name}");
                if policy == AmbiguityPolicy::Preserve {
                    subset(
                        &json!({"normalized_text":case.get("output").cloned().unwrap_or_else(||json!(input)),"complete":false}),
                        &result,
                        id,
                    );
                } else {
                    subset(
                        &json!({"complete":true,"issues":[],"fallback_used":true}),
                        &result,
                        id,
                    );
                }
            }
        }
        if policy == AmbiguityPolicy::Reject && result.get("error").is_some() {
            assert_eq!(
                result["issues"], preserved_issues,
                "{id}: Reject/Preserve diagnostics differ"
            );
        }
    }
}

pub fn partition(input: &str, result: &Result<NormalizeResult, NormalizeError>) {
    use unicode_segmentation::UnicodeSegmentation;
    let boundaries: Vec<_> = input
        .grapheme_indices(true)
        .map(|(i, _)| i)
        .chain(std::iter::once(input.len()))
        .collect();
    let check_range = |range: SourceRange| {
        assert!(range.start() < range.end());
        assert!(boundaries.binary_search(&range.start()).is_ok());
        assert!(boundaries.binary_search(&range.end()).is_ok());
    };
    match result {
        Ok(result) => {
            let mut cursor = 0;
            for segment in result.segments() {
                check_range(segment.range());
                assert_eq!(segment.range().start(), cursor);
                cursor = segment.range().end();
                if matches!(
                    segment.kind(),
                    SegmentKind::Verbatim | SegmentKind::Unresolved
                ) {
                    assert_eq!(segment.text(), &input[segment.range().start()..cursor]);
                }
            }
            assert_eq!(cursor, input.len());
            assert_eq!(
                result.normalized_text(),
                result
                    .segments()
                    .iter()
                    .map(|s| s.text())
                    .collect::<String>()
            );
            assert_eq!(result.complete(), result.issues().is_empty());
            for issue in result.issues() {
                check_range(issue.range());
            }
            for fallback in result.fallbacks() {
                check_range(fallback.range());
            }
        }
        Err(NormalizeError::Unresolved(issues)) => {
            for issue in issues {
                check_range(issue.range());
            }
        }
        _ => {}
    }
}
