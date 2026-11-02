use std::collections::VecDeque;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, TryRecvError, sync_channel};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use synesthesia_core::ErrorCode;
use synesthesia_core::config::ComputeDevice;

const MAGIC: u32 = 0x3149_4e53;
const MAX_OUTPUT: usize = 8_200;
pub(crate) const WAIT: Duration = Duration::from_secs(60);

pub(crate) struct Reply {
    pub kind: u32,
    pub job: u64,
    pub payload: Vec<u8>,
}

/// Only a fixed-length header and bounded payload are ever read from a helper.
fn read_reply(reader: &mut impl Read) -> Result<Reply, ErrorCode> {
    let mut header = [0; 20];
    reader
        .read_exact(&mut header)
        .map_err(|_| ErrorCode::InferenceFailed)?;
    let u32_at = |offset| u32::from_le_bytes(header[offset..offset + 4].try_into().unwrap());
    let kind = u32_at(4);
    let job = u64::from_le_bytes(header[8..16].try_into().unwrap());
    let length = u32_at(16) as usize;
    if u32_at(0) != MAGIC || !(10..=14).contains(&kind) || length > MAX_OUTPUT {
        return Err(ErrorCode::InvalidProtocol);
    }
    let mut payload = vec![0; length];
    reader
        .read_exact(&mut payload)
        .map_err(|_| ErrorCode::InferenceFailed)?;
    Ok(Reply { kind, job, payload })
}

pub(crate) struct NativeProcess {
    child: Child,
    input: Option<ChildStdin>,
    replies: Receiver<Result<Reply, ErrorCode>>,
    reader: Option<JoinHandle<()>>,
    last_job: u64,
    pending: VecDeque<Reply>,
}
#[derive(Debug, Clone, Copy)]
pub struct WorkStatus {
    pub stage: u32,
    pub in_graph: bool,
    pub graphs: u64,
    pub active_job: u64,
}
impl NativeProcess {
    pub fn id(&self) -> u32 {
        self.child.id()
    }
    pub fn spawn(
        executable: &Path,
        model: Option<&Path>,
        device: &ComputeDevice,
    ) -> Result<Self, ErrorCode> {
        if !executable.is_absolute() || !executable.is_file() {
            return Err(ErrorCode::InvalidPath);
        }
        let mut command = Command::new(executable);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .current_dir(executable.parent().ok_or(ErrorCode::InvalidPath)?);
        // Owned native helpers have no content logger. Foreign stderr is discarded
        // directly by the OS, so a giant unbroken log cannot allocate app memory.
        for (name, _) in std::env::vars_os() {
            let name_text = name.to_string_lossy().to_ascii_uppercase();
            if name_text.starts_with("GGML_")
                || name_text.starts_with("CRISP")
                || name_text.starts_with("LLAMA_")
            {
                command.env_remove(name);
            }
        }
        if let Some(model) = model {
            command.arg(model);
        } else {
            command.arg("--probe");
        }
        match device {
            ComputeDevice::Cpu {} => {
                command.arg("cpu");
            }
            ComputeDevice::Vulkan {
                vendor_id,
                device_id,
                device_name,
            } => {
                command.args([
                    "vulkan",
                    &vendor_id.to_string(),
                    &device_id.to_string(),
                    device_name,
                ]);
            }
        }
        let mut child = command.spawn().map_err(|_| ErrorCode::InferenceFailed)?;
        let input = child.stdin.take().ok_or(ErrorCode::InferenceFailed)?;
        let mut output = child.stdout.take().ok_or(ErrorCode::InferenceFailed)?;
        let (sender, replies) = sync_channel(8);
        let reader = thread::spawn(move || {
            loop {
                let reply = read_reply(&mut output);
                let failed = reply.is_err();
                if sender.send(reply).is_err() || failed {
                    return;
                }
            }
        });
        let mut process = Self {
            child,
            input: Some(input),
            replies,
            reader: Some(reader),
            last_job: 0,
            pending: VecDeque::new(),
        };
        let ready = process.receive(Duration::from_secs(180))?;
        process.verify_ready(ready, device)?;
        Ok(process)
    }
    fn verify_ready(&self, reply: Reply, device: &ComputeDevice) -> Result<(), ErrorCode> {
        if reply.kind != 10 || reply.job != 0 || reply.payload.len() != 12 {
            return Err(ErrorCode::InferenceFailed);
        }
        let field = |i| u32::from_le_bytes(reply.payload[i..i + 4].try_into().unwrap());
        let expected = match device {
            ComputeDevice::Cpu {} => (0, 0, 0),
            ComputeDevice::Vulkan {
                vendor_id,
                device_id,
                ..
            } => (1, *vendor_id, *device_id),
        };
        if (field(0), field(4), field(8)) != expected {
            return Err(ErrorCode::InferenceFailed);
        }
        Ok(())
    }
    pub fn submit(&mut self, payload: &[u8]) -> Result<u64, ErrorCode> {
        self.send(1, payload)
    }
    fn send(&mut self, kind: u32, payload: &[u8]) -> Result<u64, ErrorCode> {
        if payload.len() > 320_001 {
            return Err(ErrorCode::Overloaded);
        }
        let job = self.last_job.checked_add(1).ok_or(ErrorCode::Overflow)?;
        let input = self.input.as_mut().ok_or(ErrorCode::InvalidState)?;
        let mut header = Vec::with_capacity(20);
        header.extend(MAGIC.to_le_bytes());
        header.extend(kind.to_le_bytes());
        header.extend(job.to_le_bytes());
        header.extend((payload.len() as u32).to_le_bytes());
        input
            .write_all(&header)
            .and_then(|()| input.write_all(payload))
            .and_then(|()| input.flush())
            .map_err(|_| ErrorCode::InferenceFailed)?;
        self.last_job = job;
        Ok(job)
    }
    pub fn poll(&mut self) -> Result<Option<Reply>, ErrorCode> {
        if let Some(reply) = self.pending.pop_front() {
            return Ok(Some(reply));
        }
        match self.replies.try_recv() {
            Ok(result) => result.map(Some),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(ErrorCode::InferenceFailed),
        }
    }
    fn receive(&mut self, timeout: Duration) -> Result<Reply, ErrorCode> {
        if let Some(reply) = self.pending.pop_front() {
            return Ok(reply);
        }
        self.replies
            .recv_timeout(timeout)
            .map_err(|_| ErrorCode::InferenceFailed)?
    }
    pub fn status(&mut self) -> Result<WorkStatus, ErrorCode> {
        let job = self.send(4, &[])?;
        let deadline = Instant::now() + WAIT;
        loop {
            let reply = self
                .replies
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|_| ErrorCode::InferenceFailed)??;
            if reply.kind == 14 && reply.job == job && reply.payload.len() == 24 {
                let p = &reply.payload;
                let stage = u32::from_le_bytes(p[..4].try_into().unwrap());
                let busy = u32::from_le_bytes(p[4..8].try_into().unwrap());
                if stage > 4 || busy > 1 {
                    return Err(ErrorCode::InvalidProtocol);
                }
                return Ok(WorkStatus {
                    stage,
                    in_graph: busy == 1,
                    graphs: u64::from_le_bytes(p[8..16].try_into().unwrap()),
                    active_job: u64::from_le_bytes(p[16..24].try_into().unwrap()),
                });
            }
            if reply.job >= job || !matches!(reply.kind, 11 | 13) || self.pending.len() >= 8 {
                return Err(ErrorCode::InvalidProtocol);
            }
            self.pending.push_back(reply);
        }
    }
    pub fn cancel(&mut self) -> Result<u64, ErrorCode> {
        let job = self.begin_cancel()?;
        let deadline = Instant::now() + WAIT;
        loop {
            let reply = self.receive(deadline.saturating_duration_since(Instant::now()))?;
            if reply.kind == 12 && reply.job == job && reply.payload.len() == 8 {
                return Ok(u64::from_le_bytes(reply.payload[..8].try_into().unwrap()));
            }
            if reply.job >= job || !matches!(reply.kind, 11..=13) {
                return Err(ErrorCode::InvalidProtocol);
            }
        }
    }
    pub fn begin_cancel(&mut self) -> Result<u64, ErrorCode> {
        self.send(2, &[])
    }
    pub fn shutdown(&mut self) -> Result<(), ErrorCode> {
        let job = self.send(3, &[])?;
        loop {
            let reply = self.receive(WAIT)?;
            if reply.kind == 12 && reply.job == job && reply.payload.len() == 8 {
                break;
            }
            if reply.job >= job || !matches!(reply.kind, 11..=13) {
                return Err(ErrorCode::InvalidProtocol);
            }
        }
        self.input.take();
        self.wait_exit()
    }
    pub fn wait_exit(&mut self) -> Result<(), ErrorCode> {
        let deadline = Instant::now() + WAIT;
        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|_| ErrorCode::InferenceFailed)?
            {
                if status.success() {
                    return Ok(());
                }
                return Err(ErrorCode::InferenceFailed);
            }
            if Instant::now() >= deadline {
                return Err(ErrorCode::InferenceFailed);
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}
impl Drop for NativeProcess {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Drop the receiver before joining: otherwise a full bounded channel
        // would keep the reader blocked after owned process termination.
        let (_, replacement) = sync_channel(1);
        let old = std::mem::replace(&mut self.replies, replacement);
        drop(old);
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_oversize_before_payload_read() {
        let mut header = Vec::new();
        header.extend(MAGIC.to_le_bytes());
        header.extend(11_u32.to_le_bytes());
        header.extend(1_u64.to_le_bytes());
        header.extend(8_201_u32.to_le_bytes());
        assert!(matches!(
            read_reply(&mut &header[..]),
            Err(ErrorCode::InvalidProtocol)
        ));
    }
    #[test]
    fn rejects_bad_magic_and_truncated_data() {
        assert!(read_reply(&mut &[0; 20][..]).is_err());
        assert!(read_reply(&mut &[0; 19][..]).is_err());
    }
}
