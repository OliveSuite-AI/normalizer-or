//! Test-only JSONL interface for installed-wheel conformance and stress checks.
#[path = "mod.rs"]
#[allow(dead_code)]
mod support;
use normalizer_or::*;
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

fn main() {
    let n = Normalizer::new().unwrap();
    let stdout = io::stdout();
    let mut output = io::BufWriter::new(stdout.lock());
    for line in io::stdin().lock().lines() {
        let request: Value =
            serde_json::from_str(&line.unwrap()).expect("valid UTF-8 JSON request");
        let control = WorkControl::default();
        if request["cancelled"] == true {
            control.cancel();
        }
        let input = request["input"].as_str().unwrap();
        let result = if request["operation"] == "clean" {
            let digits = match request["digits"].as_str().unwrap_or("preserve") {
                "preserve" => DigitStyle::Preserve,
                "ascii" => DigitStyle::Ascii,
                "odia" => DigitStyle::Odia,
                _ => panic!("invalid harness digit style"),
            };
            let whitespace = match request["whitespace"].as_str().unwrap_or("preserve") {
                "preserve" => WhitespacePolicy::Preserve,
                "collapse" => WhitespacePolicy::Collapse,
                _ => panic!("invalid harness whitespace"),
            };
            n.clean_controlled(input, &CleanOptions { digits, whitespace }, &control)
        } else {
            let ambiguity_policy = match request["policy"].as_str().unwrap_or("preserve") {
                "preserve" => AmbiguityPolicy::Preserve,
                "reject" => AmbiguityPolicy::Reject,
                "fallback" => AmbiguityPolicy::Fallback,
                _ => panic!("invalid harness policy"),
            };
            let hints = request["hints"].as_array().map_or_else(Vec::new, |hs| {
                hs.iter()
                    .map(|h| {
                        let kind = match h["kind"].as_str().unwrap() {
                            "digits" => HintKind::Digits,
                            "cardinal" => HintKind::Cardinal,
                            "date" => HintKind::Date,
                            "time" => HintKind::Time,
                            _ => panic!("invalid harness hint kind"),
                        };
                        Hint::new(
                            SourceRange::new(
                                h["start"].as_u64().unwrap() as usize,
                                h["end"].as_u64().unwrap() as usize,
                            ),
                            kind,
                        )
                    })
                    .collect()
            });
            n.normalize_controlled(
                input,
                &NormalizeOptions {
                    ambiguity_policy,
                    hints,
                },
                &control,
            )
        };
        support::partition(input, &result);
        writeln!(
            output,
            "{}",
            json!({"id":request["id"],"result":support::canonical(result)})
        )
        .unwrap();
    }
}
