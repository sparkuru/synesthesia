//! Versioned codecs only. Networking and handshake authentication belong to C4.
use serde::{Deserialize, Serialize};
use std::fmt;
use synesthesia_core::config::{SourceLanguage, TargetLanguage};
use synesthesia_core::domain::{InputId, PublishedEvent, SourceEvent, TranslationEvent};
use synesthesia_core::{
    ErrorCode, MAX_AUDIO_SAMPLES, MAX_FRAME_SAMPLES, MAX_TEXT_BYTES, SAMPLE_RATE,
};

pub const VERSION: u16 = 1;
pub const MAX_JSON_BYTES: usize = 65_536;
pub const AUDIO_HEADER_BYTES: usize = 52;
pub const MAX_AUDIO_BYTES: usize = AUDIO_HEADER_BYTES + MAX_FRAME_SAMPLES as usize * 2;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Loading,
    Ready,
    Failed,
    ShuttingDown,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Message {
    Hello {
        source: SourceLanguage,
        target: TargetLanguage,
    },
    HelloAck {
        sample_rate: u32,
        channels: u8,
        max_frame_samples: u32,
        max_audio_samples: u64,
        max_text_bytes: usize,
    },
    SessionStart {
        source: SourceLanguage,
        target: TargetLanguage,
    },
    SessionStarted {
        input: InputId,
    },
    InputSwitch {
        input: InputId,
    },
    InputSwitched {
        input: InputId,
    },
    Source {
        event: SourceEvent,
    },
    Translation {
        event: TranslationEvent,
    },
    Status {
        state: ServiceState,
    },
    Error {
        code: ErrorCode,
    },
    SessionStop {
        input: InputId,
    },
    SessionStopped {
        input: InputId,
    },
}
impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source { event } => f.debug_tuple("Source").field(event).finish(),
            Self::Translation { event } => f.debug_tuple("Translation").field(event).finish(),
            _ => f.write_str("Message { control: [typed] }"),
        }
    }
}
impl From<PublishedEvent> for Message {
    fn from(event: PublishedEvent) -> Self {
        match event {
            PublishedEvent::Source(event) => Self::Source { event },
            PublishedEvent::Translation(event) => Self::Translation { event },
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub protocol_version: u16,
    pub request_id: u64,
    pub message: Message,
}
impl Envelope {
    pub fn new(request_id: u64, message: Message) -> Self {
        Self {
            protocol_version: VERSION,
            request_id,
            message,
        }
    }
    pub fn validate(&self) -> Result<(), ErrorCode> {
        if self.protocol_version != VERSION {
            return Err(ErrorCode::UnsupportedVersion);
        }
        if self.request_id == 0 {
            return Err(ErrorCode::InvalidProtocol);
        }
        match &self.message {
            Message::HelloAck {
                sample_rate,
                channels,
                max_frame_samples,
                max_audio_samples,
                max_text_bytes,
            } => {
                if *sample_rate != SAMPLE_RATE
                    || *channels != 1
                    || *max_frame_samples != MAX_FRAME_SAMPLES
                    || *max_audio_samples != MAX_AUDIO_SAMPLES
                    || *max_text_bytes != MAX_TEXT_BYTES
                {
                    return Err(ErrorCode::InvalidProtocol);
                }
            }
            Message::Source { event } => event.validate()?,
            Message::Translation { event } => event.validate()?,
            Message::SessionStarted { input }
            | Message::SessionStop { input }
            | Message::SessionStopped { input }
            | Message::InputSwitch { input }
            | Message::InputSwitched { input } => input.validate()?,
            _ => {}
        }
        Ok(())
    }
}
pub fn decode_json(bytes: &[u8]) -> Result<Envelope, ErrorCode> {
    if bytes.len() > MAX_JSON_BYTES {
        return Err(ErrorCode::InvalidProtocol);
    }
    let envelope: Envelope =
        serde_json::from_slice(bytes).map_err(|_| ErrorCode::InvalidProtocol)?;
    envelope.validate()?;
    Ok(envelope)
}
pub fn encode_json(envelope: &Envelope) -> Result<Vec<u8>, ErrorCode> {
    envelope.validate()?;
    let bytes = serde_json::to_vec(envelope).map_err(|_| ErrorCode::InvalidProtocol)?;
    if bytes.len() > MAX_JSON_BYTES {
        return Err(ErrorCode::InvalidProtocol);
    }
    Ok(bytes)
}
pub use synesthesia_core::domain::{AudioChunk as AudioFrame, AudioOrder};
pub fn encode_audio(frame: &AudioFrame) -> Result<Vec<u8>, ErrorCode> {
    frame.validate()?;
    let mut bytes = Vec::with_capacity(AUDIO_HEADER_BYTES + frame.pcm.len() * 2);
    bytes.extend_from_slice(b"SYNA");
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&(AUDIO_HEADER_BYTES as u16).to_le_bytes());
    for value in [
        frame.input.session_id,
        frame.input.input_generation,
        frame.seq,
        frame.start_sample,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&(frame.pcm.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&[1, 1, 0, 0]);
    for sample in &frame.pcm {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(bytes)
}
fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    let mut array = [0; 8];
    array.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(array)
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    let mut array = [0; 4];
    array.copy_from_slice(&bytes[offset..offset + 4]);
    u32::from_le_bytes(array)
}
pub fn decode_audio(bytes: &[u8]) -> Result<AudioFrame, ErrorCode> {
    if bytes.len() < AUDIO_HEADER_BYTES || bytes.len() > MAX_AUDIO_BYTES || &bytes[..4] != b"SYNA" {
        return Err(ErrorCode::InvalidAudio);
    }
    if u16::from_le_bytes([bytes[4], bytes[5]]) != VERSION {
        return Err(ErrorCode::UnsupportedVersion);
    }
    let count = u32_at(bytes, 40);
    if u16::from_le_bytes([bytes[6], bytes[7]]) as usize != AUDIO_HEADER_BYTES
        || count == 0
        || count > MAX_FRAME_SAMPLES
        || bytes.len() != AUDIO_HEADER_BYTES + count as usize * 2
        || u32_at(bytes, 44) != SAMPLE_RATE
        || bytes[48..52] != [1, 1, 0, 0]
    {
        return Err(ErrorCode::InvalidAudio);
    }
    let frame = AudioFrame {
        input: InputId {
            session_id: u64_at(bytes, 8),
            input_generation: u64_at(bytes, 16),
        },
        seq: u64_at(bytes, 24),
        start_sample: u64_at(bytes, 32),
        pcm: bytes[AUDIO_HEADER_BYTES..]
            .chunks_exact(2)
            .map(|s| i16::from_le_bytes([s[0], s[1]]))
            .collect(),
    };
    frame.validate()?;
    Ok(frame)
}
/// Server-side connection state. C4 performs actual header authentication first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    AwaitHello,
    Idle,
    Starting,
    Running(InputId),
    Switching(InputId),
    Stopping(InputId),
    Closed,
}
pub struct ConnectionGuard {
    state: ConnectionState,
    last_request: u64,
    last_session: u64,
}
impl ConnectionGuard {
    /// Construction records that the caller already authenticated the transport.
    pub fn authenticated() -> Self {
        Self {
            state: ConnectionState::AwaitHello,
            last_request: 0,
            last_session: 0,
        }
    }
    pub fn state(&self) -> ConnectionState {
        self.state
    }
    pub fn receive(&mut self, envelope: &Envelope) -> Result<(), ErrorCode> {
        envelope.validate()?;
        if envelope.request_id <= self.last_request {
            return Err(ErrorCode::InvalidProtocol);
        }
        let next = match (&self.state, &envelope.message) {
            (ConnectionState::AwaitHello, Message::Hello { .. }) => ConnectionState::Idle,
            (ConnectionState::Idle, Message::SessionStart { .. }) => ConnectionState::Starting,
            (ConnectionState::Running(active), Message::SessionStop { input })
                if active == input =>
            {
                ConnectionState::Stopping(*input)
            }
            (ConnectionState::Running(active), Message::InputSwitch { input })
                if active.session_id == input.session_id
                    && active.input_generation.checked_add(1) == Some(input.input_generation) =>
            {
                ConnectionState::Switching(*input)
            }
            _ => return Err(ErrorCode::InvalidState),
        };
        self.state = next;
        self.last_request = envelope.request_id;
        Ok(())
    }
    pub fn started(&mut self, input: InputId) -> Result<(), ErrorCode> {
        input.validate()?;
        if self.state != ConnectionState::Starting
            || input.input_generation != 1
            || input.session_id <= self.last_session
        {
            return Err(ErrorCode::InvalidState);
        }
        self.state = ConnectionState::Running(input);
        self.last_session = input.session_id;
        Ok(())
    }
    pub fn switched(&mut self, input: InputId) -> Result<(), ErrorCode> {
        if self.state != ConnectionState::Switching(input) {
            return Err(ErrorCode::InvalidState);
        }
        self.state = ConnectionState::Running(input);
        Ok(())
    }
    pub fn stopped(&mut self, input: InputId) -> Result<(), ErrorCode> {
        if self.state != ConnectionState::Stopping(input) {
            return Err(ErrorCode::InvalidState);
        }
        self.state = ConnectionState::Idle;
        Ok(())
    }
    pub fn audio_allowed(&self, input: InputId) -> Result<(), ErrorCode> {
        if self.state != ConnectionState::Running(input) {
            return Err(ErrorCode::InvalidState);
        }
        Ok(())
    }
    pub fn close(&mut self) {
        self.state = ConnectionState::Closed;
    }
}
