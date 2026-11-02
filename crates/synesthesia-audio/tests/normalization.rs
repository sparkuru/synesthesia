use synesthesia_audio::{
    format::{Encoding, MixFormat},
    normalize::Normalizer,
};
use synesthesia_core::{
    ErrorCode,
    domain::{AudioChunk, AudioOrder, InputId},
};

fn input(generation: u64) -> InputId {
    InputId {
        session_id: 1,
        input_generation: generation,
    }
}
fn format(
    rate: u32,
    channels: u16,
    bits: u16,
    valid: u16,
    encoding: Encoding,
    mask: u32,
) -> MixFormat {
    MixFormat {
        rate,
        channels,
        container_bits: bits,
        valid_bits: valid,
        block_align: channels * bits / 8,
        channel_mask: mask,
        encoding,
    }
}
fn decode(f: MixFormat, data: &[u8], frames: usize, silent: bool) -> Result<Vec<f32>, ErrorCode> {
    let mut output = Vec::new();
    f.validate()?.decode(data, frames, silent, &mut output)?;
    Ok(output)
}

#[test]
fn extensible_wave_descriptor_is_bounded_and_subtype_checked() {
    let mut descriptor = Vec::new();
    descriptor.extend(0xfffe_u16.to_le_bytes());
    descriptor.extend(2_u16.to_le_bytes());
    descriptor.extend(48_000_u32.to_le_bytes());
    descriptor.extend(384_000_u32.to_le_bytes());
    descriptor.extend(8_u16.to_le_bytes());
    descriptor.extend(32_u16.to_le_bytes());
    descriptor.extend(22_u16.to_le_bytes());
    descriptor.extend(24_u16.to_le_bytes());
    descriptor.extend(3_u32.to_le_bytes());
    descriptor.extend(1_u32.to_le_bytes());
    descriptor.extend([0, 0, 0x10, 0, 0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71]);
    assert_eq!(
        MixFormat::from_wave_descriptor(&descriptor).unwrap(),
        format(48_000, 2, 32, 24, Encoding::Pcm, 3)
    );
    for length in 0..40 {
        assert!(MixFormat::from_wave_descriptor(&descriptor[..length]).is_err());
    }
    for (offset, value) in [(16, 21), (18, 33), (20, 1), (24, 9), (28, 1), (8, 1)] {
        let mut invalid = descriptor.clone();
        invalid[offset] = value;
        assert!(MixFormat::from_wave_descriptor(&invalid).is_err());
    }
    descriptor[24] = 3;
    descriptor[18] = 32;
    assert_eq!(
        MixFormat::from_wave_descriptor(&descriptor)
            .unwrap()
            .encoding,
        Encoding::Float
    );
    descriptor.push(0);
    assert!(MixFormat::from_wave_descriptor(&descriptor).is_err());
}

#[test]
fn pcm_containers_and_left_aligned_valid_bits_preserve_amplitude() {
    for (bits, valid, bytes) in [
        (16, 16, vec![0, 0x40, 0, 0xc0]),
        (24, 24, vec![0, 0, 0x40, 0, 0, 0xc0]),
        (32, 24, vec![0, 0, 0, 0x40, 0, 0, 0, 0xc0]),
        (32, 32, vec![0, 0, 0, 0x40, 0, 0, 0, 0xc0]),
    ] {
        assert_eq!(
            decode(
                format(48_000, 1, bits, valid, Encoding::Pcm, 0),
                &bytes,
                2,
                false
            )
            .unwrap(),
            vec![0.5, -0.5]
        );
    }
}

#[test]
fn packed_pcm24_sign_extension_includes_negative_full_scale() {
    assert_eq!(
        decode(
            format(48_000, 1, 24, 24, Encoding::Pcm, 0),
            &[0, 0, 0x80, 0xff, 0xff, 0xff],
            2,
            false
        )
        .unwrap(),
        vec![-1.0, -1.0 / 8_388_608.0]
    );
}

#[test]
fn stereo_antiphase_and_center_only_surround_have_explicit_weights() {
    let data = [0.75_f32, -0.75]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    assert_eq!(
        decode(
            format(48_000, 2, 32, 32, Encoding::Float, 3),
            &data,
            1,
            false
        )
        .unwrap(),
        vec![0.0]
    );
    let data = [0.0_f32, 0.0, 0.75, 1.0, 0.0, 0.0]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    assert!(
        (decode(
            format(48_000, 6, 32, 32, Encoding::Float, 0x3f),
            &data,
            1,
            false
        )
        .unwrap()[0]
            - 0.15)
            .abs()
            < 1e-6
    );
}

#[test]
fn nonfinite_float_is_rejected_even_on_excluded_lfe_channel() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let data = [0.0_f32, 0.0, 0.0, value, 0.0, 0.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        assert_eq!(
            decode(
                format(48_000, 6, 32, 32, Encoding::Float, 0x3f),
                &data,
                1,
                false
            ),
            Err(ErrorCode::InvalidAudio)
        );
    }
}

#[test]
fn ambiguous_descriptors_and_packet_bounds_fail_before_decode() {
    let valid = format(48_000, 2, 32, 32, Encoding::Float, 3);
    for invalid in [
        MixFormat {
            rate: 384_000,
            ..valid
        },
        MixFormat {
            channels: 0,
            ..valid
        },
        MixFormat {
            block_align: 6,
            ..valid
        },
        MixFormat {
            valid_bits: 24,
            ..valid
        },
        MixFormat {
            channel_mask: 1,
            ..valid
        },
        MixFormat {
            channel_mask: 0x8000_0003,
            ..valid
        },
        format(48_000, 6, 16, 16, Encoding::Pcm, 0),
        format(48_000, 1, 16, 16, Encoding::Pcm, 8),
        format(48_000, 1, 8, 8, Encoding::Pcm, 0),
    ] {
        assert_eq!(invalid.validate(), Err(ErrorCode::InvalidAudio));
    }
    assert_eq!(
        decode(valid, &[0; 7], 1, false),
        Err(ErrorCode::InvalidAudio)
    );
    assert_eq!(decode(valid, &[], 0, true), Err(ErrorCode::InvalidAudio));
    assert_eq!(decode(valid, &[], 4801, true), Err(ErrorCode::InvalidAudio));
    assert_eq!(
        decode(valid, &[0; 8], 1, true),
        Err(ErrorCode::InvalidAudio)
    );
}

fn resample(rate: u32, frames: usize, frequency: f32, partition: usize) -> Vec<i16> {
    let f = format(rate, 1, 32, 32, Encoding::Float, 0);
    let mut normalizer = Normalizer::new(f, input(1)).unwrap();
    let mut order = AudioOrder::new(input(1)).unwrap();
    let mut output = Vec::new();
    let mut publish = |chunk: AudioChunk| {
        order.accept(&chunk)?;
        output.extend(chunk.pcm);
        Ok(())
    };
    let mut position = 0;
    while position < frames {
        let count = partition.min(frames - position);
        let bytes = (position..position + count)
            .flat_map(|sample| {
                (0.5 * (2.0 * std::f32::consts::PI * frequency * sample as f32 / rate as f32).sin())
                    .to_le_bytes()
            })
            .collect::<Vec<_>>();
        normalizer.push(&bytes, count, false, &mut publish).unwrap();
        position += count;
    }
    normalizer.finish_segment(&mut publish).unwrap();
    assert_eq!(
        normalizer.emitted_samples(),
        frames as u64 * 16_000 / u64::from(rate)
    );
    output
}

#[test]
fn fractional_rates_and_filter_tail_have_exact_sample_counts() {
    for rate in [8_000, 16_000, 44_100, 48_000, 96_000, 192_000] {
        for frames in [1, 17, 255, 256, 257, rate as usize + 137] {
            let output = resample(rate, frames, 900.0, (rate as usize / 200).max(1));
            assert_eq!(output.len(), frames * 16_000 / rate as usize);
        }
    }
}

#[test]
fn packet_partition_does_not_reset_fractional_phase_or_filter() {
    for rate in [44_100, 48_000, 96_000] {
        let reference = resample(rate, rate as usize * 3 + 97, 997.0, 127);
        assert_eq!(
            reference,
            resample(rate, rate as usize * 3 + 97, 997.0, rate as usize / 10)
        );
    }
}

#[test]
fn downsample_rejects_above_nyquist_energy_without_hiding_speech_band() {
    let rms = |samples: &[i16]| {
        (samples[1000..samples.len() - 1000]
            .iter()
            .map(|&v| f64::from(v).powi(2))
            .sum::<f64>()
            / (samples.len() - 2000) as f64)
            .sqrt()
    };
    for rate in [44_100, 48_000, 96_000] {
        let pass = resample(rate, rate as usize * 2, 1000.0, 113);
        let reject = resample(rate, rate as usize * 2, 12_000.0, 113);
        assert!(rms(&pass) > 9000.0);
        assert!(rms(&reject) < rms(&pass) / 100.0);
    }
}

#[test]
fn actual_silent_packets_own_time_but_absence_adds_nothing() {
    let f = format(48_000, 2, 32, 32, Encoding::Float, 3);
    let mut n = Normalizer::new(f, input(1)).unwrap();
    let mut chunks = Vec::new();
    n.push(&[], 480, true, |c| {
        chunks.push(c);
        Ok(())
    })
    .unwrap();
    n.finish_segment(|c| {
        chunks.push(c);
        Ok(())
    })
    .unwrap();
    assert_eq!(n.emitted_samples(), 160);
    assert!(
        chunks
            .iter()
            .flat_map(|c| &c.pcm)
            .all(|&sample| sample == 0)
    );
    let before = chunks.len();
    n.finish_segment(|c| {
        chunks.push(c);
        Ok(())
    })
    .unwrap();
    assert_eq!(chunks.len(), before);
}

#[test]
fn resumed_segments_keep_identity_and_exact_global_fractional_count() {
    let f = format(44_100, 1, 16, 16, Encoding::Pcm, 0);
    let mut n = Normalizer::new(f, input(1)).unwrap();
    let mut order = AudioOrder::new(input(1)).unwrap();
    for _ in 0..100 {
        n.push(&[0; 258], 129, false, |c| order.accept(&c)).unwrap();
        n.finish_segment(|c| order.accept(&c)).unwrap();
    }
    assert_eq!(n.emitted_samples(), 12900 * 16000 / 44100);
    let mut fresh = Normalizer::new(f, input(2)).unwrap();
    let mut chunks = Vec::new();
    fresh
        .push(&[0; 258], 129, false, |c| {
            chunks.push(c);
            Ok(())
        })
        .unwrap();
    fresh
        .finish_segment(|c| {
            chunks.push(c);
            Ok(())
        })
        .unwrap();
    assert_eq!(chunks[0].input, input(2));
    assert_eq!((chunks[0].seq, chunks[0].start_sample), (0, 0));
}

#[test]
fn clipping_quantization_and_publication_failure_are_explicit() {
    let mut n = Normalizer::new(format(16_000, 1, 32, 32, Encoding::Float, 0), input(1)).unwrap();
    let data = [2_f32, -2.0, 0.5, -0.5]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    let mut output = Vec::new();
    n.push(&data, 4, false, |c| {
        output.extend(c.pcm);
        Ok(())
    })
    .unwrap();
    assert_eq!(output, vec![32767, -32768, 16384, -16384]);
    assert_eq!(
        n.push(&data, 4, false, |_| Err(ErrorCode::Overloaded)),
        Err(ErrorCode::Overloaded)
    );
    assert_eq!(n.emitted_samples(), 4);
}
