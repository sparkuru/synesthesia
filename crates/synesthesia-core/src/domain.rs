//! Single subtitle reducer shared by local and remote consumers.
use crate::{ErrorCode, MAX_HISTORY, MAX_TEXT_BYTES};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt};

/// Normalized audio retains identity and timing through every queue/adapter.
#[derive(Clone, PartialEq, Eq)]
pub struct AudioChunk {
    pub input: InputId,
    pub seq: u64,
    pub start_sample: u64,
    pub pcm: Vec<i16>,
}
impl fmt::Debug for AudioChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioChunk")
            .field("input", &self.input)
            .field("seq", &self.seq)
            .field("sample_count", &self.pcm.len())
            .finish_non_exhaustive()
    }
}
impl AudioChunk {
    pub fn validate(&self) -> Result<(), ErrorCode> {
        self.input.validate().map_err(|_| ErrorCode::InvalidAudio)?;
        if self.pcm.is_empty() || self.pcm.len() > crate::MAX_FRAME_SAMPLES as usize {
            return Err(ErrorCode::InvalidAudio);
        }
        self.start_sample
            .checked_add(self.pcm.len() as u64)
            .ok_or(ErrorCode::Overflow)?;
        self.seq.checked_add(1).ok_or(ErrorCode::Overflow)?;
        Ok(())
    }
}

/// One guard per input. Failed validation cannot advance the timeline.
pub struct AudioOrder {
    input: InputId,
    seq: u64,
    sample: u64,
}
impl AudioOrder {
    pub fn new(input: InputId) -> Result<Self, ErrorCode> {
        input.validate()?;
        Ok(Self {
            input,
            seq: 0,
            sample: 0,
        })
    }
    pub fn accept(&mut self, frame: &AudioChunk) -> Result<(), ErrorCode> {
        frame.validate()?;
        if frame.input != self.input {
            return Err(ErrorCode::InvalidState);
        }
        if frame.seq != self.seq || frame.start_sample != self.sample {
            return Err(ErrorCode::Discontinuity);
        }
        let next_seq = self.seq.checked_add(1).ok_or(ErrorCode::Overflow)?;
        let next_sample = self
            .sample
            .checked_add(frame.pcm.len() as u64)
            .ok_or(ErrorCode::Overflow)?;
        self.seq = next_seq;
        self.sample = next_sample;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputId {
    pub session_id: u64,
    pub input_generation: u64,
}
impl InputId {
    pub fn next_generation(self) -> Result<Self, ErrorCode> {
        self.validate()?;
        Ok(Self {
            input_generation: self
                .input_generation
                .checked_add(1)
                .ok_or(ErrorCode::Overflow)?,
            ..self
        })
    }
    pub fn validate(self) -> Result<(), ErrorCode> {
        if self.session_id == 0 || self.input_generation == 0 {
            return Err(ErrorCode::InvalidEvent);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UtteranceId {
    pub input: InputId,
    pub utterance_id: u64,
}
impl UtteranceId {
    pub fn validate(self) -> Result<(), ErrorCode> {
        self.input.validate()?;
        if self.utterance_id == 0 {
            return Err(ErrorCode::InvalidEvent);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Draft,
    Final,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceEvent {
    pub id: UtteranceId,
    pub revision: u64,
    pub phase: Phase,
    pub text: String,
    pub start_sample: u64,
    pub end_sample: u64,
}
impl fmt::Debug for SourceEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceEvent")
            .field("id", &self.id)
            .field("revision", &self.revision)
            .field("phase", &self.phase)
            .finish_non_exhaustive()
    }
}
impl SourceEvent {
    pub fn validate(&self) -> Result<(), ErrorCode> {
        self.id.validate()?;
        if self.revision == 0 || self.end_sample < self.start_sample {
            return Err(ErrorCode::InvalidEvent);
        }
        validate_text(&self.text)
    }
}
pub fn validate_text(text: &str) -> Result<(), ErrorCode> {
    if text.trim().is_empty()
        || text.len() > MAX_TEXT_BYTES
        || text
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(ErrorCode::InvalidEvent);
    }
    Ok(())
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case", deny_unknown_fields)]
pub enum TranslationResult {
    Complete { text: String },
    Failed { code: ErrorCode },
}
impl fmt::Debug for TranslationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete { .. } => f.write_str("Complete { text: [redacted] }"),
            Self::Failed { code } => f.debug_tuple("Failed").field(code).finish(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationEvent {
    pub id: UtteranceId,
    pub source_revision: u64,
    pub outcome: TranslationResult,
}
impl TranslationEvent {
    pub fn validate(&self) -> Result<(), ErrorCode> {
        self.id.validate()?;
        if self.source_revision == 0 {
            return Err(ErrorCode::InvalidEvent);
        }
        if let TranslationResult::Complete { text } = &self.outcome {
            validate_text(text)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subtitle {
    pub source: SourceEvent,
    pub translation: Option<TranslationResult>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Update {
    Ignored,
    SourcePublished { translate: Option<SourceEvent> },
    TranslationPublished,
}

/// Immutable payload captured at publication, not a notification to reread state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishedEvent {
    Source(SourceEvent),
    Translation(TranslationEvent),
}

pub struct SubtitleReducer {
    input: InputId,
    capacity: usize,
    highest_utterance: u64,
    history: VecDeque<Subtitle>,
}
impl SubtitleReducer {
    pub fn new(input: InputId, capacity: usize) -> Result<Self, ErrorCode> {
        input.validate()?;
        if !(1..=MAX_HISTORY).contains(&capacity) {
            return Err(ErrorCode::InvalidConfig);
        }
        Ok(Self {
            input,
            capacity,
            highest_utterance: 0,
            history: VecDeque::new(),
        })
    }
    pub fn history(&self) -> &VecDeque<Subtitle> {
        &self.history
    }
    pub fn clear(&mut self) {
        self.history.clear();
    }
    pub fn reset(&mut self, input: InputId) -> Result<(), ErrorCode> {
        input.validate()?;
        if input.session_id < self.input.session_id
            || (input.session_id == self.input.session_id
                && input.input_generation <= self.input.input_generation)
        {
            return Err(ErrorCode::InvalidState);
        }
        self.input = input;
        self.highest_utterance = 0;
        self.clear();
        Ok(())
    }
    pub fn source(&mut self, event: SourceEvent) -> Result<Update, ErrorCode> {
        event.validate()?;
        if event.id.input != self.input {
            return Ok(Update::Ignored);
        }
        if let Some(subtitle) = self.history.iter_mut().find(|s| s.source.id == event.id) {
            let previous = &subtitle.source;
            if previous.phase == Phase::Final || event.revision < previous.revision {
                return Ok(Update::Ignored);
            }
            if event.revision == previous.revision {
                if event.text != previous.text
                    || event.start_sample != previous.start_sample
                    || event.end_sample != previous.end_sample
                {
                    return Err(ErrorCode::InvalidEvent);
                }
                if event.phase == previous.phase {
                    return Ok(Update::Ignored);
                }
            }
            if event.start_sample != previous.start_sample || event.end_sample < previous.end_sample
            {
                return Err(ErrorCode::InvalidEvent);
            }
            subtitle.source = event.clone();
        } else {
            // The high watermark rejects evicted and previously unseen old IDs.
            if event.id.utterance_id <= self.highest_utterance {
                return Ok(Update::Ignored);
            }
            if self
                .history
                .back()
                .is_some_and(|s| event.start_sample < s.source.start_sample)
            {
                return Err(ErrorCode::InvalidEvent);
            }
            self.highest_utterance = event.id.utterance_id;
            if self.history.len() == self.capacity {
                self.history.pop_front();
            }
            self.history.push_back(Subtitle {
                source: event.clone(),
                translation: None,
            });
        }
        Ok(Update::SourcePublished {
            translate: (event.phase == Phase::Final).then_some(event),
        })
    }
    pub fn translation(&mut self, event: TranslationEvent) -> Result<Update, ErrorCode> {
        event.validate()?;
        if event.id.input != self.input {
            return Ok(Update::Ignored);
        }
        let Some(subtitle) = self.history.iter_mut().find(|s| s.source.id == event.id) else {
            return Ok(Update::Ignored);
        };
        if subtitle.source.phase != Phase::Final
            || subtitle.source.revision != event.source_revision
            || subtitle.translation.is_some()
        {
            return Ok(Update::Ignored);
        }
        subtitle.translation = Some(event.outcome);
        Ok(Update::TranslationPublished)
    }
}

/// Checked monotone IDs: exhaustion is an error, never a wrap to an old identity.
#[derive(Debug)]
pub struct IdSequence {
    current: u64,
}
impl IdSequence {
    pub fn new(last_assigned: u64) -> Self {
        Self {
            current: last_assigned,
        }
    }
    pub fn allocate(&mut self) -> Result<u64, ErrorCode> {
        self.current = self.current.checked_add(1).ok_or(ErrorCode::Overflow)?;
        Ok(self.current)
    }
}
