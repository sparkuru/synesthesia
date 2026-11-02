//! Finite numeric-only capture probe. Controlled playback remains in memory.
use std::{
    thread,
    time::{Duration, Instant},
};
use synesthesia_audio::{CaptureHandle, ControlledPlayback, FailureKind, render_devices};
use synesthesia_core::{
    config::CaptureConfig,
    domain::{AudioOrder, InputId},
};

#[derive(Default)]
struct ToneMeasure {
    count: u64,
    sin: [f64; 2],
    cos: [f64; 2],
}
impl ToneMeasure {
    fn push(&mut self, sample: i16, cursor: u64) {
        for (index, frequency) in [697.0, 941.0].into_iter().enumerate() {
            let phase = 2.0 * std::f64::consts::PI * frequency * cursor as f64 / 16000.0;
            let value = f64::from(sample) / 32768.0;
            self.sin[index] += value * phase.sin();
            self.cos[index] += value * phase.cos();
        }
        self.count += 1;
    }
    fn amplitude(&self, index: usize) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.sin[index].hypot(self.cos[index]) * 2.0 / self.count as f64
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut seconds = 6_u64;
    let mut expect_active = false;
    let mut overload = false;
    let mut locked = false;
    let mut controlled = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seconds" => {
                seconds = args.next().ok_or("missing duration")?.parse()?;
            }
            "--expect-active" => expect_active = true,
            "--overload" => overload = true,
            "--locked-default" => locked = true,
            "--controlled-playback" => controlled = true,
            _ => return Err("unsupported diagnostic argument".into()),
        }
    }
    if !(1..=30).contains(&seconds) || (expect_active && overload) {
        return Err("invalid duration or incompatible diagnostic assertions".into());
    }
    let devices = render_devices()?;
    println!(
        "render_devices={} defaults={}",
        devices.len(),
        devices.iter().filter(|d| d.is_default).count()
    );
    let default = devices
        .iter()
        .find(|d| d.is_default)
        .ok_or("no default render endpoint")?;
    let invalid = CaptureHandle::start(
        CaptureConfig::Device {
            device_id: "controlled-missing-render-id".into(),
        },
        InputId {
            session_id: 700,
            input_generation: 1,
        },
    );
    if invalid.is_ok() {
        return Err("missing locked endpoint unexpectedly selected another device".into());
    }
    println!("missing_locked_device_rejected=1");
    let mut playback: Option<ControlledPlayback> = None;
    let mut playback_id = String::new();
    for generation in 1..=2 {
        let input = InputId {
            session_id: 701,
            input_generation: generation,
        };
        let selection = if locked {
            CaptureConfig::Device {
                device_id: default.id.clone(),
            }
        } else {
            CaptureConfig::FollowDefault {}
        };
        let mut capture = CaptureHandle::start(selection, input)?;
        if controlled {
            if playback_id.is_empty() {
                playback_id = capture.device.id.clone();
                playback = Some(ControlledPlayback::start(&capture.device)?);
            } else {
                if playback_id != capture.device.id {
                    return Err("selected endpoint changed during controlled probe".into());
                }
                playback.as_mut().unwrap().resume()?;
            }
        }
        println!(
            "generation={} native_rate={} channels={} container_bits={} valid_bits={} channel_mask={} selected_default={}",
            generation,
            capture.format.rate,
            capture.format.channels,
            capture.format.container_bits,
            capture.format.valid_bits,
            capture.format.channel_mask,
            u8::from(capture.device.is_default)
        );
        let mut order = AudioOrder::new(input)?;
        let start = Instant::now();
        let mut last_report = Instant::now();
        let mut received = 0_u64;
        let mut nonzero = 0_u64;
        let mut peak = 0_i32;
        let mut max_idle = 0;
        let mut paused = false;
        let mut idle_sample: Option<(u64, u64)> = None;
        let mut idle_verified = false;
        let mut active_tones = ToneMeasure::default();
        let mut paused_tones = ToneMeasure::default();
        if overload {
            thread::sleep(Duration::from_millis(700));
        }
        while start.elapsed() < Duration::from_secs(seconds) {
            if controlled
                && !overload
                && !paused
                && start.elapsed() >= Duration::from_millis(seconds * 650)
            {
                playback.as_mut().unwrap().pause()?;
                paused = true;
                println!("generation={} renderer_paused=1", generation);
            }
            match capture.next_chunk() {
                Ok(Some(frame)) => {
                    order.accept(&frame)?;
                    if received == 0 && (frame.seq != 0 || frame.start_sample != 0) {
                        return Err("fresh input did not reset audio timeline".into());
                    }
                    received += frame.pcm.len() as u64;
                    for (offset, sample) in frame.pcm.into_iter().enumerate() {
                        if controlled {
                            let measure = if paused {
                                &mut paused_tones
                            } else {
                                &mut active_tones
                            };
                            measure.push(sample, frame.start_sample + offset as u64);
                        }
                        let amplitude = i32::from(sample).abs();
                        peak = peak.max(amplitude);
                        if amplitude > 16 {
                            nonzero += 1;
                        }
                    }
                }
                Ok(None) => thread::sleep(Duration::from_millis(2)),
                Err(error) if overload && error.kind == FailureKind::Overloaded => break,
                Err(error) => {
                    capture.stop();
                    return Err(error.into());
                }
            }
            let observed = capture.observation();
            if paused
                && observed.no_packet_ms >= 250
                && observed.idle_tail_drained
                && observed.raw_queued_frames == 0
            {
                if let Some((cursor, packets)) = idle_sample
                    && packets == observed.packets
                {
                    if cursor != observed.emitted_samples {
                        return Err("absent packets manufactured captured samples".into());
                    }
                    idle_verified = true;
                }
                idle_sample = Some((observed.emitted_samples, observed.packets));
            }
            max_idle = max_idle.max(observed.no_packet_ms);
            if last_report.elapsed() >= Duration::from_millis(500) {
                println!(
                    "generation={} elapsed_ms={} packets={} native_frames={} emitted_samples={} received_samples={} silent_packets={} timestamp_errors={} no_packet_ms={} queued_samples={} raw_frames={} tail_drained={}",
                    generation,
                    observed.elapsed_ms,
                    observed.packets,
                    observed.native_frames,
                    observed.emitted_samples,
                    received,
                    observed.silent_packets,
                    observed.timestamp_errors,
                    observed.no_packet_ms,
                    observed.queued_samples,
                    observed.raw_queued_frames,
                    u8::from(observed.idle_tail_drained)
                );
                last_report = Instant::now();
            }
        }
        let before = capture.observation();
        if let Some(playback) = &mut playback
            && !paused
        {
            playback.pause()?;
        }
        let stopped_at = Instant::now();
        let stopped = capture.stop();
        let stop_ms = stopped_at.elapsed().as_millis();
        let cleared = match capture.next_chunk() {
            Ok(None) => true,
            Err(error) if overload && error.kind == FailureKind::Overloaded => true,
            _ => false,
        };
        if !stopped.capture_finished
            || !stopped.normalizer_finished
            || stopped.queued_samples != 0
            || !cleared
        {
            return Err("stop did not quiesce and clear workers".into());
        }
        if overload && stopped.failure.map(|e| e.kind) != Some(FailureKind::Overloaded) {
            return Err("controlled slow consumer did not fail explicitly".into());
        }
        if expect_active && (nonzero < 100 || peak < 100) {
            return Err("no meaningful output activity captured".into());
        }
        if idle_verified
            && before.idle_tail_drained
            && before.raw_queued_frames == 0
            && before.emitted_samples
                != before.native_frames * 16000 / u64::from(capture.format.rate)
        {
            return Err("drained native sample count differs from normalized timeline".into());
        }
        if controlled {
            if !overload && (active_tones.amplitude(0) < 0.005 || active_tones.amplitude(1) < 0.005)
            {
                return Err("controlled stereo tones missing from normalized capture".into());
            }
            println!(
                "generation={} tone697_milli={} tone941_milli={} paused697_milli={} paused941_milli={}",
                generation,
                (active_tones.amplitude(0) * 1000.0).round(),
                (active_tones.amplitude(1) * 1000.0).round(),
                (paused_tones.amplitude(0) * 1000.0).round(),
                (paused_tones.amplitude(1) * 1000.0).round()
            );
            println!(
                "generation={} controlled_playback=1 same_endpoint=1 resumed={} no_packet_assertion_available={}",
                generation,
                u8::from(generation == 2),
                u8::from(idle_verified)
            );
        }
        println!(
            "generation={} stopped=1 stop_ms={} received_samples={} nonzero_samples={} peak={} max_no_packet_ms={} discarded_queued_samples={} overloaded={}",
            generation,
            stop_ms,
            received,
            nonzero,
            peak,
            max_idle,
            before.queued_samples,
            u8::from(overload)
        );
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("audio_probe_failed: {error}");
        std::process::exit(2);
    }
}
