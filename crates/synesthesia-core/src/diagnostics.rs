//! One shared bounded sink for application and owned-engine safe diagnostics.
use crate::ErrorCode;
use serde::Serialize;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const LOG_BUDGET_BYTES: u64 = 20 * 1024 * 1024;
pub const LOG_SEGMENT_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_RECORD_BYTES: usize = 1024;
const SEGMENTS: usize = 5;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Component {
    Application,
    Asr,
    Translation,
    Audio,
    Network,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    Detecting,
    Loading,
    Ready,
    Started,
    Stopped,
    Failed,
    Overloaded,
    Released,
    Measurement,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    Cpu,
    Vulkan,
}

/// Deliberately has no String, arbitrary payload, format argument or foreign error.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Diagnostic {
    pub timestamp_ms: u64,
    pub component: Component,
    pub kind: DiagnosticKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<ErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<Backend>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queued_samples: Option<u64>,
}

pub struct DiagnosticLog {
    directory: PathBuf,
    _lock: File,
    segment_bytes: u64,
}
fn storage<T>(result: std::io::Result<T>) -> Result<T, ErrorCode> {
    result.map_err(|_| ErrorCode::StorageUnavailable)
}
impl DiagnosticLog {
    /// The directory is exclusively owned by this sink; engines must use it too.
    pub fn open(directory: &Path) -> Result<Self, ErrorCode> {
        Self::open_with_segment(directory, LOG_SEGMENT_BYTES)
    }
    fn open_with_segment(directory: &Path, segment_bytes: u64) -> Result<Self, ErrorCode> {
        storage(fs::create_dir_all(directory))?;
        let lock_path = directory.join("writer.lock");
        if fs::symlink_metadata(&lock_path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(ErrorCode::StorageUnavailable);
        }
        let lock = storage(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(lock_path),
        )?;
        lock.try_lock().map_err(|_| ErrorCode::StorageUnavailable)?;
        // Lock metadata is not diagnostic content and must never retain bytes
        // outside the aggregate segment budget. Only truncate after ownership.
        storage(lock.set_len(0))?;
        let log = Self {
            directory: directory.to_path_buf(),
            _lock: lock,
            segment_bytes,
        };
        log.check_owned_files()?;
        log.reconcile()?;
        Ok(log)
    }
    fn path(&self, index: usize) -> PathBuf {
        self.directory.join(format!("diagnostics.{index}.jsonl"))
    }
    fn check_owned_files(&self) -> Result<(), ErrorCode> {
        for entry in storage(fs::read_dir(&self.directory))? {
            let entry = storage(entry)?;
            let name = entry.file_name();
            let allowed = name == "writer.lock"
                || (0..SEGMENTS).any(|i| name == format!("diagnostics.{i}.jsonl").as_str());
            if !allowed || !storage(entry.file_type())?.is_file() {
                return Err(ErrorCode::StorageUnavailable);
            }
        }
        Ok(())
    }
    fn length(&self, index: usize) -> Result<u64, ErrorCode> {
        match fs::metadata(self.path(index)) {
            Ok(metadata) => Ok(metadata.len()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
            Err(_) => Err(ErrorCode::StorageUnavailable),
        }
    }
    fn remove(&self, index: usize) -> Result<(), ErrorCode> {
        match fs::remove_file(self.path(index)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(ErrorCode::StorageUnavailable),
        }
    }
    fn reconcile(&self) -> Result<(), ErrorCode> {
        // Recovery from a previous interrupted write never retains oversized files.
        for index in (0..SEGMENTS).rev() {
            if self.length(index)? > self.segment_bytes {
                self.remove(index)?;
            }
        }
        // Strip a partial last JSONL record after an interrupted write.
        if self.length(0)? > 0 {
            let mut file = storage(OpenOptions::new().read(true).write(true).open(self.path(0)))?;
            let mut bytes = Vec::new();
            storage(file.read_to_end(&mut bytes))?;
            let length = bytes
                .iter()
                .rposition(|b| *b == b'\n')
                .map_or(0, |position| position + 1);
            storage(file.set_len(length as u64))?;
        }
        Ok(())
    }
    fn rotate(&self) -> Result<(), ErrorCode> {
        self.remove(SEGMENTS - 1)?;
        for index in (0..SEGMENTS - 1).rev() {
            if self.path(index).exists() {
                storage(fs::rename(self.path(index), self.path(index + 1)))?;
            }
        }
        Ok(())
    }
    pub fn total_bytes(&self) -> Result<u64, ErrorCode> {
        (0..SEGMENTS).try_fold(0u64, |total, i| {
            total
                .checked_add(self.length(i)?)
                .ok_or(ErrorCode::Overflow)
        })
    }
    pub fn write(&mut self, diagnostic: Diagnostic) -> Result<(), ErrorCode> {
        let mut bytes = serde_json::to_vec(&diagnostic).map_err(|_| ErrorCode::RecordTooLarge)?;
        bytes.push(b'\n');
        self.append_record(&bytes)
    }
    fn append_record(&mut self, bytes: &[u8]) -> Result<(), ErrorCode> {
        if bytes.len() > MAX_RECORD_BYTES || bytes.len() as u64 > self.segment_bytes {
            return Err(ErrorCode::RecordTooLarge);
        }
        if self
            .length(0)?
            .checked_add(bytes.len() as u64)
            .ok_or(ErrorCode::Overflow)?
            > self.segment_bytes
        {
            self.rotate()?;
        }
        let mut file = storage(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.path(0)),
        )?;
        storage(file.write_all(bytes))?;
        storage(file.flush())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "synesthesia-log-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn record(n: u64) -> Diagnostic {
        Diagnostic {
            timestamp_ms: n,
            component: Component::Asr,
            kind: DiagnosticKind::Measurement,
            code: None,
            backend: Some(Backend::Vulkan),
            vendor_id: Some(4098),
            device_id: Some(30032),
            elapsed_ms: Some(n),
            queued_samples: Some(0),
        }
    }
    #[test]
    fn rotation_keeps_newest_and_restart_enforces_budget() {
        let temp = Temp::new();
        let mut log = DiagnosticLog::open_with_segment(&temp.0, 512).unwrap();
        for i in 0..100 {
            log.write(record(i)).unwrap();
            assert!(log.total_bytes().unwrap() <= 5 * 512);
        }
        assert!(fs::read_to_string(log.path(0)).unwrap().contains("99"));
        assert!(
            !fs::read_to_string(log.path(4))
                .unwrap()
                .contains("\"timestamp_ms\":0,")
        );
        drop(log);
        fs::write(temp.0.join("diagnostics.0.jsonl"), "x".repeat(1024)).unwrap();
        let log = DiagnosticLog::open_with_segment(&temp.0, 512).unwrap();
        assert!(log.total_bytes().unwrap() <= 5 * 512);
    }
    #[test]
    fn rejects_oversized_records_and_concurrent_or_foreign_writers() {
        let temp = Temp::new();
        let mut log = DiagnosticLog::open(&temp.0).unwrap();
        assert_eq!(
            log.append_record(&vec![b'x'; MAX_RECORD_BYTES + 1]),
            Err(ErrorCode::RecordTooLarge)
        );
        assert!(DiagnosticLog::open(&temp.0).is_err());
        assert_eq!(log.total_bytes().unwrap(), 0);
        drop(log);
        fs::write(temp.0.join("engine.log"), "content must not be logged").unwrap();
        assert!(DiagnosticLog::open(&temp.0).is_err());
    }
    #[test]
    fn restart_truncates_partial_record_and_io_errors_are_safe() {
        let temp = Temp::new();
        fs::write(temp.0.join("diagnostics.0.jsonl"), "{}\nunfinished").unwrap();
        let mut log = DiagnosticLog::open(&temp.0).unwrap();
        assert_eq!(log.total_bytes().unwrap(), 3);
        fs::remove_file(log.path(0)).unwrap();
        fs::create_dir(log.path(0)).unwrap();
        assert_eq!(log.write(record(1)), Err(ErrorCode::StorageUnavailable));
        assert!(
            DiagnosticLog::open(&temp.0.join("diagnostics.0.jsonl").join("../writer.lock"))
                .is_err()
        );
    }
    #[test]
    fn schema_only_exposes_whitelisted_fields() {
        let value = serde_json::to_value(record(1)).unwrap();
        assert!(value.as_object().unwrap().keys().all(|key| {
            [
                "timestamp_ms",
                "component",
                "kind",
                "code",
                "backend",
                "vendor_id",
                "device_id",
                "elapsed_ms",
                "queued_samples",
            ]
            .contains(&key.as_str())
        }));
        assert_eq!(LOG_SEGMENT_BYTES * SEGMENTS as u64, LOG_BUDGET_BYTES);
    }

    #[test]
    fn actual_twenty_mib_boundary_rotates_before_append() {
        use std::io::{Seek, SeekFrom};
        let temp = Temp::new();
        for index in 0..SEGMENTS {
            let mut file = File::create(temp.0.join(format!("diagnostics.{index}.jsonl"))).unwrap();
            file.set_len(LOG_SEGMENT_BYTES).unwrap();
            file.seek(SeekFrom::End(-1)).unwrap();
            file.write_all(b"\n").unwrap();
        }
        let mut log = DiagnosticLog::open(&temp.0).unwrap();
        assert_eq!(log.total_bytes().unwrap(), LOG_BUDGET_BYTES);
        log.write(record(123)).unwrap();
        assert!(log.total_bytes().unwrap() < LOG_BUDGET_BYTES);
        assert!(fs::read_to_string(log.path(0)).unwrap().contains("123"));
    }

    #[test]
    fn lock_file_cannot_retain_bytes_outside_the_disk_budget() {
        let temp = Temp::new();
        let lock_path = temp.0.join("writer.lock");
        File::create(&lock_path)
            .unwrap()
            .set_len(LOG_BUDGET_BYTES + 1)
            .unwrap();
        let log = DiagnosticLog::open(&temp.0).unwrap();
        assert_eq!(fs::metadata(&lock_path).unwrap().len(), 0);
        assert_eq!(log.total_bytes().unwrap(), 0);
        // Another sink cannot alter a lock held by this owner.
        assert!(DiagnosticLog::open(&temp.0).is_err());
    }
}
