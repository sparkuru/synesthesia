use synesthesia_core::{
    ErrorCode,
    config::DisplayMode,
    domain::{
        InputId, Phase, SourceEvent, Subtitle, SubtitleReducer, TranslationEvent,
        TranslationResult, Update, UtteranceId,
    },
};
use synesthesia_desktop::presentation::{TranslationState, project};

fn source(phase: Phase) -> SourceEvent {
    SourceEvent {
        id: UtteranceId {
            input: InputId {
                session_id: 1,
                input_generation: 1,
            },
            utterance_id: 1,
        },
        revision: 1,
        phase,
        text: "A controlled source sentence.".into(),
        start_sample: 0,
        end_sample: 16_000,
    }
}

#[test]
fn display_modes_select_text_without_fallback() {
    let subtitle = Subtitle {
        source: source(Phase::Final),
        translation: Some(TranslationResult::Complete {
            text: "受控译文。".into(),
        }),
    };

    let bilingual = project(&subtitle, DisplayMode::Bilingual);
    assert_eq!(bilingual.source, Some("A controlled source sentence."));
    assert_eq!(bilingual.translation, Some("受控译文。"));

    let original = project(&subtitle, DisplayMode::SourceOnly);
    assert_eq!(original.source, bilingual.source);
    assert_eq!(original.translation, None);

    let translated = project(&subtitle, DisplayMode::TranslationOnly);
    assert_eq!(translated.source, None);
    assert_eq!(translated.translation, bilingual.translation);

    for mode in [
        DisplayMode::Bilingual,
        DisplayMode::SourceOnly,
        DisplayMode::TranslationOnly,
    ] {
        let caption = project(&subtitle, mode);
        assert_eq!(caption.phase, Phase::Final);
        assert!(matches!(
            caption.translation_state,
            TranslationState::Complete
        ));
    }
}

#[test]
fn draft_and_pending_final_remain_distinct_in_translation_only_mode() {
    let mut subtitle = Subtitle {
        source: source(Phase::Draft),
        translation: None,
    };

    let draft = project(&subtitle, DisplayMode::TranslationOnly);
    assert_eq!(draft.source, None);
    assert_eq!(draft.translation, None);
    assert_eq!(draft.phase, Phase::Draft);
    assert!(matches!(draft.translation_state, TranslationState::Draft));

    subtitle.source.phase = Phase::Final;
    let pending = project(&subtitle, DisplayMode::TranslationOnly);
    assert_eq!(pending.source, None);
    assert_eq!(pending.translation, None);
    assert_eq!(pending.phase, Phase::Final);
    assert!(matches!(
        pending.translation_state,
        TranslationState::Pending
    ));

    let bilingual = project(&subtitle, DisplayMode::Bilingual);
    assert_eq!(bilingual.source, Some("A controlled source sentence."));
    assert_eq!(bilingual.translation, None);
}

#[test]
fn translation_failure_preserves_original_and_safe_failure_category() {
    let subtitle = Subtitle {
        source: source(Phase::Final),
        translation: Some(TranslationResult::Failed {
            code: ErrorCode::InferenceFailed,
        }),
    };

    for mode in [DisplayMode::Bilingual, DisplayMode::SourceOnly] {
        let caption = project(&subtitle, mode);
        assert_eq!(caption.source, Some("A controlled source sentence."));
        assert_eq!(caption.translation, None);
        assert!(matches!(
            caption.translation_state,
            TranslationState::Failed(ErrorCode::InferenceFailed)
        ));
    }

    let translated = project(&subtitle, DisplayMode::TranslationOnly);
    assert_eq!(translated.source, None);
    assert_eq!(translated.translation, None);
    assert!(matches!(
        translated.translation_state,
        TranslationState::Failed(ErrorCode::InferenceFailed)
    ));
}

#[test]
fn draft_does_not_expose_a_translation_or_report_pending() {
    let subtitle = Subtitle {
        source: source(Phase::Draft),
        translation: Some(TranslationResult::Complete {
            text: "Not a translation of a final source.".into(),
        }),
    };

    let caption = project(&subtitle, DisplayMode::Bilingual);
    assert_eq!(caption.source, Some("A controlled source sentence."));
    assert_eq!(caption.translation, None);
    assert!(matches!(caption.translation_state, TranslationState::Draft));
}

#[test]
fn mode_changes_do_not_mutate_reducer_or_schedule_translation() {
    let original = source(Phase::Final);
    let mut reducer = SubtitleReducer::new(original.id.input, 3).unwrap();
    let update = reducer.source(original.clone()).unwrap();
    assert!(matches!(
        update,
        Update::SourcePublished { translate: Some(_) }
    ));
    let before = reducer.history().clone();

    for mode in [
        DisplayMode::TranslationOnly,
        DisplayMode::SourceOnly,
        DisplayMode::Bilingual,
        DisplayMode::TranslationOnly,
    ] {
        let subtitle = reducer.history().back().unwrap();
        let caption = project(subtitle, mode);
        assert!(matches!(
            caption.translation_state,
            TranslationState::Pending
        ));
    }
    assert_eq!(reducer.history(), &before);
    assert_eq!(reducer.source(original.clone()).unwrap(), Update::Ignored);

    let translation = TranslationEvent {
        id: original.id,
        source_revision: original.revision,
        outcome: TranslationResult::Complete {
            text: "受控译文。".into(),
        },
    };
    assert_eq!(
        reducer.translation(translation).unwrap(),
        Update::TranslationPublished
    );
    let subtitle = reducer.history().back().unwrap();
    let before = subtitle.clone();
    let caption = project(subtitle, DisplayMode::TranslationOnly);
    assert_eq!(caption.translation, Some("受控译文。"));
    assert_eq!(subtitle, &before);
}
