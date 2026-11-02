use crate::format::MixFormat;
use std::{
    collections::VecDeque,
    fmt,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    thread::JoinHandle,
    time::Instant,
};
use synesthesia_core::{
    ErrorCode,
    config::CaptureConfig,
    domain::{AudioChunk, InputId},
};

#[cfg(windows)]
pub(crate) const RAW_PACKETS: usize = 8;
#[cfg(any(windows, test))]
pub(crate) const OUTPUT_SAMPLES: usize = 3_200;
#[cfg(any(windows, test))]
pub(crate) const OUTPUT_FRAMES: usize = 64;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    PlatformUnavailable = 1,
    DeviceUnavailable,
    UnsupportedFormat,
    Backend,
    Overloaded,
    Discontinuity,
    InputChangeRequired,
    WorkerFailed,
    InvalidInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureError {
    pub kind: FailureKind,
    pub backend_code: i32,
}
impl CaptureError {
    pub(crate) fn new(kind: FailureKind) -> Self {
        Self {
            kind,
            backend_code: 0,
        }
    }
    pub fn code(self) -> ErrorCode {
        match self.kind {
            FailureKind::Overloaded => ErrorCode::Overloaded,
            FailureKind::Discontinuity | FailureKind::InputChangeRequired => {
                ErrorCode::Discontinuity
            }
            FailureKind::InvalidInput => ErrorCode::InvalidState,
            FailureKind::PlatformUnavailable => ErrorCode::NotImplemented,
            _ => ErrorCode::InvalidAudio,
        }
    }
}
impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ({:#x})", self.kind, self.backend_code)
    }
}
impl std::error::Error for CaptureError {}

/// Opaque output identity for settings. Debug never exports endpoint metadata.
#[derive(Clone)]
pub struct RenderDevice {
    pub id: String,
    pub is_default: bool,
}
impl fmt::Debug for RenderDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderDevice")
            .field("is_default", &self.is_default)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CaptureObservation {
    pub input: InputId,
    pub elapsed_ms: u64,
    pub no_packet_ms: u64,
    pub emitted_samples: u64,
    pub native_frames: u64,
    pub packets: u64,
    pub silent_packets: u64,
    pub timestamp_errors: u64,
    pub queued_samples: usize,
    pub raw_queued_frames: usize,
    pub idle_tail_drained: bool,
    pub capture_finished: bool,
    pub normalizer_finished: bool,
    pub failure: Option<CaptureError>,
}

pub(crate) struct Shared {
    pub input: InputId,
    pub born: Instant,
    pub stop: AtomicBool,
    #[cfg(windows)]
    pub dirty: AtomicBool,
    pub failed: AtomicU64,
    pub packets: AtomicU64,
    pub last_packet_ms: AtomicU64,
    pub native_frames: AtomicU64,
    pub silent_packets: AtomicU64,
    pub timestamp_errors: AtomicU64,
    pub emitted: AtomicU64,
    pub raw_frames: AtomicUsize,
    pub idle_drained: AtomicBool,
    pub capture_done: AtomicBool,
    pub normalizer_done: AtomicBool,
    pub output: Mutex<VecDeque<AudioChunk>>,
    pub output_samples: AtomicUsize,
}

impl Shared {
    #[cfg(any(windows, test))]
    pub fn new(input: InputId) -> Self {
        Self {
            input,
            born: Instant::now(),
            stop: AtomicBool::new(false),
            #[cfg(windows)]
            dirty: AtomicBool::new(false),
            failed: AtomicU64::new(0),
            packets: AtomicU64::new(0),
            last_packet_ms: AtomicU64::new(0),
            native_frames: AtomicU64::new(0),
            silent_packets: AtomicU64::new(0),
            timestamp_errors: AtomicU64::new(0),
            emitted: AtomicU64::new(0),
            raw_frames: AtomicUsize::new(0),
            idle_drained: AtomicBool::new(false),
            capture_done: AtomicBool::new(false),
            normalizer_done: AtomicBool::new(false),
            output: Mutex::new(VecDeque::with_capacity(OUTPUT_FRAMES)),
            output_samples: AtomicUsize::new(0),
        }
    }
    pub fn fail(&self, error: CaptureError) {
        // Failure is reserved control state, independent of a saturated PCM queue.
        let value = ((error.kind as u64) << 32) | u64::from(error.backend_code as u32);
        let _ = self
            .failed
            .compare_exchange(0, value, Ordering::AcqRel, Ordering::Acquire);
        self.stop.store(true, Ordering::Release);
    }
    pub fn error(&self) -> Option<CaptureError> {
        let value = self.failed.load(Ordering::Acquire);
        let kind = match value >> 32 {
            0 => return None,
            1 => FailureKind::PlatformUnavailable,
            2 => FailureKind::DeviceUnavailable,
            3 => FailureKind::UnsupportedFormat,
            4 => FailureKind::Backend,
            5 => FailureKind::Overloaded,
            6 => FailureKind::Discontinuity,
            7 => FailureKind::InputChangeRequired,
            8 => FailureKind::WorkerFailed,
            _ => FailureKind::InvalidInput,
        };
        Some(CaptureError {
            kind,
            backend_code: value as u32 as i32,
        })
    }
    #[cfg(any(windows, test))]
    pub fn publish(&self, frame: AudioChunk) -> Result<(), ErrorCode> {
        if self.stop.load(Ordering::Acquire) {
            return Err(ErrorCode::InvalidState);
        }
        let mut queue = self.output.lock().map_err(|_| ErrorCode::InvalidState)?;
        if self.stop.load(Ordering::Acquire) {
            return Err(ErrorCode::InvalidState);
        }
        let samples = self.output_samples.load(Ordering::Relaxed);
        if queue.len() == OUTPUT_FRAMES || samples + frame.pcm.len() > OUTPUT_SAMPLES {
            self.fail(CaptureError::new(FailureKind::Overloaded));
            return Err(ErrorCode::Overloaded);
        }
        let end = frame.start_sample + frame.pcm.len() as u64;
        self.output_samples
            .store(samples + frame.pcm.len(), Ordering::Relaxed);
        queue.push_back(frame);
        self.emitted.store(end, Ordering::Release);
        Ok(())
    }
    pub fn observe(&self) -> CaptureObservation {
        let elapsed_ms = self.born.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        CaptureObservation {
            input: self.input,
            elapsed_ms,
            no_packet_ms: elapsed_ms.saturating_sub(self.last_packet_ms.load(Ordering::Acquire)),
            emitted_samples: self.emitted.load(Ordering::Acquire),
            native_frames: self.native_frames.load(Ordering::Acquire),
            packets: self.packets.load(Ordering::Acquire),
            silent_packets: self.silent_packets.load(Ordering::Acquire),
            timestamp_errors: self.timestamp_errors.load(Ordering::Acquire),
            queued_samples: self.output_samples.load(Ordering::Acquire),
            raw_queued_frames: self.raw_frames.load(Ordering::Acquire),
            idle_tail_drained: self.idle_drained.load(Ordering::Acquire),
            capture_finished: self.capture_done.load(Ordering::Acquire),
            normalizer_finished: self.normalizer_done.load(Ordering::Acquire),
            failure: self.error(),
        }
    }
}

/// Publish worker termination even during unwinding; never leave a dead input
/// looking live until the caller next requests stop.
#[cfg(any(windows, test))]
pub(crate) struct WorkerCompletion {
    shared: Arc<Shared>,
    capture: bool,
}
#[cfg(any(windows, test))]
impl WorkerCompletion {
    pub fn new(shared: Arc<Shared>, capture: bool) -> Self {
        Self { shared, capture }
    }
}
#[cfg(any(windows, test))]
impl Drop for WorkerCompletion {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.shared
                .fail(CaptureError::new(FailureKind::WorkerFailed));
        }
        if self.capture {
            self.shared.capture_done.store(true, Ordering::Release);
        } else {
            self.shared.normalizer_done.store(true, Ordering::Release);
        }
    }
}

/// The capture worker owns the normalizer, including its unwind path.
#[cfg(any(windows, test))]
pub(crate) struct OwnedNormalizer {
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
}
#[cfg(any(windows, test))]
impl OwnedNormalizer {
    pub fn new(shared: Arc<Shared>, worker: JoinHandle<()>) -> Self {
        Self {
            shared,
            worker: Some(worker),
        }
    }
    pub fn finish(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            self.shared
                .fail(CaptureError::new(FailureKind::WorkerFailed));
        }
        self.shared.raw_frames.store(0, Ordering::Release);
    }
}
#[cfg(any(windows, test))]
impl Drop for OwnedNormalizer {
    fn drop(&mut self) {
        self.finish();
    }
}

/// Owns both workers. stop joins them and removes every pending old-input frame.
pub struct CaptureHandle {
    pub(crate) shared: Arc<Shared>,
    pub(crate) worker: Option<JoinHandle<()>>,
    pub format: MixFormat,
    pub device: RenderDevice,
}
impl CaptureHandle {
    pub fn start(selection: CaptureConfig, input: InputId) -> Result<Self, CaptureError> {
        input
            .validate()
            .map_err(|_| CaptureError::new(FailureKind::InvalidInput))?;
        #[cfg(windows)]
        {
            crate::native::start(selection, input)
        }
        #[cfg(not(windows))]
        {
            let _ = selection;
            Err(CaptureError::new(FailureKind::PlatformUnavailable))
        }
    }
    /// A terminal failure rejects all queued data before the caller admits more.
    pub fn next_chunk(&self) -> Result<Option<AudioChunk>, CaptureError> {
        if let Some(error) = self.shared.error() {
            return Err(error);
        }
        if self.shared.stop.load(Ordering::Acquire) {
            return Ok(None);
        }
        let mut queue = self
            .shared
            .output
            .lock()
            .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
        if let Some(error) = self.shared.error() {
            return Err(error);
        }
        let frame = queue.pop_front();
        if let Some(frame) = &frame {
            self.shared
                .output_samples
                .fetch_sub(frame.pcm.len(), Ordering::Relaxed);
        }
        Ok(frame)
    }
    pub fn observation(&self) -> CaptureObservation {
        self.shared.observe()
    }
    pub fn stop(&mut self) -> CaptureObservation {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            self.shared
                .fail(CaptureError::new(FailureKind::WorkerFailed));
        }
        if let Ok(mut queue) = self.shared.output.lock() {
            queue.clear();
        }
        self.shared.output_samples.store(0, Ordering::Release);
        self.shared.observe()
    }
}
impl Drop for CaptureHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn render_devices() -> Result<Vec<RenderDevice>, CaptureError> {
    #[cfg(windows)]
    {
        crate::native::devices()
    }
    #[cfg(not(windows))]
    {
        Err(CaptureError::new(FailureKind::PlatformUnavailable))
    }
}

/// Diagnostic-only, finite caller-owned playback on exactly the selected output.
/// Synthesizes tones in memory; never changes the default device or volume.
pub struct ControlledPlayback {
    #[cfg(windows)]
    inner: crate::native::Playback,
}
impl ControlledPlayback {
    pub fn start(device: &RenderDevice) -> Result<Self, CaptureError> {
        #[cfg(windows)]
        {
            Ok(Self {
                inner: crate::native::Playback::start(device.id.clone())?,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = device;
            Err(CaptureError::new(FailureKind::PlatformUnavailable))
        }
    }
    pub fn pause(&mut self) -> Result<(), CaptureError> {
        #[cfg(windows)]
        {
            self.inner.pause()
        }
        #[cfg(not(windows))]
        {
            Err(CaptureError::new(FailureKind::PlatformUnavailable))
        }
    }
    pub fn resume(&mut self) -> Result<(), CaptureError> {
        #[cfg(windows)]
        {
            self.inner.resume()
        }
        #[cfg(not(windows))]
        {
            Err(CaptureError::new(FailureKind::PlatformUnavailable))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> InputId {
        InputId {
            session_id: 7,
            input_generation: 3,
        }
    }
    fn frame(seq: u64, samples: usize) -> AudioChunk {
        AudioChunk {
            input: input(),
            seq,
            start_sample: seq * samples as u64,
            pcm: vec![1; samples],
        }
    }

    #[test]
    fn saturated_output_has_reserved_failure_and_never_drops_then_continues() {
        let state = Shared::new(input());
        state.publish(frame(0, OUTPUT_SAMPLES)).unwrap();
        assert_eq!(state.publish(frame(1, 1)), Err(ErrorCode::Overloaded));
        assert_eq!(state.error().unwrap().kind, FailureKind::Overloaded);
        assert!(state.stop.load(Ordering::Acquire));
        assert_eq!(state.output.lock().unwrap().len(), 1);
        assert_eq!(state.publish(frame(2, 1)), Err(ErrorCode::InvalidState));
    }

    #[test]
    fn tiny_frames_also_have_an_independent_count_bound() {
        let state = Shared::new(input());
        for seq in 0..OUTPUT_FRAMES as u64 {
            state.publish(frame(seq, 1)).unwrap();
        }
        assert_eq!(
            state.publish(frame(OUTPUT_FRAMES as u64, 1)),
            Err(ErrorCode::Overloaded)
        );
        assert_eq!(state.output_samples.load(Ordering::Acquire), OUTPUT_FRAMES);
    }

    #[test]
    fn stop_is_a_join_barrier_and_clears_old_output() {
        let shared = Arc::new(Shared::new(input()));
        shared.publish(frame(0, 100)).unwrap();
        let child = shared.clone();
        let worker = std::thread::spawn(move || {
            while !child.stop.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
            assert_eq!(child.publish(frame(1, 100)), Err(ErrorCode::InvalidState));
            child.capture_done.store(true, Ordering::Release);
            child.normalizer_done.store(true, Ordering::Release);
        });
        let mut handle = CaptureHandle {
            shared,
            worker: Some(worker),
            format: MixFormat {
                rate: 16000,
                channels: 1,
                container_bits: 16,
                valid_bits: 16,
                block_align: 2,
                channel_mask: 0,
                encoding: crate::format::Encoding::Pcm,
            },
            device: RenderDevice {
                id: "controlled-private-id".into(),
                is_default: true,
            },
        };
        let report = handle.stop();
        assert!(report.capture_finished && report.normalizer_finished);
        assert_eq!(report.queued_samples, 0);
        assert!(handle.next_chunk().unwrap().is_none());
        assert!(handle.worker.is_none());
        assert!(handle.stop().capture_finished);
    }

    #[test]
    fn debug_does_not_disclose_device_identity() {
        let device = RenderDevice {
            id: "controlled-private-id".into(),
            is_default: true,
        };
        assert!(!format!("{device:?}").contains("controlled-private-id"));
    }

    #[test]
    fn capture_unwind_joins_the_owned_normalizer_before_completion() {
        let shared = Arc::new(Shared::new(input()));
        let child = shared.clone();
        let owner = std::thread::spawn(move || {
            let _capture = WorkerCompletion::new(child.clone(), true);
            let norm_shared = child.clone();
            let normalizer = std::thread::spawn(move || {
                let _completion = WorkerCompletion::new(norm_shared.clone(), false);
                while !norm_shared.stop.load(Ordering::Acquire) {
                    std::thread::yield_now();
                }
                assert_eq!(
                    norm_shared.publish(frame(0, 1)),
                    Err(ErrorCode::InvalidState)
                );
            });
            let _normalizer = OwnedNormalizer::new(child, normalizer);
            panic!("controlled owner failure");
        });
        assert!(owner.join().is_err());
        assert!(shared.capture_done.load(Ordering::Acquire));
        assert!(shared.normalizer_done.load(Ordering::Acquire));
        assert_eq!(shared.error().unwrap().kind, FailureKind::WorkerFailed);
        assert!(shared.output.lock().unwrap().is_empty());
    }

    #[test]
    fn normalizer_panic_signals_failure_without_waiting_for_more_audio() {
        let shared = Arc::new(Shared::new(input()));
        let child = shared.clone();
        let normalizer = std::thread::spawn(move || {
            let _completion = WorkerCompletion::new(child, false);
            panic!("controlled normalizer failure");
        });
        let mut owner = OwnedNormalizer::new(shared.clone(), normalizer);
        owner.finish();
        assert_eq!(shared.error().unwrap().kind, FailureKind::WorkerFailed);
        assert!(shared.normalizer_done.load(Ordering::Acquire));
        assert!(shared.stop.load(Ordering::Acquire));
        owner.finish();
    }
}
