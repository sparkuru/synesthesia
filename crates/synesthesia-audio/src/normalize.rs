//! Persistent bandlimited conversion. Only actual input counts own output time.
use crate::format::MixFormat;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use synesthesia_core::{
    ErrorCode, MAX_FRAME_SAMPLES, SAMPLE_RATE,
    domain::{AudioChunk, InputId},
};

const CHUNK: usize = 256;

pub struct Normalizer {
    format: MixFormat,
    input: InputId,
    resampler: Option<SincFixedIn<f32>>,
    fifo: Vec<f32>,
    mono: Vec<f32>,
    output: Vec<Vec<f32>>,
    delay: usize,
    actual_frames: u64,
    emitted: u64,
    seq: u64,
}

impl Normalizer {
    pub fn new(format: MixFormat, input: InputId) -> Result<Self, ErrorCode> {
        let format = format.validate()?;
        input.validate()?;
        let resampler = Self::resampler(format)?;
        let delay = resampler.as_ref().map_or(0, Resampler::output_delay);
        let output = resampler
            .as_ref()
            .map_or_else(|| vec![Vec::new()], |r| r.output_buffer_allocate(true));
        Ok(Self {
            format,
            input,
            resampler,
            fifo: Vec::with_capacity(CHUNK),
            mono: Vec::with_capacity(format.rate as usize / 10),
            output,
            delay,
            actual_frames: 0,
            emitted: 0,
            seq: 0,
        })
    }

    fn resampler(format: MixFormat) -> Result<Option<SincFixedIn<f32>>, ErrorCode> {
        if format.rate == SAMPLE_RATE {
            return Ok(None);
        }
        SincFixedIn::new(
            f64::from(SAMPLE_RATE) / f64::from(format.rate),
            1.0,
            SincInterpolationParameters {
                sinc_len: 256,
                f_cutoff: 0.95,
                interpolation: SincInterpolationType::Linear,
                oversampling_factor: 256,
                window: WindowFunction::BlackmanHarris2,
            },
            CHUNK,
            1,
        )
        .map(Some)
        .map_err(|_| ErrorCode::InvalidAudio)
    }

    pub fn push(
        &mut self,
        data: &[u8],
        frames: usize,
        silent: bool,
        mut publish: impl FnMut(AudioChunk) -> Result<(), ErrorCode>,
    ) -> Result<(), ErrorCode> {
        self.format.decode(data, frames, silent, &mut self.mono)?;
        self.actual_frames = self
            .actual_frames
            .checked_add(frames as u64)
            .ok_or(ErrorCode::Overflow)?;
        if self.resampler.is_none() {
            let samples = std::mem::take(&mut self.mono);
            let result = self.emit(&samples, &mut publish);
            self.mono = samples;
            return result;
        }
        let mut position = 0;
        while position < self.mono.len() {
            let count = (CHUNK - self.fifo.len()).min(self.mono.len() - position);
            self.fifo
                .extend_from_slice(&self.mono[position..position + count]);
            position += count;
            if self.fifo.len() == CHUNK {
                self.process(&mut publish)?;
            }
        }
        Ok(())
    }

    fn process(
        &mut self,
        publish: &mut impl FnMut(AudioChunk) -> Result<(), ErrorCode>,
    ) -> Result<(), ErrorCode> {
        let (_, written) = self
            .resampler
            .as_mut()
            .ok_or(ErrorCode::InvalidState)?
            .process_into_buffer(&[&self.fifo], &mut self.output, None)
            .map_err(|_| ErrorCode::InvalidAudio)?;
        self.fifo.clear();
        let skip = self.delay.min(written);
        self.delay -= skip;
        let output = std::mem::take(&mut self.output);
        let result = self.emit(&output[0][skip..written], publish);
        self.output = output;
        result
    }

    fn emit(
        &mut self,
        samples: &[f32],
        publish: &mut impl FnMut(AudioChunk) -> Result<(), ErrorCode>,
    ) -> Result<(), ErrorCode> {
        let target = self
            .actual_frames
            .checked_mul(u64::from(SAMPLE_RATE))
            .ok_or(ErrorCode::Overflow)?
            / u64::from(self.format.rate);
        let available = usize::try_from(target.saturating_sub(self.emitted))
            .map_err(|_| ErrorCode::Overflow)?;
        for block in samples[..samples.len().min(available)].chunks(MAX_FRAME_SAMPLES as usize) {
            let mut pcm = Vec::with_capacity(block.len());
            for &sample in block {
                if !sample.is_finite() {
                    return Err(ErrorCode::InvalidAudio);
                }
                pcm.push(
                    (sample.clamp(-1.0, 1.0) * 32768.0)
                        .round()
                        .clamp(-32768.0, 32767.0) as i16,
                );
            }
            let next = self
                .emitted
                .checked_add(pcm.len() as u64)
                .ok_or(ErrorCode::Overflow)?;
            let next_seq = self.seq.checked_add(1).ok_or(ErrorCode::Overflow)?;
            publish(AudioChunk {
                input: self.input,
                seq: self.seq,
                start_sample: self.emitted,
                pcm,
            })?;
            self.emitted = next;
            self.seq = next_seq;
        }
        Ok(())
    }

    /// Finish an actual segment at observed output idle. Filter padding is never
    /// assigned audio time. Resetting avoids carrying virtual padding into resume.
    /// Stop/switch does not call this: it discards all old pending content instead.
    pub fn finish_segment(
        &mut self,
        mut publish: impl FnMut(AudioChunk) -> Result<(), ErrorCode>,
    ) -> Result<(), ErrorCode> {
        if self.resampler.is_none() {
            return Ok(());
        }
        let target = self
            .actual_frames
            .checked_mul(u64::from(SAMPLE_RATE))
            .ok_or(ErrorCode::Overflow)?
            / u64::from(self.format.rate);
        for _ in 0..8 {
            if self.emitted == target {
                break;
            }
            self.fifo.resize(CHUNK, 0.0);
            self.process(&mut publish)?;
        }
        if self.emitted != target {
            return Err(ErrorCode::InvalidAudio);
        }
        self.fifo.clear();
        self.resampler = Self::resampler(self.format)?;
        self.delay = self.resampler.as_ref().map_or(0, Resampler::output_delay);
        Ok(())
    }

    pub fn emitted_samples(&self) -> u64 {
        self.emitted
    }
}
