//! Controlled compatibility/lifecycle experiment; emits numeric evidence only.
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use synesthesia_core::ErrorCode;
use synesthesia_core::config::{Config, Role, RuntimeRoot, SourceLanguage};
use synesthesia_core::domain::{
    AudioChunk, InputId, Phase, PublishedEvent, SourceEvent, TranslationResult, UtteranceId,
};
use synesthesia_core::session::ComputeAdapter;
use synesthesia_inference::{NativeInference, NativeTools};

fn read_pcm(
    file: &mut File,
    input: InputId,
    seq: u64,
    sample: u64,
) -> Result<Option<AudioChunk>, ErrorCode> {
    let mut bytes = [0; 6400];
    let mut count = 0;
    while count < bytes.len() {
        let n = file
            .read(&mut bytes[count..])
            .map_err(|_| ErrorCode::InvalidAudio)?;
        if n == 0 {
            break;
        }
        count += n;
    }
    if count == 0 {
        return Ok(None);
    }
    if count % 2 != 0 {
        return Err(ErrorCode::InvalidAudio);
    }
    let pcm = bytes[..count]
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    Ok(Some(AudioChunk {
        input,
        seq,
        start_sample: sample,
        pcm,
    }))
}
fn translation_request(input: InputId, language: SourceLanguage) -> SourceEvent {
    SourceEvent {
        id: UtteranceId {
            input,
            utterance_id: 1,
        },
        revision: 1,
        phase: Phase::Final,
        text: match language {
            SourceLanguage::English => "The weather is sunny today.".into(),
            SourceLanguage::Japanese => "今日は晴れています。".into(),
        },
        start_sample: 0,
        end_sample: 1,
    }
}
fn run() -> Result<(), ErrorCode> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 6 {
        return Err(ErrorCode::InvalidConfig);
    }
    let root = RuntimeRoot::new(PathBuf::from(&args[1])).map_err(|error| error.code)?;
    let Config::Full {
        inference: inference_config,
        source,
        ..
    } = root
        .load(&PathBuf::from(&args[0]), Role::Full)
        .map_err(|error| error.code)?
    else {
        return Err(ErrorCode::WrongRole);
    };
    let mut inference = NativeInference::new(
        inference_config,
        NativeTools {
            asr_helper: args[2].clone().into(),
            translation_helper: args[3].clone().into(),
        },
        source,
    )?;
    let started = Instant::now();
    inference.detect()?;
    inference.load()?;
    let load_millis = started.elapsed().as_millis();
    let pids = inference.process_ids().ok_or(ErrorCode::InvalidState)?;
    println!(
        "{{\"phase\":\"loaded\",\"elapsed_ms\":{load_millis},\"asr_pid\":{},\"translation_pid\":{}}}",
        pids.0, pids.1
    );

    // Stop during both owned computations, then probe the same loaded processes.
    for stage in 2..=4 {
        let input = InputId {
            session_id: u64::from(stage - 1),
            input_generation: 1,
        };
        inference.start_input(input, SourceLanguage::English)?;
        let mut file = File::open(&args[4]).map_err(|_| ErrorCode::InvalidPath)?;
        let mut sample = 0;
        for seq in 0..30 {
            let frame = read_pcm(&mut file, input, seq, sample)?.ok_or(ErrorCode::InvalidAudio)?;
            sample += frame.pcm.len() as u64;
            inference.accept_audio(&frame)?;
        }
        let mut submitted_final = inference.metrics().queued_samples == 0;
        if submitted_final {
            inference.finish_utterance(input, sample)?;
        }
        inference.translate(&translation_request(input, SourceLanguage::English))?;
        let deadline = Instant::now() + Duration::from_secs(30);
        let reached = loop {
            let progress = inference.probe_asr_work()?;
            if progress.stage == stage && progress.in_graph && progress.active_job != 0 {
                break progress;
            }
            while inference.next_event()?.is_some() {}
            if !submitted_final && inference.metrics().queued_samples == 0 {
                inference.finish_utterance(input, sample)?;
                submitted_final = true;
            }
            if Instant::now() >= deadline {
                return Err(ErrorCode::InferenceFailed);
            }
            thread::sleep(Duration::from_millis(1));
        };
        let cancel_start = Instant::now();
        inference.cancel(input)?;
        let cancelled = inference.probe_idle_residency()?;
        thread::sleep(Duration::from_millis(200));
        let idle = inference.probe_idle_residency()?;
        let stopped = inference.probe_asr_work()?;
        if cancelled.asr_graphs != idle.asr_graphs
            || cancelled.translation_graphs != idle.translation_graphs
            || inference.process_ids() != Some(pids)
            || idle.model_loads != 1
            || stopped.active_job != 0
            || stopped.in_graph
            || stopped.stage != 0
        {
            return Err(ErrorCode::InvalidState);
        }
        println!(
            "{{\"phase\":\"cancelled_retained\",\"observed_asr_stage\":{stage},\"observed_graphs\":{},\"observed_job\":{},\"elapsed_ms\":{},\"asr_graphs\":{},\"translation_graphs\":{},\"models_loaded_once\":true,\"idle_graphs_stable\":true}}",
            reached.graphs,
            reached.active_job,
            cancel_start.elapsed().as_millis(),
            idle.asr_graphs,
            idle.translation_graphs
        );
    }

    for (session, language, fixture) in [
        (4, SourceLanguage::English, &args[4]),
        (5, SourceLanguage::Japanese, &args[5]),
    ] {
        let input = InputId {
            session_id: session,
            input_generation: 1,
        };
        inference.start_input(input, language)?;
        let mut file = File::open(fixture).map_err(|_| ErrorCode::InvalidPath)?;
        let mut sample = 0;
        let mut seq = 0;
        let mut drafts = 0;
        let mut finals = 0;
        let mut translated = 0;
        let mut failed_translations = 0;
        let mut submitted_final = false;
        let start = Instant::now();
        loop {
            let Some(frame) = read_pcm(&mut file, input, seq, sample)? else {
                break;
            };
            sample += frame.pcm.len() as u64;
            seq += 1;
            inference.accept_audio(&frame)?;
            let until = start + Duration::from_secs_f64(sample as f64 / 16000.0);
            while Instant::now() < until {
                drain(
                    &mut inference,
                    &mut drafts,
                    &mut finals,
                    &mut translated,
                    &mut failed_translations,
                )?;
                thread::sleep(Duration::from_millis(5));
            }
            if sample >= 16000 * 15 {
                break;
            }
        }
        let deadline = Instant::now() + Duration::from_secs(60);
        while Instant::now() < deadline {
            if !submitted_final && inference.metrics().queued_samples == 0 {
                inference.finish_utterance(input, sample)?;
                submitted_final = true;
            }
            drain(
                &mut inference,
                &mut drafts,
                &mut finals,
                &mut translated,
                &mut failed_translations,
            )?;
            if submitted_final
                && !inference.has_pending_work()
                && finals > 0
                && translated + failed_translations == finals
            {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        if finals == 0 || translated == 0 || failed_translations != 0 {
            return Err(ErrorCode::InferenceFailed);
        }
        inference.cancel(input)?;
        if inference.process_ids() != Some(pids) {
            return Err(ErrorCode::InvalidState);
        }
        println!(
            "{{\"phase\":\"speech\",\"session\":{session},\"samples\":{sample},\"drafts\":{drafts},\"finals\":{finals},\"translations\":{translated},\"elapsed_ms\":{}}}",
            start.elapsed().as_millis()
        );
    }
    inference.shutdown()?;
    println!("{{\"phase\":\"released\",\"owned_processes\":0}}");
    Ok(())
}
fn drain(
    inference: &mut NativeInference,
    drafts: &mut u64,
    finals: &mut u64,
    translated: &mut u64,
    failed: &mut u64,
) -> Result<(), ErrorCode> {
    while let Some(event) = inference.next_event()? {
        match event {
            PublishedEvent::Source(source) => {
                if source.phase == Phase::Draft {
                    *drafts += 1;
                } else {
                    *finals += 1;
                    inference.translate(&source)?;
                }
            }
            PublishedEvent::Translation(translation) => match translation.outcome {
                TranslationResult::Complete { .. } => *translated += 1,
                TranslationResult::Failed { .. } => *failed += 1,
            },
        }
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("native experiment failed: {error}");
        std::process::exit(2);
    }
}
