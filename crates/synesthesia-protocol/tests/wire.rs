use synesthesia_core::config::{SourceLanguage, TargetLanguage};
use synesthesia_core::domain::{InputId, Phase, PublishedEvent, SourceEvent};
use synesthesia_core::{ErrorCode, MAX_TEXT_BYTES};
use synesthesia_protocol::*;
fn input() -> InputId {
    InputId {
        session_id: 1,
        input_generation: 1,
    }
}
fn hello() -> Message {
    Message::Hello {
        source: SourceLanguage::English,
        target: TargetLanguage::SimplifiedChinese,
    }
}
fn start() -> Message {
    Message::SessionStart {
        source: SourceLanguage::English,
        target: TargetLanguage::SimplifiedChinese,
    }
}
fn golden_audio() -> Vec<u8> {
    let hex: String = include_str!("fixtures/audio.hex")
        .split_whitespace()
        .collect();
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn frame() -> AudioFrame {
    AudioFrame {
        input: input(),
        seq: 0,
        start_sample: 0,
        pcm: vec![i16::MIN, i16::MAX],
    }
}
#[test]
fn golden_json_and_pcm_contract_are_byte_stable() {
    let hello_fixture = include_str!("fixtures/hello.json").trim();
    assert_eq!(
        decode_json(hello_fixture.as_bytes()).unwrap(),
        Envelope::new(1, hello())
    );
    assert_eq!(
        encode_json(&Envelope::new(1, hello())).unwrap(),
        hello_fixture.as_bytes()
    );
    let source_fixture = include_str!("fixtures/source.json").trim();
    let event = decode_json(source_fixture.as_bytes()).unwrap();
    assert_eq!(encode_json(&event).unwrap(), source_fixture.as_bytes());
    assert_eq!(decode_audio(&golden_audio()).unwrap(), frame());
    assert_eq!(encode_audio(&frame()).unwrap(), golden_audio());
}
#[test]
fn malformed_versions_fields_lengths_and_audio_formats_are_rejected() {
    let golden = golden_audio();
    for length in 0..golden.len() {
        assert!(decode_audio(&golden[..length]).is_err());
    }
    for offset in [0, 6, 40, 44, 48, 49, 50, 51] {
        let mut bad = golden.clone();
        bad[offset] ^= 0xff;
        assert!(decode_audio(&bad).is_err(), "offset {offset}");
    }
    for offset in [8, 16] {
        let mut bad = golden.clone();
        bad[offset..offset + 8].fill(0);
        assert!(decode_audio(&bad).is_err());
    }
    let mut bad = golden.clone();
    bad[4] = 2;
    assert_eq!(
        decode_audio(&bad).unwrap_err(),
        ErrorCode::UnsupportedVersion
    );
    let mut bad = golden.clone();
    bad.extend_from_slice(&[0, 0]);
    assert!(decode_audio(&bad).is_err());
    assert!(decode_audio(&vec![0; MAX_AUDIO_BYTES + 1]).is_err());
    let mut overflow = frame();
    overflow.seq = u64::MAX;
    assert_eq!(encode_audio(&overflow), Err(ErrorCode::Overflow));
    overflow.seq = 0;
    overflow.start_sample = u64::MAX;
    assert_eq!(encode_audio(&overflow), Err(ErrorCode::Overflow));
    let json = include_str!("fixtures/hello.json");
    assert_eq!(
        decode_json(
            json.replace("\"protocol_version\":1", "\"protocol_version\":2")
                .as_bytes()
        )
        .unwrap_err(),
        ErrorCode::UnsupportedVersion
    );
    for bad in [
        json.replace("\"source\":\"en\"", "\"source\":\"auto\""),
        json.replace("\"target\":\"zh-Hans\"", "\"target\":\"en\""),
        json.replace("\"source\":\"en\"", "\"source\":\"en\",\"source\":\"ja\""),
        json.replace(
            "\"source\":\"en\"",
            "\"source\":\"en\",\"token\":\"must-not-reflect\"",
        ),
        json.replace("\"request_id\":1", "\"request_id\":0"),
    ] {
        let error = decode_json(bad.as_bytes()).unwrap_err();
        assert!(!format!("{error:?} {error}").contains("must-not-reflect"));
    }
    assert!(decode_json(&vec![b' '; MAX_JSON_BYTES + 1]).is_err());
}
#[test]
fn valid_maximum_escaped_content_survives_local_to_wire_boundary() {
    let mut envelope = decode_json(include_str!("fixtures/source.json").as_bytes()).unwrap();
    let Message::Source { event } = &mut envelope.message else {
        panic!();
    };
    event.text = format!("x{}", "\t".repeat(MAX_TEXT_BYTES - 1));
    let bytes = encode_json(&envelope).unwrap();
    assert_eq!(decode_json(&bytes).unwrap(), envelope);
    let Message::Source { event } = envelope.message else {
        panic!();
    };
    let payload = Message::from(PublishedEvent::Source(event.clone()));
    assert_eq!(payload, Message::Source { event });
}
#[test]
fn frame_order_session_generation_and_gaps_are_checked_without_advancing_on_error() {
    let mut order = AudioOrder::new(input()).unwrap();
    order.accept(&frame()).unwrap();
    assert_eq!(order.accept(&frame()), Err(ErrorCode::Discontinuity));
    let mut next = frame();
    next.seq = 1;
    next.start_sample = 2;
    let mut gap = next.clone();
    gap.seq = 2;
    assert_eq!(order.accept(&gap), Err(ErrorCode::Discontinuity));
    let mut stale = next.clone();
    stale.input.input_generation = 2;
    assert_eq!(order.accept(&stale), Err(ErrorCode::InvalidState));
    order.accept(&next).unwrap();
}
#[test]
fn connection_requires_handshake_then_started_and_stop_prevents_audio() {
    let mut guard = ConnectionGuard::authenticated();
    assert!(guard.audio_allowed(input()).is_err());
    assert!(guard.receive(&Envelope::new(1, start())).is_err());
    guard.receive(&Envelope::new(1, hello())).unwrap();
    assert!(guard.receive(&Envelope::new(1, start())).is_err());
    guard.receive(&Envelope::new(2, start())).unwrap();
    assert!(guard.audio_allowed(input()).is_err());
    guard.started(input()).unwrap();
    guard.audio_allowed(input()).unwrap();
    let next = InputId {
        input_generation: 2,
        ..input()
    };
    guard
        .receive(&Envelope::new(3, Message::InputSwitch { input: next }))
        .unwrap();
    assert!(guard.audio_allowed(input()).is_err());
    assert!(guard.audio_allowed(next).is_err());
    guard.switched(next).unwrap();
    guard.audio_allowed(next).unwrap();
    guard
        .receive(&Envelope::new(4, Message::SessionStop { input: next }))
        .unwrap();
    assert!(guard.audio_allowed(next).is_err());
    guard.stopped(next).unwrap();
    guard.receive(&Envelope::new(5, start())).unwrap();
    assert!(guard.started(input()).is_err());
    let fresh = InputId {
        session_id: 2,
        input_generation: 1,
    };
    guard.started(fresh).unwrap();
    assert!(guard.audio_allowed(input()).is_err());
    guard.close();
    assert!(guard.audio_allowed(fresh).is_err());
}
#[test]
fn random_and_mutated_inputs_never_panic_or_leak_payload_in_errors() {
    let mut state = 17u64;
    for length in 0..512 {
        let mut bytes = Vec::new();
        for _ in 0..length {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            bytes.push((state >> 32) as u8);
        }
        let _ = decode_audio(&bytes);
        let _ = decode_json(&bytes);
    }
    let event = SourceEvent {
        id: synesthesia_core::domain::UtteranceId {
            input: input(),
            utterance_id: 1,
        },
        revision: 1,
        phase: Phase::Final,
        text: "sensitive-subtitle".into(),
        start_sample: 0,
        end_sample: 1,
    };
    assert!(
        !format!("{:?}", Envelope::new(1, Message::Source { event }))
            .contains("sensitive-subtitle")
    );
    assert!(!format!("{:?}", frame()).contains("-32768"));
}

#[test]
fn every_control_and_translation_message_has_a_valid_roundtrip() {
    use synesthesia_core::domain::{TranslationEvent, TranslationResult, UtteranceId};
    let translation_fixture = include_str!("fixtures/translation.json").trim();
    let envelope = decode_json(translation_fixture.as_bytes()).unwrap();
    assert_eq!(
        encode_json(&envelope).unwrap(),
        translation_fixture.as_bytes()
    );
    let messages = [
        hello(),
        start(),
        Message::HelloAck {
            sample_rate: 16_000,
            channels: 1,
            max_frame_samples: 3_200,
            max_audio_samples: 32_000,
            max_text_bytes: 8_192,
        },
        Message::SessionStarted { input: input() },
        Message::InputSwitch { input: input() },
        Message::InputSwitched { input: input() },
        Message::SessionStop { input: input() },
        Message::SessionStopped { input: input() },
        Message::Status {
            state: ServiceState::Ready,
        },
        Message::Error {
            code: ErrorCode::Overloaded,
        },
        Message::Translation {
            event: TranslationEvent {
                id: UtteranceId {
                    input: input(),
                    utterance_id: 1,
                },
                source_revision: 1,
                outcome: TranslationResult::Failed {
                    code: ErrorCode::InferenceFailed,
                },
            },
        },
    ];
    for message in messages {
        let envelope = Envelope::new(1, message);
        assert_eq!(
            decode_json(&encode_json(&envelope).unwrap()).unwrap(),
            envelope
        );
    }
}
