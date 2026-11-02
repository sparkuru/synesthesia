//! Borrowed display projections of the authoritative core subtitle state.

use synesthesia_core::{
    ErrorCode,
    config::DisplayMode,
    domain::{Phase, Subtitle, TranslationResult},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TranslationState {
    Draft,
    Pending,
    Complete,
    Failed(ErrorCode),
}

/// Content-bearing views deliberately omit Debug to avoid logging captions.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Caption<'a> {
    pub source: Option<&'a str>,
    pub translation: Option<&'a str>,
    pub phase: Phase,
    pub translation_state: TranslationState,
}

/// Select visible text without scheduling work or retaining another history.
///
/// The current core Subtitle does not distinguish disabled translation from a
/// pending final. Callers must handle that pipeline setting separately.
pub fn project(subtitle: &Subtitle, mode: DisplayMode) -> Caption<'_> {
    let (translation, translation_state) = match subtitle.source.phase {
        Phase::Draft => (None, TranslationState::Draft),
        Phase::Final => match &subtitle.translation {
            Some(TranslationResult::Complete { text }) => {
                (Some(text.as_str()), TranslationState::Complete)
            }
            Some(TranslationResult::Failed { code }) => (None, TranslationState::Failed(*code)),
            None => (None, TranslationState::Pending),
        },
    };

    Caption {
        source: match mode {
            DisplayMode::Bilingual | DisplayMode::SourceOnly => Some(&subtitle.source.text),
            DisplayMode::TranslationOnly => None,
        },
        translation: match mode {
            DisplayMode::Bilingual | DisplayMode::TranslationOnly => translation,
            DisplayMode::SourceOnly => None,
        },
        phase: subtitle.source.phase,
        translation_state,
    }
}
