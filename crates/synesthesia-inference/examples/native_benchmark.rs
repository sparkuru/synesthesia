//! Private fixed-fixture benchmark. Generated content stays in bounded memory.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use synesthesia_core::ErrorCode;
use synesthesia_core::config::{Config, Role, RuntimeRoot, SourceLanguage};
use synesthesia_core::domain::{AudioChunk, InputId, Phase, PublishedEvent, TranslationResult};
use synesthesia_core::session::ComputeAdapter;
use synesthesia_inference::{NativeInference, NativeTools};

struct Results {
    drafts: u64,
    finals: u64,
    translations: u64,
    failed: u64,
    transcript: String,
    first_drafts: BTreeMap<u64, ()>,
    final_at: BTreeMap<u64, Instant>,
    draft_delay: Vec<u64>,
    final_delay: Vec<u64>,
    translation_delay: Vec<u64>,
    max_queue: u64,
}
impl Results {
    fn new() -> Self {
        Self {
            drafts: 0,
            finals: 0,
            translations: 0,
            failed: 0,
            transcript: String::new(),
            first_drafts: BTreeMap::new(),
            final_at: BTreeMap::new(),
            draft_delay: Vec::new(),
            final_delay: Vec::new(),
            translation_delay: Vec::new(),
            max_queue: 0,
        }
    }
    fn drain(
        &mut self,
        inference: &mut NativeInference,
        started: Instant,
    ) -> Result<(), ErrorCode> {
        self.max_queue = self.max_queue.max(inference.metrics().queued_samples);
        while let Some(event) = inference.next_event()? {
            let now = Instant::now();
            match event {
                PublishedEvent::Source(source) => {
                    let id = source.id.utterance_id;
                    if source.phase == Phase::Draft {
                        self.drafts += 1;
                        if self.first_drafts.insert(id, ()).is_none() {
                            let from = started
                                + Duration::from_secs_f64(source.start_sample as f64 / 16000.0);
                            self.draft_delay
                                .push(now.saturating_duration_since(from).as_millis() as u64);
                        }
                    } else {
                        self.finals += 1;
                        if self.transcript.len() + source.text.len() + 1 > 512 * 1024 {
                            return Err(ErrorCode::Overloaded);
                        }
                        self.transcript.push_str(&source.text);
                        self.transcript.push(' ');
                        self.final_at.insert(id, now);
                        let from =
                            started + Duration::from_secs_f64(source.end_sample as f64 / 16000.0);
                        self.final_delay
                            .push(now.saturating_duration_since(from).as_millis() as u64);
                        inference.translate(&source)?;
                    }
                }
                PublishedEvent::Translation(translation) => {
                    let from = self
                        .final_at
                        .remove(&translation.id.utterance_id)
                        .ok_or(ErrorCode::InvalidEvent)?;
                    self.translation_delay
                        .push(now.duration_since(from).as_millis() as u64);
                    match translation.outcome {
                        TranslationResult::Complete { .. } => self.translations += 1,
                        TranslationResult::Failed { .. } => self.failed += 1,
                    }
                }
            }
            if self.first_drafts.len() > 2048 || self.final_at.len() > 4 {
                return Err(ErrorCode::Overloaded);
            }
        }
        Ok(())
    }
}
fn summary(values: &mut [u64]) -> Value {
    values.sort_unstable();
    if values.is_empty() {
        return json!({"count": 0});
    }
    json!({"count": values.len(), "p50_ms": values[(values.len() - 1) / 2], "p95_ms": values[((values.len() * 95).div_ceil(100) - 1).min(values.len() - 1)], "max_ms": values[values.len() - 1]})
}
fn units(text: &str, language: SourceLanguage) -> Vec<String> {
    let normalized: String = text
        .chars()
        .flat_map(char::to_lowercase)
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    match language {
        SourceLanguage::English => normalized.split_whitespace().map(str::to_owned).collect(),
        SourceLanguage::Japanese => normalized
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_string())
            .collect(),
    }
}
fn distance(reference: &[String], actual: &[String]) -> usize {
    let mut row: Vec<_> = (0..=actual.len()).collect();
    for (i, expected) in reference.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, observed) in actual.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (diagonal + usize::from(expected != observed))
                .min(above + 1)
                .min(row[j] + 1);
            diagonal = above;
        }
    }
    row[actual.len()]
}
fn run() -> Result<(), ErrorCode> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 6 {
        return Err(ErrorCode::InvalidConfig);
    }
    let root = RuntimeRoot::new(args[1].clone().into()).map_err(|e| e.code)?;
    let Config::Full {
        inference: config,
        source,
        ..
    } = root
        .load(&PathBuf::from(&args[0]), Role::Full)
        .map_err(|e| e.code)?
    else {
        return Err(ErrorCode::WrongRole);
    };
    let mut manifest_bytes = Vec::new();
    File::open(&args[5])
        .map_err(|_| ErrorCode::InvalidPath)?
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut manifest_bytes)
        .map_err(|_| ErrorCode::InvalidConfig)?;
    if manifest_bytes.len() > 2 * 1024 * 1024 {
        return Err(ErrorCode::InvalidConfig);
    }
    let manifest: Value =
        serde_json::from_slice(&manifest_bytes).map_err(|_| ErrorCode::InvalidConfig)?;
    let expected_language = match source {
        SourceLanguage::English => "en",
        SourceLanguage::Japanese => "ja",
    };
    if manifest["version"] != 1
        || manifest["language"] != expected_language
        || manifest["duration_samples"] != 9_600_000
        || manifest["sample_rate"] != 16000
        || manifest["channels"] != 1
        || manifest["format"] != "pcm_s16le"
    {
        return Err(ErrorCode::InvalidConfig);
    }
    let annotations = manifest["annotations"]
        .as_array()
        .ok_or(ErrorCode::InvalidConfig)?;
    if annotations.is_empty() || annotations.len() > 500 {
        return Err(ErrorCode::InvalidConfig);
    }
    let mut reference = String::new();
    for annotation in annotations {
        let text = annotation["reference"]
            .as_str()
            .ok_or(ErrorCode::InvalidConfig)?;
        if text.len() > 8192 || reference.len() + text.len() > 512 * 1024 {
            return Err(ErrorCode::InvalidConfig);
        }
        reference.push_str(text);
        reference.push(' ');
    }
    let mut file = File::open(&args[4]).map_err(|_| ErrorCode::InvalidPath)?;
    if file.metadata().map_err(|_| ErrorCode::InvalidPath)?.len() != 19_200_000 {
        return Err(ErrorCode::InvalidAudio);
    }
    let mut digest = Sha256::new();
    let mut block = [0; 6400];
    loop {
        let n = file.read(&mut block).map_err(|_| ErrorCode::InvalidAudio)?;
        if n == 0 {
            break;
        }
        digest.update(&block[..n]);
    }
    if manifest["pcm_sha256"].as_str() != Some(format!("{:x}", digest.finalize()).as_str()) {
        return Err(ErrorCode::InvalidAudio);
    }
    let mut inference = NativeInference::new(
        config,
        NativeTools {
            asr_helper: args[2].clone().into(),
            translation_helper: args[3].clone().into(),
        },
        source,
    )?;
    inference.detect()?;
    inference.load()?;
    let input = InputId {
        session_id: 1,
        input_generation: 1,
    };
    inference.start_input(input, source)?;
    let mut file = File::open(&args[4]).map_err(|_| ErrorCode::InvalidPath)?;
    let started = Instant::now();
    let mut results = Results::new();
    for seq in 0..3000 {
        file.read_exact(&mut block)
            .map_err(|_| ErrorCode::InvalidAudio)?;
        let frame = AudioChunk {
            input,
            seq,
            start_sample: seq * 3200,
            pcm: block
                .chunks_exact(2)
                .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
                .collect(),
        };
        inference.accept_audio(&frame)?;
        let until = started + Duration::from_millis((seq + 1) * 200);
        while Instant::now() < until {
            results.drain(&mut inference, started)?;
            thread::sleep(Duration::from_millis(5));
        }
    }
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut finalized = false;
    while Instant::now() < deadline {
        if !finalized && inference.metrics().queued_samples == 0 {
            inference.finish_utterance(input, 9_600_000)?;
            finalized = true;
        }
        results.drain(&mut inference, started)?;
        if finalized && !inference.has_pending_work() {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    let reference_units = units(&reference, source);
    let actual_units = units(&results.transcript, source);
    if reference_units.len() > 20000
        || actual_units.len() > 20000
        || reference_units.len() * actual_units.len() > 100_000_000
    {
        return Err(ErrorCode::Overloaded);
    }
    let edit_distance = distance(&reference_units, &actual_units);
    let metrics = inference.metrics();
    let incomplete = inference.has_pending_work();
    inference.cancel(input)?;
    inference.shutdown()?;
    println!(
        "{}",
        json!({
            "language": expected_language, "duration_samples": 9600000,
            "expected_corpus_clips": annotations.len(), "pending_at_deadline": incomplete,
            "elapsed_ms": started.elapsed().as_millis(), "drafts": results.drafts,
            "finals": results.finals, "translations": results.translations,
            "failed_translations": results.failed, "max_queued_samples": results.max_queue,
            "asr_graphs": metrics.asr_graphs, "translation_graphs": metrics.translation_graphs,
            "reference_units": reference_units.len(), "edit_distance": edit_distance,
            "unit": if source == SourceLanguage::English { "normalized_words" } else { "normalized_characters" },
            "draft_segment_latency": summary(&mut results.draft_delay),
            "final_segment_latency": summary(&mut results.final_delay),
            "translation_from_final_latency": summary(&mut results.translation_delay),
            "latency_scope": "Energy-segment sample boundaries, not manually annotated phonetic sentence boundaries",
            "translation_quality": "requires human review; no generated content persisted"
        })
    );
    if results.finals == 0 || results.failed != 0 || !results.final_at.is_empty() || incomplete {
        return Err(ErrorCode::InferenceFailed);
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("native benchmark failed: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scoring_counts_missing_and_wrong_content() {
        let reference = units("Good morning, world!", SourceLanguage::English);
        assert_eq!(
            distance(&reference, &units("good world", SourceLanguage::English)),
            1
        );
        assert_eq!(distance(&reference, &[]), 3);
        assert_eq!(
            distance(
                &units("今日は晴れ。", SourceLanguage::Japanese),
                &units("今日は雨。", SourceLanguage::Japanese)
            ),
            2
        );
    }
}
