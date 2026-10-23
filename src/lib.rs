//! Offline Odia speech normalization and conservative written-text cleanup.
//! Source coordinates are original, grapheme-safe UTF-8 byte offsets.
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod api;
mod engine;
mod normalizer;
mod numerals;
mod source_map;
mod vocabulary;

pub use api::{
    AmbiguityPolicy, CleanOptions, DigitStyle, FallbackClass, FallbackDiagnostic, FallbackReason,
    FallbackStrategy, Hint, HintKind, Issue, IssueCategory, LimitKind, NormalizeError,
    NormalizeOptions, NormalizeResult, Segment, SegmentKind, SourceRange, WhitespacePolicy,
    WorkControl,
};
pub use normalizer::Normalizer;
/// Diagnostic identity of this Odia-only build.
pub const NORMALIZER_ID: &str = concat!("normalizer-or/", env!("CARGO_PKG_VERSION"));

/// Maximum original UTF-8 input length in bytes.
pub const MAX_INPUT_BYTES: usize = 32 * 1024;
/// Maximum number of caller hints.
pub const MAX_HINTS: usize = 256;
/// Maximum number of semiotic candidate records, including unresolved spans.
pub const MAX_CANDIDATES: usize = 4096;
/// Maximum logical bytes of owned result and diagnostic allocations.
pub const MAX_RESULT_BYTES: usize = 512 * 1024;

#[cfg(test)]
#[path = "../tests/support/config.rs"]
mod test_config;
