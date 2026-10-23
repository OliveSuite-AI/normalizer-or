//! Caller intent and resolution options.
use super::SourceRange;

/// How unresolved linguistic expressions are handled.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AmbiguityPolicy {
    /// Preserve source spans and return structured issues.
    #[default]
    Preserve,
    /// Return an error containing unresolved diagnostics, without partial output.
    Reject,
    /// Render unresolved source faithfully and report handled fallback diagnostics.
    Fallback,
}

/// Explicit interpretation of a whole original-source span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HintKind {
    /// An exact integer, including valid grouping and explicit zero-padding intent.
    Cardinal,
    /// ASCII/Odia digits, optional initial plus, and space/dot/slash/hyphen/parentheses.
    Digits,
    /// A valid numeric Gregorian date.
    Date,
    /// A valid 24-hour clock.
    Time,
}

/// Caller intent at original grapheme-safe byte coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hint {
    pub(crate) range: SourceRange,
    pub(crate) kind: HintKind,
}

impl Hint {
    /// Create a hint; normalization validates coordinates, overlap, and content.
    pub const fn new(range: SourceRange, kind: HintKind) -> Self {
        Self { range, kind }
    }
    /// Original-source range.
    pub const fn range(self) -> SourceRange {
        self.range
    }
    /// Requested interpretation.
    pub const fn kind(self) -> HintKind {
        self.kind
    }
}

/// Per-call options. Source-faithful fallback is opt-in.
#[derive(Clone, Debug, Default)]
pub struct NormalizeOptions {
    /// Preserve by default; explicitly reject or apply source-faithful fallback.
    pub ambiguity_policy: AmbiguityPolicy,
    /// Non-overlapping, whole-expression hints in original-source coordinates.
    pub hints: Vec<Hint>,
}

/// Optional written digit conversion; never expands digits into words.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DigitStyle {
    /// Keep each digit's original script.
    #[default]
    Preserve,
    /// Convert ASCII/Odia decimal digits to ASCII.
    Ascii,
    /// Convert ASCII/Odia decimal digits to Odia.
    Odia,
}

/// Written-text whitespace handling.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WhitespacePolicy {
    /// Keep whitespace exactly, including line endings.
    #[default]
    Preserve,
    /// Replace each whitespace run with one ordinary space; do not trim.
    Collapse,
}

/// Conservative cleanup. NFC is always applied; spelling and joiners are retained.
#[derive(Clone, Debug, Default)]
pub struct CleanOptions {
    /// Optional decimal digit conversion.
    pub digits: DigitStyle,
    /// Optional whitespace-run collapse.
    pub whitespace: WhitespacePolicy,
}
