use crate::{
    CleanOptions, NORMALIZER_ID, NormalizeError, NormalizeOptions, NormalizeResult, WorkControl,
    engine,
};

/// Immutable, reusable, thread-safe Odia normalizer. No models or network services.
#[derive(Clone, Default)]
pub struct Normalizer;

impl Normalizer {
    /// Identity of this Odia build.
    pub fn normalizer_id(&self) -> &'static str {
        NORMALIZER_ID
    }
    /// Construct the built-in Odia normalizer.
    pub fn new() -> Result<Self, NormalizeError> {
        Ok(Self)
    }
    /// Expand supported notation into Odia speech text.
    pub fn normalize(
        &self,
        input: &str,
        options: &NormalizeOptions,
    ) -> Result<NormalizeResult, NormalizeError> {
        self.normalize_controlled(input, options, &WorkControl::default())
    }
    /// Speech normalization with cooperative cancellation/deadline checks.
    pub fn normalize_controlled(
        &self,
        input: &str,
        options: &NormalizeOptions,
        control: &WorkControl,
    ) -> Result<NormalizeResult, NormalizeError> {
        engine::normalize(input, options, control)
    }
    /// NFC cleanup with optional whitespace/digit conversion and original ranges.
    pub fn clean(
        &self,
        input: &str,
        options: &CleanOptions,
    ) -> Result<NormalizeResult, NormalizeError> {
        self.clean_controlled(input, options, &WorkControl::default())
    }
    /// Written cleanup with cooperative cancellation/deadline checks.
    pub fn clean_controlled(
        &self,
        input: &str,
        options: &CleanOptions,
        control: &WorkControl,
    ) -> Result<NormalizeResult, NormalizeError> {
        engine::clean(input, options, control)
    }
}
