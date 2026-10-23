//! Untuned Odia workload baseline; measurements are host-specific.
use normalizer_or::{CleanOptions, NormalizeOptions, Normalizer};
use std::{hint::black_box, time::Instant};
fn main() {
    let normalizer = Normalizer::new().unwrap();
    let inputs = [
        "ଆଜି ୨୫ kg ଚାଉଳ",
        "₹1,23,456.50",
        "2026-10-23 09:30",
        "English ଓଡ଼ିଆ ୧୨୩",
    ];
    let start = Instant::now();
    for _ in 0..1000 {
        for input in inputs {
            black_box(
                normalizer
                    .normalize(black_box(input), &NormalizeOptions::default())
                    .unwrap(),
            );
            black_box(
                normalizer
                    .clean(black_box(input), &CleanOptions::default())
                    .unwrap(),
            );
        }
    }
    println!("8000 Odia operations: {:?}", start.elapsed());
}
