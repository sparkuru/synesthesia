use std::{ffi::OsString, path::PathBuf};
use synesthesia_core::config::{Config, DisplayMode, Role, RuntimeRoot};
use synesthesia_core::domain::*;
use synesthesia_core::session::*;
use synesthesia_core::{ErrorCode, MAX_EVENTS, MAX_FRAME_SAMPLES};

fn input() -> InputId {
    InputId {
        session_id: 1,
        input_generation: 1,
    }
}
fn source(id: u64, revision: u64, phase: Phase, text: &str) -> SourceEvent {
    SourceEvent {
        id: UtteranceId {
            input: input(),
            utterance_id: id,
        },
        revision,
        phase,
        text: text.into(),
        start_sample: id * 100,
        end_sample: id * 100 + 50,
    }
}
fn translation(event: &SourceEvent, text: &str) -> TranslationEvent {
    TranslationEvent {
        id: event.id,
        source_revision: event.revision,
        outcome: TranslationResult::Complete { text: text.into() },
    }
}
#[test]
fn drafts_replace_and_final_enqueues_exactly_once() {
    let mut reducer = SubtitleReducer::new(input(), 3).unwrap();
    assert_eq!(
        reducer.source(source(1, 1, Phase::Draft, "first")).unwrap(),
        Update::SourcePublished { translate: None }
    );
    reducer
        .source(source(1, 2, Phase::Draft, "replacement"))
        .unwrap();
    assert_eq!(reducer.history().len(), 1);
    assert_eq!(reducer.history()[0].source.text, "replacement");
    let final_event = source(1, 3, Phase::Final, "confirmed");
    assert_eq!(
        reducer.source(final_event.clone()).unwrap(),
        Update::SourcePublished {
            translate: Some(final_event.clone())
        }
    );
    assert_eq!(
        reducer.source(final_event.clone()).unwrap(),
        Update::Ignored
    );
    assert_eq!(
        reducer.source(source(1, 99, Phase::Draft, "late")).unwrap(),
        Update::Ignored
    );
    assert_eq!(reducer.history()[0].source.text, "confirmed");
    assert_eq!(
        reducer
            .translation(translation(&final_event, "译文"))
            .unwrap(),
        Update::TranslationPublished
    );
    assert_eq!(
        reducer
            .translation(translation(&final_event, "duplicate"))
            .unwrap(),
        Update::Ignored
    );
}
#[test]
fn same_revision_cannot_change_content_but_can_finalize_exact_draft() {
    let mut reducer = SubtitleReducer::new(input(), 3).unwrap();
    reducer.source(source(1, 1, Phase::Draft, "text")).unwrap();
    assert_eq!(
        reducer.source(source(1, 1, Phase::Final, "different")),
        Err(ErrorCode::InvalidEvent)
    );
    assert!(matches!(
        reducer.source(source(1, 1, Phase::Final, "text")).unwrap(),
        Update::SourcePublished { translate: Some(_) }
    ));
    assert_eq!(
        reducer
            .source(source(1, 2, Phase::Final, "different"))
            .unwrap(),
        Update::Ignored
    );
}
#[test]
fn stale_translation_and_evicted_ids_never_reappear() {
    let mut reducer = SubtitleReducer::new(input(), 3).unwrap();
    let old = source(1, 1, Phase::Final, "old");
    reducer.source(old.clone()).unwrap();
    let mut wrong_revision = translation(&old, "stale");
    wrong_revision.source_revision = 2;
    assert_eq!(
        reducer.translation(wrong_revision).unwrap(),
        Update::Ignored
    );
    for id in 2..1000 {
        reducer.source(source(id, 1, Phase::Final, "new")).unwrap();
    }
    assert_eq!(reducer.history().len(), 3);
    assert_eq!(reducer.history()[0].source.id.utterance_id, 997);
    assert_eq!(
        reducer.translation(translation(&old, "late")).unwrap(),
        Update::Ignored
    );
    assert_eq!(reducer.source(old).unwrap(), Update::Ignored);
}
#[test]
fn reset_isolates_generations_sessions_and_clears_text() {
    let mut reducer = SubtitleReducer::new(input(), 3).unwrap();
    let old = source(1, 1, Phase::Final, "old");
    reducer.source(old.clone()).unwrap();
    assert!(reducer.reset(input()).is_err());
    reducer
        .reset(InputId {
            input_generation: 2,
            ..input()
        })
        .unwrap();
    assert!(reducer.history().is_empty());
    assert_eq!(reducer.source(old.clone()).unwrap(), Update::Ignored);
    assert_eq!(
        reducer.translation(translation(&old, "late")).unwrap(),
        Update::Ignored
    );
    reducer
        .reset(InputId {
            session_id: 2,
            input_generation: 1,
        })
        .unwrap();
    assert!(reducer.reset(input()).is_err());
}
#[test]
fn malformed_text_ranges_ids_and_revision_overflow_are_rejected() {
    let mut reducer = SubtitleReducer::new(input(), 3).unwrap();
    for invalid in [
        source(0, 1, Phase::Draft, "text"),
        source(1, 0, Phase::Draft, "text"),
        source(1, 1, Phase::Draft, " "),
        source(1, 1, Phase::Draft, "\0"),
    ] {
        assert_eq!(reducer.source(invalid), Err(ErrorCode::InvalidEvent));
    }
    let mut reversed = source(1, 1, Phase::Draft, "text");
    reversed.end_sample = 0;
    assert_eq!(reducer.source(reversed), Err(ErrorCode::InvalidEvent));
    let mut ids = IdSequence::new(u64::MAX - 1);
    assert_eq!(ids.allocate().unwrap(), u64::MAX);
    assert_eq!(ids.allocate(), Err(ErrorCode::Overflow));
    assert_eq!(ids.allocate(), Err(ErrorCode::Overflow));
}

#[derive(Default)]
struct FakeCompute {
    loads: usize,
    owned: bool,
    cancelled: Vec<InputId>,
    audio: usize,
}
impl ComputeAdapter for FakeCompute {
    fn detect(&mut self) -> Result<(), ErrorCode> {
        Ok(())
    }
    fn load(&mut self) -> Result<(), ErrorCode> {
        self.loads += 1;
        self.owned = true;
        Ok(())
    }
    fn accept_audio(&mut self, audio: &AudioChunk) -> Result<(), ErrorCode> {
        assert!(self.owned);
        self.audio += audio.pcm.len();
        Ok(())
    }
    fn cancel(&mut self, input: InputId) -> Result<(), ErrorCode> {
        self.cancelled.push(input);
        Ok(())
    }
    fn translate(&mut self, source: &SourceEvent) -> Result<(), ErrorCode> {
        assert!(self.owned);
        assert_eq!(source.phase, Phase::Final);
        Ok(())
    }
    fn next_event(&mut self) -> Result<Option<PublishedEvent>, ErrorCode> {
        Ok(None)
    }
    fn shutdown(&mut self) -> Result<(), ErrorCode> {
        self.owned = false;
        Ok(())
    }
}
fn ready() -> (Session, FakeCompute) {
    let mut session = Session::new(0, 3).unwrap();
    let mut fake = FakeCompute::default();
    assert!(session.start().is_err());
    assert_eq!(session.initialize().unwrap(), Effect::Detect);
    fake.detect().unwrap();
    assert_eq!(session.detected().unwrap(), Effect::Load);
    fake.load().unwrap();
    assert_eq!(session.compute_state(), ComputeState::Loading);
    assert!(session.start().is_err());
    session.loaded().unwrap();
    (session, fake)
}
fn running() -> (Session, FakeCompute, InputId) {
    let (mut session, fake) = ready();
    let Effect::StartCapture(input) = session.start().unwrap() else {
        panic!("expected capture effect");
    };
    session.capture_started(input).unwrap();
    (session, fake, input)
}
fn cleanup(fake: &mut FakeCompute, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::CancelInput(id) => fake.cancel(id).unwrap(),
            Effect::Shutdown => fake.shutdown().unwrap(),
            _ => {}
        }
    }
}
#[test]
fn stop_cancels_tasks_reuses_model_and_exit_releases_owned_engine() {
    let (mut session, mut fake, active) = running();
    session.push_audio(audio(active, 0, 0, vec![1, 2])).unwrap();
    fake.accept_audio(&session.pop_audio().unwrap()).unwrap();
    assert!(session.pop_audio().is_none());
    session.asr_completed(active, 0).unwrap();
    let event = source(1, 1, Phase::Final, "source");
    session.source(event.clone()).unwrap();
    session.pop_translation().unwrap();
    cleanup(&mut fake, session.stop().unwrap());
    assert_eq!(session.pending(), (0, 0, 0));
    assert!(session.subtitles().is_none());
    assert!(fake.owned);
    assert_eq!(fake.cancelled, vec![active]);
    assert_eq!(
        session.translation(translation(&event, "late")).unwrap(),
        Update::Ignored
    );
    session.stopped().unwrap();
    let Effect::StartCapture(next) = session.start().unwrap() else {
        panic!();
    };
    assert_ne!(active.session_id, next.session_id);
    assert_eq!(fake.loads, 1);
    session.capture_started(next).unwrap();
    cleanup(&mut fake, session.shutdown());
    session.released().unwrap();
    assert!(!fake.owned);
    assert_eq!(session.compute_state(), ComputeState::Released);
    assert!(session.start().is_err());
}
#[test]
fn switch_input_invalidates_audio_and_old_results_without_reload() {
    let (mut session, fake, active) = running();
    session
        .push_audio(audio(active, 0, 0, vec![1; 10]))
        .unwrap();
    let event = source(1, 1, Phase::Final, "old");
    session.source(event.clone()).unwrap();
    let effects = session.switch_input().unwrap();
    let next = InputId {
        input_generation: 2,
        ..active
    };
    assert_eq!(
        effects,
        vec![
            Effect::StopCapture,
            Effect::CancelInput(active),
            Effect::ClearQueues,
            Effect::StartCapture(next)
        ]
    );
    assert_eq!(session.pending(), (0, 0, 0));
    assert!(
        session
            .push_audio(audio(active, 10, 32_000, vec![1]))
            .is_err()
    );
    session.capture_started(next).unwrap();
    assert_eq!(session.source(event).unwrap(), Update::Ignored);
    assert_eq!(fake.loads, 1);
}
#[test]
fn audio_overload_fails_session_instead_of_skipping_middle_audio() {
    let (mut session, mut fake, active) = running();
    for seq in 0..10 {
        session
            .push_audio(audio(
                active,
                seq,
                seq * MAX_FRAME_SAMPLES as u64,
                vec![1; MAX_FRAME_SAMPLES as usize],
            ))
            .unwrap();
    }
    assert_eq!(
        session.push_audio(audio(active, 10, 32_000, vec![1])),
        Err(ErrorCode::Overloaded)
    );
    assert_eq!(session.state(), SessionState::Failed);
    assert_eq!(session.pending(), (0, 0, 0));
    cleanup(&mut fake, session.take_cleanup());
    assert_eq!(fake.cancelled, vec![active]);
    assert!(fake.owned);
    assert!(
        session
            .push_audio(audio(active, 10, 32_000, vec![1]))
            .is_err()
    );
}
#[test]
fn translation_budget_counts_inflight_and_source_publish_precedes_translation() {
    let (mut session, _, _) = running();
    for id in 1..=4 {
        let event = source(id, 1, Phase::Final, "text");
        session.source(event.clone()).unwrap();
        assert!(matches!(
            session.pop_event(),
            Some(PublishedEvent::Source(_))
        ));
        assert_eq!(session.pop_translation().unwrap().id, event.id);
    }
    assert_eq!(session.pending().1, 4);
    assert_eq!(
        session.source(source(5, 1, Phase::Final, "text")),
        Err(ErrorCode::Overloaded)
    );
    assert_eq!(session.state(), SessionState::Failed);
    assert_eq!(session.pending(), (0, 0, 0));
}
#[test]
fn event_budget_and_failed_translation_do_not_create_unbounded_state() {
    let (mut session, _, _) = running();
    let event = source(1, 1, Phase::Final, "keep source");
    session.source(event.clone()).unwrap();
    session.pop_translation().unwrap();
    session
        .translation(TranslationEvent {
            id: event.id,
            source_revision: 1,
            outcome: TranslationResult::Failed {
                code: ErrorCode::InferenceFailed,
            },
        })
        .unwrap();
    assert_eq!(
        session.subtitles().unwrap().history()[0].source.text,
        "keep source"
    );
    assert_eq!(session.pending().1, 0);
    for revision in 1..=(MAX_EVENTS as u64 - 2) {
        session
            .source(source(2, revision, Phase::Draft, "draft"))
            .unwrap();
    }
    assert_eq!(
        session.source(source(2, MAX_EVENTS as u64, Phase::Draft, "draft")),
        Err(ErrorCode::Overloaded)
    );
    assert_eq!(session.state(), SessionState::Failed);
}
#[test]
fn queue_rejects_weight_overflow_without_mutating_previous_items() {
    let mut queue = BoundedQueue::new(2, u64::MAX).unwrap();
    queue.push("first", u64::MAX).unwrap();
    assert_eq!(queue.push("second", 1), Err(ErrorCode::Overflow));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.pop(), Some("first"));
    assert_eq!(queue.weight(), 0);
}
fn client() -> serde_json::Value {
    serde_json::json!({"role":"client","schema_version":1,"source":"en","target":"zh-Hans", "capture":{"mode":"follow_default"},
        "connection":{"address":"ws://127.0.0.1:8765/","token":"synthetic-secret-for-tests"}})
}
#[test]
fn strict_role_language_schema_and_unknown_fields_are_rejected_safely() {
    let value = client();
    let bytes = serde_json::to_vec(&value).unwrap();
    let config = Config::parse(&bytes, Role::Client).unwrap();
    let mut invalid_source = value.clone();
    invalid_source["source"] = serde_json::json!("auto");
    let error =
        Config::parse(&serde_json::to_vec(&invalid_source).unwrap(), Role::Client).unwrap_err();
    assert_eq!(error.field, "source");
    assert!(error.line > 0 && error.column > 0);
    assert!(config.inference().is_none());
    if let Config::Client { overlay, .. } = config {
        assert_eq!(overlay.history, 3);
        assert_eq!(overlay.mode, DisplayMode::Bilingual);
    }
    assert_eq!(
        Config::parse(&bytes, Role::Full).unwrap_err().code,
        ErrorCode::WrongRole
    );
    for (field, invalid) in [
        ("source", serde_json::json!("auto")),
        ("source", serde_json::json!("synthetic-secret-for-tests")),
        ("target", serde_json::json!("en")),
        ("inference", serde_json::json!({"model":"x"})),
        ("synthetic-secret-for-tests", serde_json::json!(1)),
    ] {
        let mut modified = value.clone();
        modified[field] = invalid;
        let error =
            Config::parse(&serde_json::to_vec(&modified).unwrap(), Role::Client).unwrap_err();
        assert!(!format!("{error:?} {error}").contains("synthetic-secret-for-tests"));
    }
    let mut modified = value.clone();
    modified["schema_version"] = serde_json::json!(2);
    assert_eq!(
        Config::parse(&serde_json::to_vec(&modified).unwrap(), Role::Client)
            .unwrap_err()
            .code,
        ErrorCode::UnsupportedVersion
    );
    let debug = format!("{:?}", Config::parse(&bytes, Role::Client).unwrap());
    assert!(!debug.contains("synthetic-secret-for-tests"));
    let mut trailing = bytes;
    trailing.extend_from_slice(b" {}");
    assert!(Config::parse(&trailing, Role::Client).is_err());
}
#[test]
fn token_header_injection_and_url_credentials_are_rejected() {
    for address in [
        "ws://127.0.0.1:8765/?token=secret",
        "ws://user:secret@localhost/",
        "ws://@localhost/",
        "wss://localhost/",
        "ws://localhost/#secret",
        "ws://localhost:0/",
    ] {
        let mut value = client();
        value["connection"]["address"] = serde_json::json!(address);
        assert!(Config::parse(&serde_json::to_vec(&value).unwrap(), Role::Client).is_err());
    }
    let mut value = client();
    value["connection"]["token"] = serde_json::json!("token-long-enough\r\nBad: header");
    assert_eq!(
        Config::parse(&serde_json::to_vec(&value).unwrap(), Role::Client)
            .unwrap_err()
            .code,
        ErrorCode::InvalidToken
    );
}

#[test]
fn nested_configuration_fields_remain_strict() {
    let value = client();
    let json = serde_json::to_string(&value).unwrap();
    for bad in [
        json.replace(
            "\"mode\":\"follow_default\"",
            "\"mode\":\"follow_default\",\"mode\":\"device\"",
        ),
        json.replace(
            "\"mode\":\"follow_default\"",
            "\"mode\":\"follow_default\",\"device_id\":\"unexpected\"",
        ),
        json.replace(
            "\"token\":\"synthetic-secret-for-tests\"",
            "\"token\":\"synthetic-secret-for-tests\",\"token\":\"another-synthetic-secret\"",
        ),
    ] {
        assert!(Config::parse(bad.as_bytes(), Role::Client).is_err());
    }
    let full = r#"{"role":"full","schema_version":1,"source":"en","target":"zh-Hans","capture":{"mode":"follow_default"},"inference":{"asr_model":"asr.gguf","translation_model":"translation.gguf","device":{"backend":"cpu"}}}"#;
    Config::parse(full.as_bytes(), Role::Full).unwrap();
    for device in [
        r#"{"backend":"cpu","device_id":1}"#,
        r#"{"backend":"cpu","backend":"cpu"}"#,
        r#"{"backend":"cpu","unexpected":null}"#,
    ] {
        let invalid = full.replace(r#"{"backend":"cpu"}"#, device);
        assert!(Config::parse(invalid.as_bytes(), Role::Full).is_err());
    }
}
#[test]
fn paths_resolve_from_runtime_root_not_cwd_and_external_models_are_allowed() {
    let root = RuntimeRoot::new(std::env::temp_dir().join("synesthesia-root")).unwrap();
    assert_eq!(
        root.resolve(std::path::Path::new("models/model.gguf"))
            .unwrap(),
        root.path().join("models/model.gguf")
    );
    let external = std::env::temp_dir().join("external models/model.gguf");
    assert_eq!(root.resolve(&external).unwrap(), external);
    assert!(root.resolve(std::path::Path::new("../escape")).is_err());
    assert!(RuntimeRoot::new(PathBuf::from("relative")).is_err());
    #[cfg(windows)]
    {
        assert_eq!(
            root.resolve(std::path::Path::new(r"E:\tmp\models\model.gguf"))
                .unwrap(),
            PathBuf::from(r"E:\tmp\models\model.gguf")
        );
        assert!(
            root.resolve(std::path::Path::new(r"E:relative.gguf"))
                .is_err()
        );
    }
}
#[test]
fn help_version_and_unsupported_runtime_are_truthful_and_safe() {
    let exe = std::env::temp_dir().join("synesthesia.exe");
    for role in [Role::Full, Role::Server, Role::Client] {
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(
            synesthesia_core::cli::run(
                role,
                [OsString::from("--help")],
                exe.clone(),
                &mut out,
                &mut err
            ),
            0
        );
        assert!(String::from_utf8(out).unwrap().contains("not integrated"));
        let mut out = Vec::new();
        assert_eq!(
            synesthesia_core::cli::run(
                role,
                [OsString::from("--version")],
                exe.clone(),
                &mut out,
                &mut err
            ),
            0
        );
        assert!(String::from_utf8(out).unwrap().contains("0.1.0"));
        let mut out = Vec::new();
        assert_eq!(
            synesthesia_core::cli::run(
                role,
                [
                    OsString::from("--run"),
                    OsString::from("synthetic-secret-for-tests")
                ],
                exe.clone(),
                &mut out,
                &mut err
            ),
            2
        );
        assert!(
            !String::from_utf8(err)
                .unwrap()
                .contains("synthetic-secret-for-tests")
        );
    }
}

#[test]
fn publication_queue_carries_each_original_payload_not_current_snapshot() {
    let (mut session, _, _) = running();
    let draft_one = source(1, 1, Phase::Draft, "first draft");
    let draft_two = source(1, 2, Phase::Draft, "second draft");
    let final_event = source(1, 3, Phase::Final, "final");
    session.source(draft_one.clone()).unwrap();
    session.source(draft_two.clone()).unwrap();
    session.source(final_event.clone()).unwrap();
    assert_eq!(
        session.subtitles().unwrap().history()[0].source,
        final_event
    );
    assert_eq!(session.pop_event(), Some(PublishedEvent::Source(draft_one)));
    assert_eq!(session.pop_event(), Some(PublishedEvent::Source(draft_two)));
    assert_eq!(
        session.pop_event(),
        Some(PublishedEvent::Source(final_event.clone()))
    );
    session.pop_translation().unwrap();
    let translation = translation(&final_event, "译文");
    session.translation(translation.clone()).unwrap();
    assert_eq!(
        session.pop_event(),
        Some(PublishedEvent::Translation(translation))
    );
}
#[test]
fn exhausted_session_and_generation_ids_fail_without_reuse() {
    let mut session = Session::new(u64::MAX, 3).unwrap();
    session.initialize().unwrap();
    session.detected().unwrap();
    session.loaded().unwrap();
    assert_eq!(session.start(), Err(ErrorCode::Overflow));
    assert_eq!(session.state(), SessionState::Idle);
    assert_eq!(session.input(), None);
    assert_eq!(
        InputId {
            input_generation: u64::MAX,
            ..input()
        }
        .next_generation(),
        Err(ErrorCode::Overflow)
    );
}
#[test]
fn full_and_server_config_keep_native_model_paths_and_role_boundaries() {
    let full = serde_json::json!({"role":"full","schema_version":1,"source":"ja","target":"zh-Hans", "capture":{"mode":"follow_default"},
        "inference":{"asr_model":"models/asr.gguf","translation_model":"models/translation.gguf","device":{"backend":"vulkan","vendor_id":4098,"device_id":30032,"device_name":"AMD Radeon RX 9070 XT"}}});
    Config::parse(&serde_json::to_vec(&full).unwrap(), Role::Full).unwrap();
    let mut server = full.clone();
    let fields = server.as_object_mut().unwrap();
    fields.remove("capture");
    fields.insert("role".into(), serde_json::json!("server"));
    fields.insert("listen".into(), serde_json::json!("127.0.0.1:8765"));
    fields.insert(
        "token".into(),
        serde_json::json!("synthetic-secret-for-tests"),
    );
    let config = Config::parse(&serde_json::to_vec(&server).unwrap(), Role::Server).unwrap();
    assert!(!format!("{config:?}").contains("synthetic-secret-for-tests"));
    server["capture"] = serde_json::json!({"mode":"follow_default"});
    assert!(Config::parse(&serde_json::to_vec(&server).unwrap(), Role::Server).is_err());
    let mut full = full;
    full["listen"] = serde_json::json!("0.0.0.0:8765");
    assert!(Config::parse(&serde_json::to_vec(&full).unwrap(), Role::Full).is_err());
}

#[test]
fn load_and_cli_validate_relative_config_and_resolve_models_before_use() {
    let path = std::env::temp_dir().join(format!("synesthesia-config-test-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let result = std::panic::catch_unwind(|| {
        let config = serde_json::json!({"role":"full","schema_version":1,"source":"en","target":"zh-Hans","capture":{"mode":"follow_default"},
            "inference":{"asr_model":"models/asr.gguf","translation_model":"models/translation.gguf","device":{"backend":"cpu"}}});
        std::fs::write(
            path.join("settings.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        let root = RuntimeRoot::new(path.clone()).unwrap();
        let resolved = root
            .load(std::path::Path::new("settings.json"), Role::Full)
            .unwrap();
        assert_eq!(
            resolved.inference().unwrap().asr_model,
            path.join("models/asr.gguf")
        );
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(
            synesthesia_core::cli::run(
                Role::Full,
                [
                    OsString::from("--check-config"),
                    OsString::from("settings.json")
                ],
                path.join("synesthesia-full.exe"),
                &mut out,
                &mut err
            ),
            0
        );
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("runtime readiness are unverified")
        );
        assert!(err.is_empty());
        let mut oversized = std::fs::File::create(path.join("oversized.json")).unwrap();
        use std::io::Write;
        oversized
            .write_all(&vec![b' '; synesthesia_core::config::MAX_CONFIG_BYTES + 1])
            .unwrap();
        assert!(
            root.load(std::path::Path::new("oversized.json"), Role::Full)
                .is_err()
        );
    });
    std::fs::remove_dir_all(&path).unwrap();
    result.unwrap();
}

fn audio(input: InputId, seq: u64, start_sample: u64, pcm: Vec<i16>) -> AudioChunk {
    AudioChunk {
        input,
        seq,
        start_sample,
        pcm,
    }
}
#[test]
fn audio_queue_preserves_timeline_and_a_gap_ends_local_session() {
    let (mut session, _, active) = running();
    let first = audio(active, 0, 0, vec![1, 2, 3]);
    let next = audio(active, 1, 3, vec![4, 5]);
    session.push_audio(first.clone()).unwrap();
    session.push_audio(next.clone()).unwrap();
    assert_eq!(session.pop_audio(), Some(first));
    session.asr_completed(active, 0).unwrap();
    assert_eq!(session.pop_audio(), Some(next));
    session.asr_completed(active, 1).unwrap();
    assert_eq!(
        session.push_audio(audio(active, 3, 5, vec![6])),
        Err(ErrorCode::Discontinuity)
    );
    assert_eq!(session.state(), SessionState::Failed);
    assert_eq!(session.pending(), (0, 0, 0));
    assert!(
        session
            .take_cleanup()
            .contains(&Effect::CancelInput(active))
    );
}

#[test]
fn asr_completion_releases_only_the_matching_frame() {
    let (mut session, _, active) = running();
    for seq in 0..3 {
        session
            .push_audio(audio(active, seq, seq, vec![1]))
            .unwrap();
    }
    assert_eq!(session.pop_audio().unwrap().seq, 0);
    assert_eq!(
        session.asr_completed(active, 1),
        Err(ErrorCode::InvalidState)
    );
    assert!(session.pop_audio().is_none());
    session.asr_completed(active, 0).unwrap();
    assert_eq!(session.pop_audio().unwrap().seq, 1);
    // A duplicate completion for frame zero cannot free frame one.
    assert_eq!(
        session.asr_completed(active, 0),
        Err(ErrorCode::InvalidState)
    );
    assert!(session.pop_audio().is_none());
    session.asr_completed(active, 1).unwrap();
    assert_eq!(session.pop_audio().unwrap().seq, 2);
    let effects = session.switch_input().unwrap();
    let next = session.input().unwrap();
    assert!(effects.contains(&Effect::CancelInput(active)));
    session.capture_started(next).unwrap();
    session.push_audio(audio(next, 0, 0, vec![1])).unwrap();
    session.pop_audio().unwrap();
    assert_eq!(
        session.asr_completed(active, 2),
        Err(ErrorCode::InvalidState)
    );
    session.asr_completed(next, 0).unwrap();
}

#[test]
fn config_duplicate_null_and_unbounded_history_fields_are_invalid() {
    let value = client();
    let json = serde_json::to_string(&value).unwrap();
    for bad in [
        json.replace("\"source\":\"en\"", "\"source\":\"en\",\"source\":\"ja\""),
        json.replace(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
        ),
    ] {
        assert!(Config::parse(bad.as_bytes(), Role::Client).is_err());
    }
    for (field, invalid) in [
        ("inference", serde_json::Value::Null),
        ("overlay", serde_json::json!({"history":21})),
        ("overlay", serde_json::json!({"history":0})),
        (
            "overlay",
            serde_json::json!({"history":3,"raw_transcript":true}),
        ),
    ] {
        let mut bad = value.clone();
        bad[field] = invalid;
        assert!(Config::parse(&serde_json::to_vec(&bad).unwrap(), Role::Client).is_err());
    }
}
