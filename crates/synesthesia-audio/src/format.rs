//! Pure descriptor validation and decoding; packet memory never reaches this API.
use synesthesia_core::ErrorCode;

pub const MAX_RATE: u32 = 192_000;
pub const MAX_CHANNELS: u16 = 8;
pub const SPEAKERS: u32 = 0x0003_ffff;
pub const LFE: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Pcm,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MixFormat {
    pub rate: u32,
    pub channels: u16,
    pub container_bits: u16,
    pub valid_bits: u16,
    pub block_align: u16,
    pub channel_mask: u32,
    pub encoding: Encoding,
}

impl MixFormat {
    /// Decode a complete little-endian WAVEFORMATEX allocation, including its
    /// declared extension. This is also the sole EXTENSIBLE boundary for WASAPI.
    pub fn from_wave_descriptor(data: &[u8]) -> Result<Self, ErrorCode> {
        if data.len() < 18 || data.len() > 1042 {
            return Err(ErrorCode::InvalidAudio);
        }
        let u16_at = |offset| u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
        let u32_at = |offset| u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        let extra = usize::from(u16_at(16));
        if data.len() != 18 + extra {
            return Err(ErrorCode::InvalidAudio);
        }
        let tag = u16_at(0);
        let bits = u16_at(14);
        let mut format = Self {
            channels: u16_at(2),
            rate: u32_at(4),
            block_align: u16_at(12),
            container_bits: bits,
            valid_bits: bits,
            channel_mask: 0,
            encoding: match tag {
                1 | 0xfffe => Encoding::Pcm,
                3 => Encoding::Float,
                _ => return Err(ErrorCode::InvalidAudio),
            },
        };
        if tag == 0xfffe {
            if extra < 22 {
                return Err(ErrorCode::InvalidAudio);
            }
            format.valid_bits = u16_at(18);
            format.channel_mask = u32_at(20);
            const GUID_TAIL: [u8; 12] = [0, 0, 0x10, 0, 0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71];
            if data[28..40] != GUID_TAIL {
                return Err(ErrorCode::InvalidAudio);
            }
            format.encoding = match u32_at(24) {
                1 => Encoding::Pcm,
                3 => Encoding::Float,
                _ => return Err(ErrorCode::InvalidAudio),
            };
        }
        if u32_at(8)
            != format
                .rate
                .checked_mul(u32::from(format.block_align))
                .ok_or(ErrorCode::Overflow)?
        {
            return Err(ErrorCode::InvalidAudio);
        }
        format.validate()
    }

    pub fn validate(self) -> Result<Self, ErrorCode> {
        let bytes = self.container_bits / 8;
        if !(8_000..=MAX_RATE).contains(&self.rate)
            || !(1..=MAX_CHANNELS).contains(&self.channels)
            || self.block_align != bytes * self.channels
            || self.channel_mask & !SPEAKERS != 0
            || (self.channel_mask != 0
                && self.channel_mask.count_ones() != u32::from(self.channels))
            || (self.channels > 2 && self.channel_mask == 0)
            || self.channel_mask == LFE
        {
            return Err(ErrorCode::InvalidAudio);
        }
        match self.encoding {
            Encoding::Pcm
                if [16, 24, 32].contains(&self.container_bits)
                    && (2..=self.container_bits).contains(&self.valid_bits) => {}
            Encoding::Float if self.container_bits == 32 && self.valid_bits == 32 => {}
            _ => return Err(ErrorCode::InvalidAudio),
        }
        Ok(self)
    }

    /// A packet is at most 100 ms. An absent packet is not a silent packet.
    pub fn packet_bytes(self, frames: usize) -> Result<usize, ErrorCode> {
        if frames == 0 || frames > self.rate as usize / 10 {
            return Err(ErrorCode::InvalidAudio);
        }
        frames
            .checked_mul(usize::from(self.block_align))
            .ok_or(ErrorCode::Overflow)
    }

    pub fn decode(
        self,
        data: &[u8],
        frames: usize,
        silent: bool,
        mono: &mut Vec<f32>,
    ) -> Result<(), ErrorCode> {
        self.validate()?;
        let expected = self.packet_bytes(frames)?;
        if (!silent && data.len() != expected) || (silent && !data.is_empty()) {
            return Err(ErrorCode::InvalidAudio);
        }
        mono.clear();
        if silent {
            mono.resize(frames, 0.0);
            return Ok(());
        }
        let bytes = usize::from(self.container_bits / 8);
        let weights = self.weights();
        for frame in data.chunks_exact(usize::from(self.block_align)) {
            let mut value = 0.0;
            for (channel, sample) in frame.chunks_exact(bytes).enumerate() {
                let decoded = match self.encoding {
                    Encoding::Float => f32::from_le_bytes(sample.try_into().unwrap()),
                    Encoding::Pcm => {
                        let container = match bytes {
                            2 => i32::from(i16::from_le_bytes(sample.try_into().unwrap())),
                            3 => {
                                (i32::from(sample[0])
                                    | (i32::from(sample[1]) << 8)
                                    | (i32::from(sample[2]) << 16))
                                    << 8
                                    >> 8
                            }
                            4 => i32::from_le_bytes(sample.try_into().unwrap()),
                            _ => unreachable!(),
                        };
                        let signed = container >> (self.container_bits - self.valid_bits);
                        signed as f32 / (2_f32.powi(i32::from(self.valid_bits) - 1))
                    }
                };
                if !decoded.is_finite() {
                    return Err(ErrorCode::InvalidAudio);
                }
                value += decoded * weights[channel];
            }
            if !value.is_finite() {
                return Err(ErrorCode::InvalidAudio);
            }
            mono.push(value);
        }
        Ok(())
    }

    fn weights(self) -> [f32; MAX_CHANNELS as usize] {
        let mut weights = [0.0; MAX_CHANNELS as usize];
        let mut channel = 0;
        let mut included = 0;
        if self.channel_mask == 0 {
            weights[..usize::from(self.channels)].fill(1.0 / f32::from(self.channels));
            return weights;
        }
        for bit in 0..18 {
            let position = 1 << bit;
            if self.channel_mask & position != 0 {
                if position != LFE {
                    weights[channel] = 1.0;
                    included += 1;
                }
                channel += 1;
            }
        }
        for weight in &mut weights {
            *weight /= included as f32;
        }
        weights
    }
}
