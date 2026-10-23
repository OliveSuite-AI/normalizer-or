//! Minimal local Odia speech and cleanup example.
use normalizer_or::{CleanOptions, DigitStyle, NormalizeOptions, Normalizer};
fn main() -> Result<(), normalizer_or::NormalizeError> {
    let n = Normalizer::new()?;
    println!(
        "{}",
        n.normalize("ଆଜି ୨୫ kg ଚାଉଳ।", &NormalizeOptions::default())?
            .normalized_text()
    );
    println!(
        "{}",
        n.clean(
            "English 123",
            &CleanOptions {
                digits: DigitStyle::Odia,
                ..Default::default()
            }
        )?
        .normalized_text()
    );
    Ok(())
}
