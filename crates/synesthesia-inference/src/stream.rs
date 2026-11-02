use super::{IDLE_FINALIZE_MILLIS, MAX_WINDOW_SAMPLES};
use synesthesia_core::domain::AudioChunk;
use synesthesia_core::{ErrorCode, SAMPLE_RATE};

const DRAFT_STEP: usize = SAMPLE_RATE as usize;
const SILENCE_SAMPLES: u64 = SAMPLE_RATE as u64 * IDLE_FINALIZE_MILLIS / 1000;
// Initial segmentation parameter, subject to the approved real speech baseline.
const VOICE_PEAK: i32 = 120;

pub(crate) struct Segment {
    pub pcm: Vec<i16>,
    pub start: u64,
    pub end: u64,
    pub finalized: bool,
}
#[derive(Default)]
pub(crate) struct Window {
    pcm: Vec<i16>,
    start: u64,
    last_voice: u64,
    cursor: u64,
    next_draft: usize,
    finalized: bool,
    submitted_final: bool,
}
impl Window {
    pub fn cursor(&self) -> u64 {
        self.cursor
    }
    pub fn is_final(&self) -> bool {
        self.finalized
    }
    pub fn accept(&mut self, frame: &AudioChunk) -> Result<bool, ErrorCode> {
        if self.finalized {
            return Err(ErrorCode::InvalidState);
        }
        if frame.start_sample != self.cursor {
            return Err(ErrorCode::Discontinuity);
        }
        let end = frame
            .start_sample
            .checked_add(frame.pcm.len() as u64)
            .ok_or(ErrorCode::Overflow)?;
        if !self.pcm.is_empty() && self.pcm.len() + frame.pcm.len() > MAX_WINDOW_SAMPLES {
            self.finalized = true;
            return Ok(false);
        }
        let voiced = frame
            .pcm
            .iter()
            .any(|&sample| i32::from(sample).abs() >= VOICE_PEAK);
        if self.pcm.is_empty() && voiced {
            self.start = frame.start_sample;
            self.next_draft = DRAFT_STEP;
        }
        if voiced {
            self.last_voice = end;
        }
        if !self.pcm.is_empty() || voiced {
            self.pcm.extend_from_slice(&frame.pcm);
            if self.pcm.len() == MAX_WINDOW_SAMPLES || end - self.last_voice >= SILENCE_SAMPLES {
                self.finalized = true;
            }
        }
        self.cursor = end;
        Ok(true)
    }
    pub fn should_decode(&self) -> bool {
        self.finalized || (!self.pcm.is_empty() && self.pcm.len() >= self.next_draft)
    }
    pub fn segment(&self) -> Option<Segment> {
        if !self.should_decode() || self.submitted_final {
            return None;
        }
        Some(Segment {
            pcm: self.pcm.clone(),
            start: self.start,
            end: self.last_voice,
            finalized: self.finalized,
        })
    }
    pub fn submitted(&mut self) {
        if self.finalized {
            self.submitted_final = true;
        }
        self.next_draft = self.pcm.len().saturating_add(DRAFT_STEP);
    }
    pub fn finish(&mut self) {
        if !self.pcm.is_empty() {
            self.finalized = true;
        }
    }
    pub fn clear_segment(&mut self) {
        self.pcm.fill(0);
        self.pcm.clear();
        self.finalized = false;
        self.submitted_final = false;
        self.next_draft = DRAFT_STEP;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use synesthesia_core::domain::InputId;
    fn frame(start: u64, n: usize, value: i16) -> AudioChunk {
        AudioChunk {
            input: InputId {
                session_id: 1,
                input_generation: 1,
            },
            seq: 0,
            start_sample: start,
            pcm: vec![value; n],
        }
    }
    #[test]
    fn leading_silence_never_creates_a_hallucinated_utterance() {
        let mut window = Window::default();
        for i in 0..100 {
            window.accept(&frame(i * 3200, 3200, 0)).unwrap();
        }
        window.finish();
        assert!(window.segment().is_none());
        assert_eq!(window.cursor(), 320000);
    }
    #[test]
    fn idle_control_uses_observed_samples_without_padding() {
        let mut window = Window::default();
        window.accept(&frame(0, 3200, 500)).unwrap();
        window.finish();
        let final_segment = window.segment().unwrap();
        assert!(final_segment.finalized);
        assert_eq!(final_segment.pcm.len(), 3200);
        assert_eq!((final_segment.start, final_segment.end), (0, 3200));
    }
    #[test]
    fn trailing_real_silence_keeps_end_monotone_and_clears_audio() {
        let mut window = Window::default();
        for i in 0..5 {
            window.accept(&frame(i * 3200, 3200, 500)).unwrap();
        }
        let draft = window.segment().unwrap();
        window.submitted();
        for i in 5..9 {
            window.accept(&frame(i * 3200, 3200, 0)).unwrap();
        }
        let final_segment = window.segment().unwrap();
        assert!(final_segment.finalized);
        assert_eq!(draft.end, final_segment.end);
        window.clear_segment();
        assert!(window.pcm.is_empty());
        assert_eq!(window.cursor(), 28800);
    }
    #[test]
    fn max_window_ends_long_sentence_and_rejects_overflow() {
        let mut window = Window::default();
        for i in 0..50 {
            window.accept(&frame(i * 3200, 3200, 500)).unwrap();
        }
        assert!(window.is_final());
        assert_eq!(window.segment().unwrap().pcm.len(), MAX_WINDOW_SAMPLES);
        assert!(window.accept(&frame(160000, 1, 500)).is_err());
    }
    #[test]
    fn variable_frames_are_held_whole_without_advancing_samples() {
        let mut window = Window::default();
        for i in 0..49 {
            assert!(window.accept(&frame(i * 3200, 3200, 500)).unwrap());
        }
        assert!(window.accept(&frame(156800, 3199, 500)).unwrap());
        let next = frame(159999, 2, 500);
        assert!(!window.accept(&next).unwrap());
        let final_segment = window.segment().unwrap();
        assert_eq!(final_segment.end, 159999);
        assert_eq!(window.cursor(), 159999);
        window.clear_segment();
        assert!(window.accept(&next).unwrap());
        window.finish();
        let next_segment = window.segment().unwrap();
        assert_eq!((next_segment.start, next_segment.end), (159999, 160001));
        assert_eq!(next_segment.pcm.len(), 2);
    }
}
