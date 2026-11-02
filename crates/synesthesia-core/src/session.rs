//! Deterministic lifecycle and budgets; platform adapters execute these effects.
use crate::domain::{
    AudioChunk, AudioOrder, IdSequence, InputId, PublishedEvent, SourceEvent, SubtitleReducer,
    TranslationEvent, Update, UtteranceId,
};
use crate::{ErrorCode, MAX_AUDIO_SAMPLES, MAX_EVENTS, MAX_TRANSLATIONS};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeState {
    Uninitialized,
    Detecting,
    Loading,
    Ready,
    Failed,
    ShuttingDown,
    Released,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Starting,
    Running,
    Stopping,
    Failed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Detect,
    Load,
    StartCapture(InputId),
    StopCapture,
    CancelInput(InputId),
    ClearQueues,
    Shutdown,
}

/// Implementations own resources; successful return is actual adapter evidence.
pub trait ComputeAdapter {
    fn detect(&mut self) -> Result<(), ErrorCode>;
    fn load(&mut self) -> Result<(), ErrorCode>;
    fn accept_audio(&mut self, audio: &AudioChunk) -> Result<(), ErrorCode>;
    fn translate(&mut self, source: &SourceEvent) -> Result<(), ErrorCode>;
    /// Raw typed engine events still pass through Session identity/revision checks.
    fn next_event(&mut self) -> Result<Option<PublishedEvent>, ErrorCode>;
    fn cancel(&mut self, input: InputId) -> Result<(), ErrorCode>;
    fn shutdown(&mut self) -> Result<(), ErrorCode>;
}

/// A weight-and-count bounded queue. Failed pushes do not evict earlier data.
#[derive(Debug)]
pub struct BoundedQueue<T> {
    entries: VecDeque<(T, u64)>,
    max_count: usize,
    max_weight: u64,
    weight: u64,
}
impl<T> BoundedQueue<T> {
    pub fn new(max_count: usize, max_weight: u64) -> Result<Self, ErrorCode> {
        if max_count == 0 || max_weight == 0 {
            return Err(ErrorCode::InvalidConfig);
        }
        Ok(Self {
            entries: VecDeque::new(),
            max_count,
            max_weight,
            weight: 0,
        })
    }
    pub fn push(&mut self, item: T, weight: u64) -> Result<(), ErrorCode> {
        let total = self.weight.checked_add(weight).ok_or(ErrorCode::Overflow)?;
        if weight == 0 || total > self.max_weight || self.entries.len() >= self.max_count {
            return Err(ErrorCode::Overloaded);
        }
        self.entries.push_back((item, weight));
        self.weight = total;
        Ok(())
    }
    pub fn pop(&mut self) -> Option<T> {
        self.entries.pop_front().map(|(item, weight)| {
            self.weight -= weight;
            item
        })
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.weight = 0;
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn weight(&self) -> u64 {
        self.weight
    }
}

pub struct Session {
    compute: ComputeState,
    state: SessionState,
    sessions: IdSequence,
    input: Option<InputId>,
    history_capacity: usize,
    reducer: Option<SubtitleReducer>,
    audio: BoundedQueue<AudioChunk>,
    audio_order: Option<AudioOrder>,
    translations: BoundedQueue<SourceEvent>,
    events: BoundedQueue<PublishedEvent>,
    active_translations: Vec<(UtteranceId, u64)>,
    asr_active: Option<(InputId, u64)>,
    cleanup: Vec<Effect>,
}
impl Session {
    pub fn new(last_session: u64, history_capacity: usize) -> Result<Self, ErrorCode> {
        SubtitleReducer::new(
            InputId {
                session_id: 1,
                input_generation: 1,
            },
            history_capacity,
        )?;
        Ok(Self {
            compute: ComputeState::Uninitialized,
            state: SessionState::Idle,
            sessions: IdSequence::new(last_session),
            input: None,
            history_capacity,
            reducer: None,
            audio: BoundedQueue::new(100, MAX_AUDIO_SAMPLES)?,
            audio_order: None,
            translations: BoundedQueue::new(MAX_TRANSLATIONS, MAX_TRANSLATIONS as u64)?,
            events: BoundedQueue::new(MAX_EVENTS, MAX_EVENTS as u64)?,
            active_translations: Vec::new(),
            asr_active: None,
            cleanup: Vec::new(),
        })
    }
    pub fn compute_state(&self) -> ComputeState {
        self.compute
    }
    pub fn state(&self) -> SessionState {
        self.state
    }
    pub fn input(&self) -> Option<InputId> {
        self.input
    }
    pub fn subtitles(&self) -> Option<&SubtitleReducer> {
        self.reducer.as_ref()
    }
    pub fn initialize(&mut self) -> Result<Effect, ErrorCode> {
        if !matches!(
            self.compute,
            ComputeState::Uninitialized | ComputeState::Failed
        ) || self.state != SessionState::Idle
        {
            return Err(ErrorCode::InvalidState);
        }
        self.compute = ComputeState::Detecting;
        Ok(Effect::Detect)
    }
    pub fn detected(&mut self) -> Result<Effect, ErrorCode> {
        if self.compute != ComputeState::Detecting {
            return Err(ErrorCode::InvalidState);
        }
        self.compute = ComputeState::Loading;
        Ok(Effect::Load)
    }
    /// Called only after adapter evidence of both model loads, never enumeration.
    pub fn loaded(&mut self) -> Result<(), ErrorCode> {
        if self.compute != ComputeState::Loading {
            return Err(ErrorCode::InvalidState);
        }
        self.compute = ComputeState::Ready;
        Ok(())
    }
    pub fn start(&mut self) -> Result<Effect, ErrorCode> {
        if self.compute != ComputeState::Ready || self.state != SessionState::Idle {
            return Err(ErrorCode::InvalidState);
        }
        let input = InputId {
            session_id: self.sessions.allocate()?,
            input_generation: 1,
        };
        self.reducer = Some(SubtitleReducer::new(input, self.history_capacity)?);
        self.input = Some(input);
        self.audio_order = Some(AudioOrder::new(input)?);
        self.state = SessionState::Starting;
        Ok(Effect::StartCapture(input))
    }
    pub fn capture_started(&mut self, input: InputId) -> Result<(), ErrorCode> {
        if self.state != SessionState::Starting || self.input != Some(input) {
            return Err(ErrorCode::InvalidState);
        }
        self.state = SessionState::Running;
        Ok(())
    }
    pub fn push_audio(&mut self, frame: AudioChunk) -> Result<(), ErrorCode> {
        self.require_running(frame.input)?;
        let result = self
            .audio_order
            .as_mut()
            .ok_or(ErrorCode::InvalidState)?
            .accept(&frame);
        self.enforce_budget(result)?;
        let weight = frame.pcm.len() as u64;
        let result = self.audio.push(frame, weight);
        self.enforce_budget(result)
    }
    pub fn pop_audio(&mut self) -> Option<AudioChunk> {
        if self.asr_active.is_some() {
            return None;
        }
        let audio = self.audio.pop()?;
        self.asr_active = Some((audio.input, audio.seq));
        Some(audio)
    }
    /// A duplicate or stale completion must not release a different frame.
    pub fn asr_completed(&mut self, input: InputId, seq: u64) -> Result<(), ErrorCode> {
        self.require_running(input)?;
        if self.asr_active != Some((input, seq)) {
            return Err(ErrorCode::InvalidState);
        }
        self.asr_active = None;
        Ok(())
    }
    pub fn pop_translation(&mut self) -> Option<SourceEvent> {
        let event = self.translations.pop()?;
        self.active_translations.push((event.id, event.revision));
        Some(event)
    }
    pub fn pop_event(&mut self) -> Option<PublishedEvent> {
        self.events.pop()
    }
    pub fn pending(&self) -> (u64, usize, usize) {
        (
            self.audio.weight(),
            self.translations.len() + self.active_translations.len(),
            self.events.len(),
        )
    }
    /// A scheduler executes cancellation effects before acknowledging stop.
    pub fn take_cleanup(&mut self) -> Vec<Effect> {
        std::mem::take(&mut self.cleanup)
    }
    pub fn source(&mut self, event: SourceEvent) -> Result<Update, ErrorCode> {
        if self.state != SessionState::Running || self.input != Some(event.id.input) {
            return Ok(Update::Ignored);
        }
        let update = self
            .reducer
            .as_mut()
            .ok_or(ErrorCode::InvalidState)?
            .source(event.clone())?;
        if let Update::SourcePublished { translate } = &update {
            // Source becomes observable before translation can be consumed.
            let result = self.events.push(PublishedEvent::Source(event), 1);
            self.enforce_budget(result)?;
            if let Some(final_event) = translate {
                if self.translations.len() + self.active_translations.len() >= MAX_TRANSLATIONS {
                    return self.enforce_budget(Err(ErrorCode::Overloaded));
                }
                let result = self.translations.push(final_event.clone(), 1);
                self.enforce_budget(result)?;
            }
        }
        Ok(update)
    }
    pub fn translation(&mut self, event: TranslationEvent) -> Result<Update, ErrorCode> {
        if self.state != SessionState::Running || self.input != Some(event.id.input) {
            return Ok(Update::Ignored);
        }
        event.validate()?;
        let Some(index) = self
            .active_translations
            .iter()
            .position(|task| *task == (event.id, event.source_revision))
        else {
            return Ok(Update::Ignored);
        };
        self.active_translations.remove(index);
        let update = self
            .reducer
            .as_mut()
            .ok_or(ErrorCode::InvalidState)?
            .translation(event.clone())?;
        if update != Update::Ignored {
            let result = self.events.push(PublishedEvent::Translation(event), 1);
            self.enforce_budget(result)?;
        }
        Ok(update)
    }
    pub fn switch_input(&mut self) -> Result<Vec<Effect>, ErrorCode> {
        let old = self.input.ok_or(ErrorCode::InvalidState)?;
        self.require_running(old)?;
        let next = old.next_generation()?;
        // Adapter must complete stop/cancel before starting the new capture.
        self.clear();
        self.reducer
            .as_mut()
            .ok_or(ErrorCode::InvalidState)?
            .reset(next)?;
        self.input = Some(next);
        self.audio_order = Some(AudioOrder::new(next)?);
        self.state = SessionState::Starting;
        Ok(vec![
            Effect::StopCapture,
            Effect::CancelInput(old),
            Effect::ClearQueues,
            Effect::StartCapture(next),
        ])
    }
    pub fn stop(&mut self) -> Result<Vec<Effect>, ErrorCode> {
        if !matches!(
            self.state,
            SessionState::Running | SessionState::Starting | SessionState::Failed
        ) {
            return Err(ErrorCode::InvalidState);
        }
        self.state = SessionState::Stopping;
        let mut effects = vec![Effect::StopCapture];
        if let Some(input) = self.input.take() {
            effects.push(Effect::CancelInput(input));
        }
        self.clear();
        self.reducer = None;
        effects.push(Effect::ClearQueues);
        Ok(effects)
    }
    pub fn stopped(&mut self) -> Result<(), ErrorCode> {
        if self.state != SessionState::Stopping {
            return Err(ErrorCode::InvalidState);
        }
        self.state = SessionState::Idle;
        Ok(())
    }
    /// Any queue overload must end the session explicitly; never skip audio.
    pub fn fail(&mut self, inference_failure: bool) -> Vec<Effect> {
        let mut effects = vec![Effect::StopCapture];
        if let Some(input) = self.input.take() {
            effects.push(Effect::CancelInput(input));
        }
        self.clear();
        self.reducer = None;
        self.state = SessionState::Failed;
        if inference_failure {
            self.compute = ComputeState::Failed;
        }
        effects.push(Effect::ClearQueues);
        effects
    }
    pub fn shutdown(&mut self) -> Vec<Effect> {
        let mut effects = self.fail(false);
        self.compute = ComputeState::ShuttingDown;
        effects.push(Effect::Shutdown);
        effects
    }
    /// Adapter acknowledges all owned processes/tasks released before this call.
    pub fn released(&mut self) -> Result<(), ErrorCode> {
        if self.compute != ComputeState::ShuttingDown {
            return Err(ErrorCode::InvalidState);
        }
        self.compute = ComputeState::Released;
        self.state = SessionState::Idle;
        Ok(())
    }
    fn require_running(&self, input: InputId) -> Result<(), ErrorCode> {
        if self.state != SessionState::Running || self.input != Some(input) {
            return Err(ErrorCode::InvalidState);
        }
        Ok(())
    }
    fn enforce_budget<T>(&mut self, result: Result<T, ErrorCode>) -> Result<T, ErrorCode> {
        if matches!(
            result,
            Err(ErrorCode::Overloaded | ErrorCode::Overflow | ErrorCode::Discontinuity)
        ) {
            self.cleanup = self.fail(false);
        }
        result
    }
    fn clear(&mut self) {
        self.audio.clear();
        self.audio_order = None;
        self.translations.clear();
        self.events.clear();
        self.active_translations.clear();
        self.asr_active = None;
    }
}
