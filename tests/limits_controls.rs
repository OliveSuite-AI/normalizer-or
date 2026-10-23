//! Boundary engineering checks independent of linguistic goldens.
use normalizer_or::*;
use std::sync::{Arc, Barrier};

#[test]
fn exact_input_and_hint_limits() {
    let n = Normalizer::new().unwrap();
    for size in [MAX_INPUT_BYTES - 1, MAX_INPUT_BYTES, MAX_INPUT_BYTES + 1] {
        for input in [
            "a".repeat(size),
            "ଅ".repeat(size / 3) + &"a".repeat(size % 3),
        ] {
            assert_eq!(input.len(), size);
            for result in [
                n.normalize(&input, &Default::default()),
                n.clean(&input, &Default::default()),
            ] {
                if size <= MAX_INPUT_BYTES {
                    assert_eq!(result.unwrap().normalized_text(), input);
                } else {
                    assert_eq!(result, Err(NormalizeError::LimitExceeded(LimitKind::Input)));
                }
            }
        }
    }

    for count in [MAX_HINTS - 1, MAX_HINTS, MAX_HINTS + 1] {
        let input = "1;".repeat(count);
        let hints = (0..count)
            .map(|i| Hint::new(SourceRange::new(2 * i, 2 * i + 1), HintKind::Digits))
            .collect();
        let result = n.normalize(
            &input,
            &NormalizeOptions {
                hints,
                ..Default::default()
            },
        );
        if count <= MAX_HINTS {
            assert!(result.unwrap().complete());
        } else {
            assert_eq!(result, Err(NormalizeError::LimitExceeded(LimitKind::Hints)));
        }
    }
    assert_eq!(
        n.normalize(
            &"🙂".repeat(4096),
            &NormalizeOptions {
                ambiguity_policy: AmbiguityPolicy::Fallback,
                ..Default::default()
            }
        ),
        Err(NormalizeError::LimitExceeded(LimitKind::Result))
    );
    // Result accounting may preempt the candidate count on public input.
    assert!(matches!(
        n.clean(
            &"1a".repeat(MAX_CANDIDATES + 1),
            &CleanOptions {
                digits: DigitStyle::Odia,
                ..Default::default()
            }
        ),
        Err(NormalizeError::LimitExceeded(
            LimitKind::Result | LimitKind::Candidates
        ))
    ));
}

#[test]
fn hint_overlap_reordering_graphemes_and_extreme_offsets() {
    let n = Normalizer::new().unwrap();
    let hint = |a, b| Hint::new(SourceRange::new(a, b), HintKind::Digits);
    let result = n
        .normalize(
            "1;2",
            &NormalizeOptions {
                hints: vec![hint(2, 3), hint(0, 1)],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(result.normalized_text(), "ଏକ;ଦୁଇ");
    for hints in [
        vec![hint(0, 1), hint(0, 1)],
        vec![hint(0, 3), hint(2, 3)],
        vec![hint(1, 1)],
        vec![hint(3, 1)],
        vec![hint(0, usize::MAX)],
    ] {
        assert_eq!(
            n.normalize(
                "1;2",
                &NormalizeOptions {
                    hints,
                    ..Default::default()
                }
            ),
            Err(NormalizeError::InvalidHint)
        );
    }
    let input = "e\u{301} କ୍\u{200d}ଷ ୧୨";
    let start = input.find('୧').unwrap();
    for offset in start + 1..input.len() {
        if offset == start + 3 {
            continue;
        } // whole second digit is a different partial-expression rejection.
        assert_eq!(
            n.normalize(
                input,
                &NormalizeOptions {
                    hints: vec![hint(start, offset)],
                    ..Default::default()
                }
            ),
            Err(NormalizeError::InvalidHint)
        );
    }
}

#[test]
fn concurrent_reuse_cancellation_and_deadlines() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<Normalizer>();
    let n = Normalizer::new().unwrap();
    let expected = n.normalize("୨୫ kg", &Default::default()).unwrap();
    let control = WorkControl::default();
    let barrier = Arc::new(Barrier::new(9));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let n = n.clone();
            let control = control.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                n.normalize_controlled("୨୫ kg", &Default::default(), &control)
            })
        })
        .collect();
    barrier.wait();
    control.cancel();
    for handle in handles {
        match handle.join().unwrap() {
            Ok(result) => assert_eq!(result, expected),
            Err(error) => assert_eq!(error, NormalizeError::Cancelled),
        }
    }
    for _ in 0..3 {
        assert_eq!(
            n.normalize_controlled("5", &Default::default(), &control.clone()),
            Err(NormalizeError::Cancelled)
        );
        assert_eq!(
            n.clean_controlled("5", &Default::default(), &control.clone()),
            Err(NormalizeError::Cancelled)
        );
    }
    let expired = WorkControl::new(Some(std::time::Instant::now()));
    assert_eq!(
        n.clean_controlled("5", &Default::default(), &expired),
        Err(NormalizeError::Cancelled)
    );
    assert_eq!(n.normalize("୨୫ kg", &Default::default()).unwrap(), expected);
}
