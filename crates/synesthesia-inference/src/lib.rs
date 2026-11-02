//! Owned native inference processes with bounded transient input and output.

mod process;
mod stream;
pub use process::WorkStatus;

use process::{NativeProcess, Reply, WAIT};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;
use stream::{Segment, Window};
use synesthesia_core::config::{InferenceConfig, SourceLanguage};
use synesthesia_core::domain::{
    AudioChunk, AudioOrder, InputId, Phase, PublishedEvent, SourceEvent, TranslationEvent,
    TranslationResult, UtteranceId, validate_text,
};
use synesthesia_core::session::{BoundedQueue, ComputeAdapter};
use synesthesia_core::{ErrorCode, MAX_AUDIO_SAMPLES, MAX_EVENTS, MAX_TRANSLATIONS};

pub const CRISPASR_REVISION: &str = "d08ec2dd83411a8389745c97c4f2e3955e084280";
pub const LLAMA_REVISION: &str = "7fe450e19305b828c199d602c23a8337aaa1f03b";

/// Ten seconds of current utterance audio, plus the separate two-second ingress
/// queue. Native KV contexts are 2048 tokens; ASR/translation outputs 256/512.
pub const MAX_WINDOW_SAMPLES: usize = 160_000;
pub const IDLE_FINALIZE_MILLIS: u64 = 800;

#[derive(Clone)]
pub struct NativeTools {
    pub asr_helper: PathBuf,
    pub translation_helper: PathBuf,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InferenceMetrics {
    pub model_loads: u64,
    pub accepted_samples: u64,
    pub asr_graphs: u64,
    pub translation_graphs: u64,
    pub queued_samples: u64,
}

struct AsrJob {
    job: u64,
    segment: Segment,
    started: Instant,
}
struct TranslationJob {
    job: u64,
    source: SourceEvent,
    started: Instant,
}

/// Safe Rust owns two private processes. No shell, listening port, model download,
/// content log, prompt cache, or content temporary file is used.
pub struct NativeInference {
    config: InferenceConfig,
    tools: NativeTools,
    detected: bool,
    asr: Option<NativeProcess>,
    translation: Option<NativeProcess>,
    input: Option<InputId>,
    previous_input: Option<InputId>,
    source: SourceLanguage,
    order: Option<AudioOrder>,
    audio: BoundedQueue<AudioChunk>,
    held: Option<AudioChunk>,
    window: Window,
    asr_job: Option<AsrJob>,
    translation_job: Option<TranslationJob>,
    translation_cancel: Option<(u64, Instant)>,
    translations: VecDeque<SourceEvent>,
    events: VecDeque<PublishedEvent>,
    utterance: u64,
    revision: u64,
    metrics: InferenceMetrics,
}
impl NativeInference {
    pub fn new(
        config: InferenceConfig,
        tools: NativeTools,
        source: SourceLanguage,
    ) -> Result<Self, ErrorCode> {
        if !config.asr_model.is_absolute() || !config.translation_model.is_absolute() {
            return Err(ErrorCode::InvalidPath);
        }
        Ok(Self {
            config,
            tools,
            detected: false,
            asr: None,
            translation: None,
            input: None,
            previous_input: None,
            source,
            order: None,
            audio: BoundedQueue::new(100, MAX_AUDIO_SAMPLES)?,
            held: None,
            window: Window::default(),
            asr_job: None,
            translation_job: None,
            translation_cancel: None,
            translations: VecDeque::new(),
            events: VecDeque::new(),
            utterance: 1,
            revision: 0,
            metrics: InferenceMetrics::default(),
        })
    }
    pub fn start_input(&mut self, input: InputId, source: SourceLanguage) -> Result<(), ErrorCode> {
        input.validate()?;
        if self.asr.is_none() || self.translation.is_none() || self.input.is_some() {
            return Err(ErrorCode::InvalidState);
        }
        if let Some(previous) = self.previous_input
            && (input.session_id < previous.session_id
                || (input.session_id == previous.session_id
                    && input.input_generation <= previous.input_generation))
        {
            return Err(ErrorCode::InvalidState);
        }
        self.order = Some(AudioOrder::new(input)?);
        self.input = Some(input);
        self.previous_input = Some(input);
        self.source = source;
        self.utterance = 1;
        self.revision = 0;
        Ok(())
    }
    /// Coordinator supplies observed no-packet inactivity; this control creates
    /// no PCM samples. Call only after capture's normalization/raw queues drain.
    pub fn finish_utterance(
        &mut self,
        input: InputId,
        observed_end_sample: u64,
    ) -> Result<(), ErrorCode> {
        if self.input != Some(input)
            || !self.audio.is_empty()
            || self.held.is_some()
            || observed_end_sample != self.window.cursor()
        {
            return Err(ErrorCode::InvalidState);
        }
        self.window.finish();
        self.pump()
    }
    pub fn metrics(&self) -> InferenceMetrics {
        InferenceMetrics {
            queued_samples: self.audio.weight()
                + self.held.as_ref().map_or(0, |f| f.pcm.len() as u64),
            ..self.metrics
        }
    }
    pub fn process_ids(&self) -> Option<(u32, u32)> {
        Some((self.asr.as_ref()?.id(), self.translation.as_ref()?.id()))
    }
    pub fn probe_asr_work(&mut self) -> Result<WorkStatus, ErrorCode> {
        self.asr.as_mut().ok_or(ErrorCode::InvalidState)?.status()
    }
    pub fn has_pending_work(&self) -> bool {
        !self.audio.is_empty()
            || self.held.is_some()
            || self.asr_job.is_some()
            || self.translation_job.is_some()
            || self.translation_cancel.is_some()
            || !self.translations.is_empty()
            || !self.events.is_empty()
            || self.window.should_decode()
    }
    /// A numeric lifecycle experiment: idle probes execute a real clear/ack on
    /// both already-loaded helpers, without reloading or admitting session work.
    pub fn probe_idle_residency(&mut self) -> Result<InferenceMetrics, ErrorCode> {
        if self.input.is_some() {
            return Err(ErrorCode::InvalidState);
        }
        self.metrics.asr_graphs = self.asr.as_mut().ok_or(ErrorCode::InvalidState)?.cancel()?;
        self.metrics.translation_graphs = self
            .translation
            .as_mut()
            .ok_or(ErrorCode::InvalidState)?
            .cancel()?;
        Ok(self.metrics())
    }
    fn publish(&mut self, event: PublishedEvent) -> Result<(), ErrorCode> {
        if self.events.len() >= MAX_EVENTS {
            return Err(ErrorCode::Overloaded);
        }
        self.events.push_back(event);
        Ok(())
    }
    fn pump(&mut self) -> Result<(), ErrorCode> {
        if self.asr_job.is_none() {
            while !self.window.is_final() {
                let Some(frame) = self.held.take().or_else(|| self.audio.pop()) else {
                    break;
                };
                if !self.window.accept(&frame)? {
                    self.held = Some(frame);
                    break;
                }
                if self.window.should_decode() {
                    break;
                }
            }
            if let Some(segment) = self.window.segment() {
                let mut payload = Vec::with_capacity(segment.pcm.len() * 2 + 1);
                payload.push(match self.source {
                    SourceLanguage::English => 1,
                    SourceLanguage::Japanese => 2,
                });
                for sample in &segment.pcm {
                    payload.extend(sample.to_le_bytes());
                }
                let job = self
                    .asr
                    .as_mut()
                    .ok_or(ErrorCode::InvalidState)?
                    .submit(&payload)?;
                payload.fill(0);
                self.window.submitted();
                self.asr_job = Some(AsrJob {
                    job,
                    segment,
                    started: Instant::now(),
                });
            }
        } else {
            // Collect only up to a final boundary while a draft is running; the
            // subsequent audio remains in the explicit bounded ingress queue.
            while !self.window.is_final() {
                let Some(frame) = self.held.take().or_else(|| self.audio.pop()) else {
                    break;
                };
                if !self.window.accept(&frame)? {
                    self.held = Some(frame);
                    break;
                }
            }
        }
        if self.translation_job.is_none()
            && self.translation_cancel.is_none()
            && let Some(source) = self.translations.pop_front()
        {
            if let Some(translation) = self.translation.as_mut() {
                let job = translation.submit(source.text.as_bytes())?;
                self.translation_job = Some(TranslationJob {
                    job,
                    source,
                    started: Instant::now(),
                });
            } else {
                self.translation_failed(source)?;
            }
        }
        Ok(())
    }
    fn accept_asr_reply(&mut self, reply: Reply) -> Result<(), ErrorCode> {
        let current = self.asr_job.take().ok_or(ErrorCode::InvalidProtocol)?;
        if reply.job != current.job || reply.kind != 11 || reply.payload.len() < 8 {
            return Err(ErrorCode::InferenceFailed);
        }
        self.metrics.asr_graphs = u64::from_le_bytes(reply.payload[..8].try_into().unwrap());
        let text =
            String::from_utf8(reply.payload[8..].to_vec()).map_err(|_| ErrorCode::InvalidEvent)?;
        let text = text.trim().to_owned();
        if text.is_empty() && current.segment.finalized && self.revision != 0 {
            return Err(ErrorCode::InferenceFailed);
        }
        if !text.is_empty() {
            validate_text(&text)?;
            self.revision = self.revision.checked_add(1).ok_or(ErrorCode::Overflow)?;
            let event = SourceEvent {
                id: UtteranceId {
                    input: self.input.ok_or(ErrorCode::InvalidState)?,
                    utterance_id: self.utterance,
                },
                revision: self.revision,
                phase: if current.segment.finalized {
                    Phase::Final
                } else {
                    Phase::Draft
                },
                text,
                start_sample: current.segment.start,
                end_sample: current.segment.end,
            };
            event.validate()?;
            self.publish(PublishedEvent::Source(event))?;
        }
        if current.segment.finalized {
            self.window.clear_segment();
            self.utterance = self.utterance.checked_add(1).ok_or(ErrorCode::Overflow)?;
            self.revision = 0;
        }
        Ok(())
    }
    fn accept_translation_reply(&mut self, reply: Reply) -> Result<(), ErrorCode> {
        let current = self
            .translation_job
            .take()
            .ok_or(ErrorCode::InvalidProtocol)?;
        if reply.job != current.job || !matches!(reply.kind, 11 | 13) {
            return Err(ErrorCode::InvalidProtocol);
        }
        let outcome = if reply.kind == 13 {
            TranslationResult::Failed {
                code: ErrorCode::InferenceFailed,
            }
        } else {
            if reply.payload.len() < 8 {
                return Err(ErrorCode::InvalidProtocol);
            }
            self.metrics.translation_graphs =
                u64::from_le_bytes(reply.payload[..8].try_into().unwrap());
            let text = String::from_utf8(reply.payload[8..].to_vec())
                .map_err(|_| ErrorCode::InvalidEvent)?;
            let text = text.trim().to_owned();
            validate_text(&text)?;
            TranslationResult::Complete { text }
        };
        self.publish(PublishedEvent::Translation(TranslationEvent {
            id: current.source.id,
            source_revision: current.source.revision,
            outcome,
        }))
    }
    fn translation_failed(&mut self, source: SourceEvent) -> Result<(), ErrorCode> {
        self.publish(PublishedEvent::Translation(TranslationEvent {
            id: source.id,
            source_revision: source.revision,
            outcome: TranslationResult::Failed {
                code: ErrorCode::InferenceFailed,
            },
        }))
    }
    fn poll_translation(&mut self) -> Result<(), ErrorCode> {
        if self
            .translation_job
            .as_ref()
            .is_some_and(|job| job.started.elapsed() > WAIT)
        {
            let current = self.translation_job.take().ok_or(ErrorCode::InvalidState)?;
            let cancelled = self
                .translation
                .as_mut()
                .ok_or(ErrorCode::InvalidState)?
                .begin_cancel();
            self.translation_failed(current.source)?;
            match cancelled {
                Ok(job) => self.translation_cancel = Some((job, Instant::now())),
                Err(_) => {
                    self.translation.take();
                }
            }
        }
        if let Some((cancel_job, started)) = self.translation_cancel {
            if started.elapsed() > WAIT {
                self.translation.take();
                self.translation_cancel = None;
            } else {
                match self
                    .translation
                    .as_mut()
                    .ok_or(ErrorCode::InvalidState)?
                    .poll()
                {
                    Ok(Some(reply))
                        if reply.kind == 12
                            && reply.job == cancel_job
                            && reply.payload.len() == 8 =>
                    {
                        self.metrics.translation_graphs =
                            u64::from_le_bytes(reply.payload[..8].try_into().unwrap());
                        self.translation_cancel = None;
                    }
                    Ok(Some(reply)) if reply.job < cancel_job && matches!(reply.kind, 11 | 13) => {}
                    Ok(None) => {}
                    _ => {
                        self.translation.take();
                        self.translation_cancel = None;
                    }
                }
            }
        } else if self.translation_job.is_some() {
            match self
                .translation
                .as_mut()
                .ok_or(ErrorCode::InvalidState)?
                .poll()
            {
                Ok(Some(reply)) => self.accept_translation_reply(reply)?,
                Ok(None) => {}
                Err(_) => {
                    let current = self.translation_job.take().ok_or(ErrorCode::InvalidState)?;
                    self.translation.take();
                    self.translation_failed(current.source)?;
                }
            }
        }
        Ok(())
    }
    fn clear_input(&mut self) {
        self.input = None;
        self.order = None;
        self.audio.clear();
        self.held = None;
        self.window = Window::default();
        self.asr_job = None;
        self.translation_job = None;
        self.translation_cancel = None;
        self.translations.clear();
        self.events.clear();
    }
}
impl ComputeAdapter for NativeInference {
    fn detect(&mut self) -> Result<(), ErrorCode> {
        if self.input.is_some() || self.asr.is_some() || self.translation.is_some() {
            return Err(ErrorCode::InvalidState);
        }
        self.detected = false;
        for tool in [&self.tools.asr_helper, &self.tools.translation_helper] {
            let mut probe = NativeProcess::spawn(tool, None, &self.config.device)?;
            probe.wait_exit()?;
        }
        self.detected = true;
        Ok(())
    }
    fn load(&mut self) -> Result<(), ErrorCode> {
        if !self.detected
            || self.input.is_some()
            || self.asr.is_some()
            || self.translation.is_some()
        {
            return Err(ErrorCode::InvalidState);
        }
        verify_model(
            &self.config.asr_model,
            1_490_915_200,
            "ec197cef7ccc589fdcae1becc3f4a3de119d0a41e790b898b519b1a048dad8d4",
        )?;
        verify_model(
            &self.config.translation_model,
            989_983_712,
            "bf575b6238c6ae3c35db73af36bc0a749aedf8de17426c1b506e2d332dd9c9ca",
        )?;
        let asr = NativeProcess::spawn(
            &self.tools.asr_helper,
            Some(&self.config.asr_model),
            &self.config.device,
        )?;
        let translation = NativeProcess::spawn(
            &self.tools.translation_helper,
            Some(&self.config.translation_model),
            &self.config.device,
        )?;
        self.asr = Some(asr);
        self.translation = Some(translation);
        self.metrics.model_loads += 1;
        Ok(())
    }
    fn accept_audio(&mut self, audio: &AudioChunk) -> Result<(), ErrorCode> {
        if self.input != Some(audio.input) {
            return Err(ErrorCode::InvalidState);
        }
        self.order
            .as_mut()
            .ok_or(ErrorCode::InvalidState)?
            .accept(audio)?;
        if self.audio.len() + usize::from(self.held.is_some()) >= 100
            || self.audio.weight()
                + self.held.as_ref().map_or(0, |f| f.pcm.len() as u64)
                + audio.pcm.len() as u64
                > MAX_AUDIO_SAMPLES
        {
            return Err(ErrorCode::Overloaded);
        }
        self.audio.push(audio.clone(), audio.pcm.len() as u64)?;
        self.metrics.accepted_samples = self
            .metrics
            .accepted_samples
            .checked_add(audio.pcm.len() as u64)
            .ok_or(ErrorCode::Overflow)?;
        self.pump()
    }
    fn translate(&mut self, source: &SourceEvent) -> Result<(), ErrorCode> {
        source.validate()?;
        if source.phase != Phase::Final || self.input != Some(source.id.input) {
            return Err(ErrorCode::InvalidState);
        }
        if self.translations.len() + usize::from(self.translation_job.is_some()) >= MAX_TRANSLATIONS
        {
            return Err(ErrorCode::Overloaded);
        }
        if self.translations.iter().any(|s| s.id == source.id)
            || self
                .translation_job
                .as_ref()
                .is_some_and(|s| s.source.id == source.id)
        {
            return Err(ErrorCode::InvalidState);
        }
        self.translations.push_back(source.clone());
        self.pump()
    }
    fn next_event(&mut self) -> Result<Option<PublishedEvent>, ErrorCode> {
        if self.input.is_none() {
            return Ok(None);
        }
        if let Some(current) = &self.asr_job {
            if current.started.elapsed() > WAIT {
                return Err(ErrorCode::InferenceFailed);
            }
            if let Some(reply) = self.asr.as_mut().ok_or(ErrorCode::InvalidState)?.poll()? {
                self.accept_asr_reply(reply)?;
            }
        }
        self.poll_translation()?;
        self.pump()?;
        Ok(self.events.pop_front())
    }
    fn cancel(&mut self, input: InputId) -> Result<(), ErrorCode> {
        if self.input != Some(input) {
            return Err(ErrorCode::InvalidState);
        }
        self.input = None;
        // Execute both cancellations even if either fails. A failed cancellation
        // invalidates model readiness; Drop terminates only these owned helpers.
        let asr = self.asr.as_mut().ok_or(ErrorCode::InvalidState)?.cancel();
        let translation = self
            .translation
            .as_mut()
            .map_or(Ok(0), NativeProcess::cancel);
        self.clear_input();
        match (asr, translation) {
            (Ok(a), Ok(t)) => {
                self.metrics.asr_graphs = a;
                self.metrics.translation_graphs = t;
                Ok(())
            }
            _ => {
                self.asr.take();
                self.translation.take();
                self.detected = false;
                Err(ErrorCode::InferenceFailed)
            }
        }
    }
    fn shutdown(&mut self) -> Result<(), ErrorCode> {
        self.clear_input();
        let a = self.asr.as_mut().map(NativeProcess::shutdown).transpose();
        let t = self
            .translation
            .as_mut()
            .map(NativeProcess::shutdown)
            .transpose();
        self.asr.take();
        self.translation.take();
        self.detected = false;
        if a.is_err() || t.is_err() {
            Err(ErrorCode::InferenceFailed)
        } else {
            Ok(())
        }
    }
}
fn verify_model(path: &Path, bytes: u64, expected: &str) -> Result<(), ErrorCode> {
    let mut file = File::open(path).map_err(|_| ErrorCode::InvalidPath)?;
    let before = file.metadata().map_err(|_| ErrorCode::InvalidPath)?;
    if !before.is_file() || before.len() != bytes {
        return Err(ErrorCode::InvalidConfig);
    }
    let mut digest = Sha256::new();
    let mut buffer = [0; 65_536];
    loop {
        let length = file.read(&mut buffer).map_err(|_| ErrorCode::InvalidPath)?;
        if length == 0 {
            break;
        }
        digest.update(&buffer[..length]);
    }
    let after = file.metadata().map_err(|_| ErrorCode::InvalidPath)?;
    if format!("{:x}", digest.finalize()) != expected
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
    {
        return Err(ErrorCode::InvalidConfig);
    }
    Ok(())
}
