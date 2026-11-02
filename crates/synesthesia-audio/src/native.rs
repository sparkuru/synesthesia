//! All unsafe WASAPI/COM ownership stays on a dedicated apartment worker.
use crate::{
    capture::{
        CaptureError, CaptureHandle, FailureKind, OwnedNormalizer, RAW_PACKETS, RenderDevice,
        Shared, WorkerCompletion,
    },
    format::{Encoding, MixFormat},
    normalize::Normalizer,
};
use std::{
    ptr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, SyncSender, TryRecvError, sync_channel},
    },
    thread,
    time::Duration,
};
use synesthesia_core::{ErrorCode, config::CaptureConfig, domain::InputId};
use windows::{
    Win32::{
        Foundation::{PROPERTYKEY, WAIT_FAILED},
        Media::Audio::*,
        System::Com::{
            CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
            CoUninitialize,
        },
        UI::WindowsAndMessaging::{
            DispatchMessageW, MSG, MWMO_INPUTAVAILABLE, MsgWaitForMultipleObjectsEx, PM_REMOVE,
            PeekMessageW, QS_ALLINPUT, TranslateMessage,
        },
    },
    core::{Interface, PCWSTR, implement},
};

fn backend(error: windows::core::Error) -> CaptureError {
    let kind = if error.code() == AUDCLNT_E_DEVICE_INVALIDATED
        || error.code() == AUDCLNT_E_RESOURCES_INVALIDATED
    {
        FailureKind::DeviceUnavailable
    } else {
        FailureKind::Backend
    };
    CaptureError {
        kind,
        backend_code: error.code().0,
    }
}

struct Apartment;
impl Apartment {
    fn new() -> Result<Self, CaptureError> {
        // SAFETY: a new dedicated worker has no prior apartment; success includes S_FALSE.
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .ok()
                .map_err(backend)?;
        }
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: worker drops interfaces before this successful initialization guard.
        unsafe {
            CoUninitialize();
        }
    }
}

struct TaskAllocation<T>(*mut T);
impl<T> Drop for TaskAllocation<T> {
    fn drop(&mut self) {
        // SAFETY: only GetId/GetMixFormat-owned task allocations enter this guard.
        unsafe {
            CoTaskMemFree(Some(self.0.cast()));
        }
    }
}

fn enumerator() -> Result<IMMDeviceEnumerator, CaptureError> {
    // SAFETY: called within the worker apartment and returns its owned COM reference.
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(backend) }
}

fn device_id(device: &IMMDevice) -> Result<String, CaptureError> {
    // SAFETY: task-allocated UTF16 output is read only while allocation is owned.
    unsafe {
        let id = TaskAllocation(device.GetId().map_err(backend)?.0);
        if id.0.is_null() {
            return Err(CaptureError::new(FailureKind::DeviceUnavailable));
        }
        let mut length = 0;
        while length < 256 && *id.0.add(length) != 0 {
            length += 1;
        }
        if length == 0 || length == 256 {
            return Err(CaptureError::new(FailureKind::DeviceUnavailable));
        }
        let value = String::from_utf16(std::slice::from_raw_parts(id.0, length))
            .map_err(|_| CaptureError::new(FailureKind::DeviceUnavailable))?;
        if value.len() > 256 || value.chars().any(char::is_control) {
            return Err(CaptureError::new(FailureKind::DeviceUnavailable));
        }
        Ok(value)
    }
}

fn select(
    enumerator: &IMMDeviceEnumerator,
    selection: &CaptureConfig,
) -> Result<IMMDevice, CaptureError> {
    // SAFETY: no interface crosses the worker and bounded UTF16 lives through GetDevice.
    unsafe {
        let device = match selection {
            CaptureConfig::FollowDefault {} => enumerator
                .GetDefaultAudioEndpoint(eRender, eMultimedia)
                .map_err(backend)?,
            CaptureConfig::Device { device_id } => {
                if device_id.is_empty()
                    || device_id.len() > 256
                    || device_id.chars().any(char::is_control)
                {
                    return Err(CaptureError::new(FailureKind::InvalidInput));
                }
                let wide: Vec<u16> = device_id.encode_utf16().chain(Some(0)).collect();
                enumerator
                    .GetDevice(PCWSTR(wide.as_ptr()))
                    .map_err(backend)?
            }
        };
        let endpoint: IMMEndpoint = device.cast().map_err(backend)?;
        if endpoint.GetDataFlow().map_err(backend)? != eRender
            || device.GetState().map_err(backend)? != DEVICE_STATE_ACTIVE
        {
            return Err(CaptureError::new(FailureKind::DeviceUnavailable));
        }
        Ok(device)
    }
}

fn mix_format(
    client: &IAudioClient,
) -> Result<(MixFormat, TaskAllocation<WAVEFORMATEX>), CaptureError> {
    // SAFETY: GetMixFormat allocates a valid descriptor and extension. Packed fields
    // are copied by value/unaligned read, never exposed as aligned references.
    unsafe {
        let allocation = TaskAllocation(client.GetMixFormat().map_err(backend)?);
        if allocation.0.is_null() {
            return Err(CaptureError::new(FailureKind::UnsupportedFormat));
        }
        let base = ptr::read_unaligned(allocation.0);
        if base.cbSize > 1024 {
            return Err(CaptureError::new(FailureKind::UnsupportedFormat));
        }
        let descriptor =
            std::slice::from_raw_parts(allocation.0.cast::<u8>(), 18 + usize::from(base.cbSize));
        let format = MixFormat::from_wave_descriptor(descriptor)
            .map_err(|_| CaptureError::new(FailureKind::UnsupportedFormat))?;
        Ok((format, allocation))
    }
}

#[implement(IMMNotificationClient)]
struct Notifications {
    shared: Arc<Shared>,
}
impl IMMNotificationClient_Impl for Notifications_Impl {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: DEVICE_STATE) -> windows::core::Result<()> {
        self.shared.dirty.store(true, Ordering::Release);
        Ok(())
    }
    fn OnDeviceAdded(&self, _: &PCWSTR) -> windows::core::Result<()> {
        self.shared.dirty.store(true, Ordering::Release);
        Ok(())
    }
    fn OnDeviceRemoved(&self, _: &PCWSTR) -> windows::core::Result<()> {
        self.shared.dirty.store(true, Ordering::Release);
        Ok(())
    }
    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        _: &PCWSTR,
    ) -> windows::core::Result<()> {
        if flow == eRender && role == eMultimedia {
            self.shared.dirty.store(true, Ordering::Release);
        }
        Ok(())
    }
    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> windows::core::Result<()> {
        self.shared.dirty.store(true, Ordering::Release);
        Ok(())
    }
}

struct Registration {
    enumerator: IMMDeviceEnumerator,
    sink: IMMNotificationClient,
    shared: Arc<Shared>,
}
impl Registration {
    fn new(enumerator: &IMMDeviceEnumerator, shared: Arc<Shared>) -> Result<Self, CaptureError> {
        let sink: IMMNotificationClient = Notifications {
            shared: shared.clone(),
        }
        .into();
        // SAFETY: retain sink strongly until after unregister; callbacks access only Arc + atomics.
        unsafe {
            enumerator
                .RegisterEndpointNotificationCallback(&sink)
                .map_err(backend)?;
        }
        Ok(Self {
            enumerator: enumerator.clone(),
            sink,
            shared,
        })
    }
}
impl Drop for Registration {
    fn drop(&mut self) {
        // SAFETY: no callback attempts unregister/COM release; its shared Arc outlives
        // callbacks, even if an already-dispatched callback completes after unregister.
        unsafe {
            if let Err(error) = self
                .enumerator
                .UnregisterEndpointNotificationCallback(&self.sink)
            {
                self.shared.fail(backend(error));
            }
        }
    }
}

struct RunningClient {
    client: IAudioClient,
    shared: Option<Arc<Shared>>,
}
impl Drop for RunningClient {
    fn drop(&mut self) {
        // SAFETY: same owning apartment; failed Stop does not prevent remaining cleanup.
        unsafe {
            if let Err(error) = self.client.Stop()
                && let Some(shared) = &self.shared
            {
                shared.fail(backend(error));
            }
        }
    }
}

struct PacketGuard<'a> {
    client: &'a IAudioCaptureClient,
    frames: u32,
    released: bool,
}
impl PacketGuard<'_> {
    fn release(mut self) -> Result<(), CaptureError> {
        // SAFETY: matching GetBuffer on the same worker, complete packet count.
        let result = unsafe { self.client.ReleaseBuffer(self.frames).map_err(backend) };
        self.released = true;
        result
    }
}
impl Drop for PacketGuard<'_> {
    fn drop(&mut self) {
        if !self.released {
            // SAFETY: error cleanup still consumes/releases the owned complete packet.
            unsafe {
                let _ = self.client.ReleaseBuffer(self.frames);
            }
        }
    }
}

struct RawPacket {
    bytes: Vec<u8>,
    frames: usize,
    silent: bool,
}

fn normalize_worker(
    shared: Arc<Shared>,
    mut normalizer: Normalizer,
    packets: Receiver<RawPacket>,
    free: SyncSender<Vec<u8>>,
) {
    let _completion = WorkerCompletion::new(shared.clone(), false);
    while !shared.stop.load(Ordering::Acquire) {
        match packets.recv_timeout(Duration::from_millis(5)) {
            Ok(packet) => {
                shared.idle_drained.store(false, Ordering::Release);
                let result =
                    normalizer.push(&packet.bytes, packet.frames, packet.silent, |frame| {
                        shared.publish(frame)
                    });
                shared.raw_frames.fetch_sub(packet.frames, Ordering::AcqRel);
                let _ = free.try_send(packet.bytes);
                if let Err(error) = result {
                    if !shared.stop.load(Ordering::Acquire) {
                        shared.fail(CaptureError::new(if error == ErrorCode::Overloaded {
                            FailureKind::Overloaded
                        } else {
                            FailureKind::UnsupportedFormat
                        }));
                    }
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let observation = shared.observe();
                if observation.no_packet_ms >= 100
                    && observation.raw_queued_frames == 0
                    && !observation.idle_tail_drained
                {
                    if normalizer
                        .finish_segment(|frame| shared.publish(frame))
                        .is_err()
                    {
                        if !shared.stop.load(Ordering::Acquire) {
                            shared.fail(CaptureError::new(FailureKind::UnsupportedFormat));
                        }
                        break;
                    }
                    shared.idle_drained.store(true, Ordering::Release);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    // Draining the bounded channel destroys old content; nothing is published after stop.
    while packets.try_recv().is_ok() {}
}

fn pump_wait() -> Result<(), CaptureError> {
    // SAFETY: dedicated STA's queue; finite timeout makes idle stop interruptible
    // without EVENTCALLBACK or post-1703-only APIs. No handles are borrowed.
    unsafe {
        if MsgWaitForMultipleObjectsEx(None, 5, QS_ALLINPUT, MWMO_INPUTAVAILABLE) == WAIT_FAILED {
            return Err(backend(windows::core::Error::from_thread()));
        }
        let mut message = MSG::default();
        let mut remaining = 64;
        while remaining > 0 && PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
            remaining -= 1;
        }
    }
    Ok(())
}

struct CaptureLoop<'a> {
    shared: &'a Shared,
    selection: &'a CaptureConfig,
    enumerator: &'a IMMDeviceEnumerator,
    selected_id: &'a str,
    capture: &'a IAudioCaptureClient,
    format: MixFormat,
    packets: &'a SyncSender<RawPacket>,
    free: &'a Receiver<Vec<u8>>,
}

fn capture_loop(context: CaptureLoop<'_>) -> Result<(), CaptureError> {
    let CaptureLoop {
        shared,
        selection,
        enumerator,
        selected_id,
        capture,
        format,
        packets,
        free,
    } = context;
    let mut first = true;
    while !shared.stop.load(Ordering::Acquire) {
        if shared.dirty.swap(false, Ordering::AcqRel) {
            let selected = select(enumerator, selection)?;
            if device_id(&selected)? != selected_id {
                return Err(CaptureError::new(FailureKind::InputChangeRequired));
            }
            // SAFETY: temporary format query on the same apartment; never Initialize twice.
            let check: IAudioClient =
                unsafe { selected.Activate(CLSCTX_ALL, None).map_err(backend)? };
            if mix_format(&check)?.0 != format {
                return Err(CaptureError::new(FailureKind::InputChangeRequired));
            }
        }
        // SAFETY: packet pointer is borrowed only until the matching PacketGuard release;
        // checked format/count bounds precede every slice/copy. No pointer crosses threads.
        unsafe {
            while !shared.stop.load(Ordering::Acquire)
                && capture.GetNextPacketSize().map_err(backend)? != 0
            {
                let mut data = ptr::null_mut();
                let mut frames = 0;
                let mut flags = 0;
                capture
                    .GetBuffer(&mut data, &mut frames, &mut flags, None, None)
                    .map_err(backend)?;
                if frames == 0 {
                    break;
                }
                let held = PacketGuard {
                    client: capture,
                    frames,
                    released: false,
                };
                if flags
                    & !(AUDCLNT_BUFFERFLAGS_SILENT.0 as u32
                        | AUDCLNT_BUFFERFLAGS_DATA_DISCONTINUITY.0 as u32
                        | AUDCLNT_BUFFERFLAGS_TIMESTAMP_ERROR.0 as u32)
                    != 0
                {
                    return Err(CaptureError::new(FailureKind::Backend));
                }
                if !first && flags & AUDCLNT_BUFFERFLAGS_DATA_DISCONTINUITY.0 as u32 != 0 {
                    return Err(CaptureError::new(FailureKind::Discontinuity));
                }
                first = false;
                let count = frames as usize;
                let length = format
                    .packet_bytes(count)
                    .map_err(|_| CaptureError::new(FailureKind::UnsupportedFormat))?;
                let silent = flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0;
                let mut bytes = match free.try_recv() {
                    Ok(bytes) => bytes,
                    Err(TryRecvError::Empty | TryRecvError::Disconnected) => {
                        return Err(CaptureError::new(FailureKind::Overloaded));
                    }
                };
                if shared
                    .raw_frames
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |queued| {
                        queued
                            .checked_add(count)
                            .filter(|&total| total <= format.rate as usize / 5)
                    })
                    .is_err()
                {
                    return Err(CaptureError::new(FailureKind::Overloaded));
                }
                bytes.clear();
                if !silent {
                    if data.is_null() {
                        return Err(CaptureError::new(FailureKind::Backend));
                    }
                    bytes.extend_from_slice(std::slice::from_raw_parts(data, length));
                }
                held.release()?;
                if shared.stop.load(Ordering::Acquire) {
                    break;
                }
                shared
                    .native_frames
                    .fetch_add(u64::from(frames), Ordering::Relaxed);
                shared.packets.fetch_add(1, Ordering::Relaxed);
                if silent {
                    shared.silent_packets.fetch_add(1, Ordering::Relaxed);
                }
                if flags & AUDCLNT_BUFFERFLAGS_TIMESTAMP_ERROR.0 as u32 != 0 {
                    shared.timestamp_errors.fetch_add(1, Ordering::Relaxed);
                }
                shared
                    .last_packet_ms
                    .store(shared.born.elapsed().as_millis() as u64, Ordering::Release);
                shared.idle_drained.store(false, Ordering::Release);
                packets
                    .try_send(RawPacket {
                        bytes,
                        frames: count,
                        silent,
                    })
                    .map_err(|_| CaptureError::new(FailureKind::Overloaded))?;
            }
        }
        pump_wait()?;
    }
    Ok(())
}

fn worker(
    shared: Arc<Shared>,
    selection: CaptureConfig,
    ready: SyncSender<Result<(MixFormat, RenderDevice), CaptureError>>,
) {
    let _completion = WorkerCompletion::new(shared.clone(), true);
    let result = (|| -> Result<(), CaptureError> {
        let _apartment = Apartment::new()?;
        let enumerator = enumerator()?;
        let registration = Registration::new(&enumerator, shared.clone())?;
        let device = select(&enumerator, &selection)?;
        let id = device_id(&device)?;
        // SAFETY: owned device/client are confined to this apartment.
        let client: IAudioClient = unsafe { device.Activate(CLSCTX_ALL, None).map_err(backend)? };
        let (format, allocated) = mix_format(&client)?;
        unsafe {
            client
                .Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    AUDCLNT_STREAMFLAGS_LOOPBACK,
                    1_000_000,
                    0,
                    allocated.0,
                    None,
                )
                .map_err(backend)?;
        }
        drop(allocated);
        let capture: IAudioCaptureClient = unsafe { client.GetService().map_err(backend)? };
        let buffer = unsafe { client.GetBufferSize().map_err(backend)? };
        // Poll 5 ms only with at least 10 ms of actual endpoint capacity.
        if u64::from(buffer) * 1000 < u64::from(format.rate) * 10 || buffer > format.rate / 5 {
            return Err(CaptureError::new(FailureKind::UnsupportedFormat));
        }
        let reselected = select(&enumerator, &selection)?;
        if device_id(&reselected)? != id {
            return Err(CaptureError::new(FailureKind::InputChangeRequired));
        }
        let (packet_tx, packet_rx) = sync_channel(RAW_PACKETS);
        let (free_tx, free_rx) = sync_channel(RAW_PACKETS);
        for _ in 0..RAW_PACKETS {
            free_tx
                .try_send(Vec::with_capacity(
                    format
                        .packet_bytes(format.rate as usize / 10)
                        .map_err(|_| CaptureError::new(FailureKind::UnsupportedFormat))?,
                ))
                .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
        }
        let norm_shared = shared.clone();
        let prepared = Normalizer::new(format, shared.input)
            .map_err(|_| CaptureError::new(FailureKind::UnsupportedFormat))?;
        let normalizer = thread::Builder::new()
            .name("synesthesia-normalize".into())
            .spawn(move || normalize_worker(norm_shared, prepared, packet_rx, free_tx))
            .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
        let mut normalizer = OwnedNormalizer::new(shared.clone(), normalizer);
        let result = (|| {
            unsafe {
                client.Start().map_err(backend)?;
            }
            let running = RunningClient {
                client,
                shared: Some(shared.clone()),
            };
            // Selection policy does not describe current endpoint metadata:
            // an explicitly locked endpoint can also be the actual default.
            let default = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) }
                .ok()
                .and_then(|device| device_id(&device).ok());
            let render = RenderDevice {
                id: id.clone(),
                is_default: default.as_ref() == Some(&id),
            };
            ready
                .try_send(Ok((format, render)))
                .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
            let result = capture_loop(CaptureLoop {
                shared: &shared,
                selection: &selection,
                enumerator: &enumerator,
                selected_id: &id,
                capture: &capture,
                format,
                packets: &packet_tx,
                free: &free_rx,
            });
            drop(capture);
            drop(running);
            result
        })();
        if let Err(error) = result {
            shared.fail(error);
        }
        shared.stop.store(true, Ordering::Release);
        drop(packet_tx);
        normalizer.finish();
        drop(registration);
        result
    })();
    if let Err(error) = result {
        shared.fail(error);
        let _ = ready.try_send(Err(error));
    }
}

pub fn start(selection: CaptureConfig, input: InputId) -> Result<CaptureHandle, CaptureError> {
    let shared = Arc::new(Shared::new(input));
    let worker_shared = shared.clone();
    let (tx, rx) = sync_channel(1);
    let worker = thread::Builder::new()
        .name("synesthesia-wasapi".into())
        .spawn(move || worker(worker_shared, selection, tx))
        .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok((format, device))) => Ok(CaptureHandle {
            shared,
            worker: Some(worker),
            format,
            device,
        }),
        outcome => {
            shared.stop.store(true, Ordering::Release);
            let _ = worker.join();
            Err(outcome
                .ok()
                .and_then(Result::err)
                .unwrap_or(CaptureError::new(FailureKind::WorkerFailed)))
        }
    }
}

pub fn devices() -> Result<Vec<RenderDevice>, CaptureError> {
    // A separate STA permits settings callers from arbitrary GUI apartments.
    thread::Builder::new()
        .name("synesthesia-endpoints".into())
        .spawn(|| {
            let _apartment = Apartment::new()?;
            let enumerator = enumerator()?;
            let default = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) }
                .ok()
                .and_then(|device| device_id(&device).ok());
            let endpoints = unsafe {
                enumerator
                    .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
                    .map_err(backend)?
            };
            let count = unsafe { endpoints.GetCount().map_err(backend)? };
            if count > 128 {
                return Err(CaptureError::new(FailureKind::Overloaded));
            }
            let mut output = Vec::with_capacity(count as usize);
            for index in 0..count {
                let device = unsafe { endpoints.Item(index).map_err(backend)? };
                let id = device_id(&device)?;
                let is_default = default.as_ref() == Some(&id);
                output.push(RenderDevice { id, is_default });
            }
            Ok(output)
        })
        .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?
        .join()
        .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?
}

enum PlaybackCommand {
    Pause,
    Resume,
}
pub(crate) struct Playback {
    stop: Arc<AtomicBool>,
    commands: SyncSender<PlaybackCommand>,
    acknowledgements: Receiver<Result<(), CaptureError>>,
    worker: Option<thread::JoinHandle<Result<(), CaptureError>>>,
}
impl Playback {
    pub fn start(id: String) -> Result<Self, CaptureError> {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let (commands, receiver) = sync_channel(1);
        let (sender, acknowledgements) = sync_channel(1);
        let worker = thread::Builder::new()
            .name("synesthesia-test-playback".into())
            .spawn(move || {
                let result = playback_worker(id, &worker_stop, receiver, &sender);
                if let Err(error) = result {
                    let _ = sender.try_send(Err(error));
                }
                result
            })
            .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
        let mut playback = Self {
            stop,
            commands,
            acknowledgements,
            worker: Some(worker),
        };
        playback.wait_ack()?;
        Ok(playback)
    }
    fn wait_ack(&mut self) -> Result<(), CaptureError> {
        self.acknowledgements
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?
    }
    fn send(&mut self, command: PlaybackCommand) -> Result<(), CaptureError> {
        self.commands
            .try_send(command)
            .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
        self.wait_ack()
    }
    pub fn pause(&mut self) -> Result<(), CaptureError> {
        self.send(PlaybackCommand::Pause)
    }
    pub fn resume(&mut self) -> Result<(), CaptureError> {
        self.send(PlaybackCommand::Resume)
    }
}
impl Drop for Playback {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn fill_tone(
    client: &IAudioClient,
    render: &IAudioRenderClient,
    format: MixFormat,
    capacity: u32,
    cursor: &mut u64,
) -> Result<(), CaptureError> {
    // SAFETY: renderer and packed format are owned by this STA. Complete every
    // GetBuffer with ReleaseBuffer on the same thread, including copy failures.
    unsafe {
        let padding = client.GetCurrentPadding().map_err(backend)?;
        let count = capacity
            .checked_sub(padding)
            .ok_or(CaptureError::new(FailureKind::Backend))?;
        if count == 0 {
            return Ok(());
        }
        let buffer = render.GetBuffer(count).map_err(backend)?;
        let result = (|| {
            if buffer.is_null() {
                return Err(CaptureError::new(FailureKind::Backend));
            }
            let length = (count as usize)
                .checked_mul(usize::from(format.block_align))
                .ok_or(CaptureError::new(FailureKind::Backend))?;
            let output = std::slice::from_raw_parts_mut(buffer, length);
            let bytes = usize::from(format.container_bits / 8);
            for (offset, frame) in output
                .chunks_exact_mut(usize::from(format.block_align))
                .enumerate()
            {
                let position = *cursor + offset as u64;
                for (channel, destination) in frame.chunks_exact_mut(bytes).enumerate() {
                    let (frequency, amplitude) = match channel {
                        0 => (697.0_f64, 0.18_f64),
                        1 => (941.0, 0.12),
                        _ => (0.0, 0.0),
                    };
                    let value = (amplitude
                        * (2.0 * std::f64::consts::PI * frequency * position as f64
                            / f64::from(format.rate))
                        .sin()) as f32;
                    match format.encoding {
                        Encoding::Float => destination.copy_from_slice(&value.to_le_bytes()),
                        Encoding::Pcm => {
                            let valid = (f64::from(value)
                                * 2_f64.powi(i32::from(format.valid_bits) - 1))
                            .round() as i64;
                            let container =
                                (valid << (format.container_bits - format.valid_bits)) as i32;
                            destination.copy_from_slice(&container.to_le_bytes()[..bytes]);
                        }
                    }
                }
            }
            *cursor = cursor
                .checked_add(u64::from(count))
                .ok_or(CaptureError::new(FailureKind::WorkerFailed))?;
            Ok(())
        })();
        // Never publish partial uninitialized render data on a diagnostic failure.
        let release = render
            .ReleaseBuffer(
                count,
                if result.is_ok() {
                    0
                } else {
                    AUDCLNT_BUFFERFLAGS_SILENT.0 as u32
                },
            )
            .map_err(backend);
        result.and(release)
    }
}

fn playback_worker(
    id: String,
    stop: &AtomicBool,
    commands: Receiver<PlaybackCommand>,
    acknowledgements: &SyncSender<Result<(), CaptureError>>,
) -> Result<(), CaptureError> {
    let _apartment = Apartment::new()?;
    let enumerator = enumerator()?;
    let device = select(&enumerator, &CaptureConfig::Device { device_id: id })?;
    // SAFETY: all render interfaces remain on the dedicated apartment worker.
    let client: IAudioClient = unsafe { device.Activate(CLSCTX_ALL, None).map_err(backend)? };
    let (format, allocated) = mix_format(&client)?;
    unsafe {
        client
            .Initialize(AUDCLNT_SHAREMODE_SHARED, 0, 1_000_000, 0, allocated.0, None)
            .map_err(backend)?;
    }
    drop(allocated);
    let render: IAudioRenderClient = unsafe { client.GetService().map_err(backend)? };
    let capacity = unsafe { client.GetBufferSize().map_err(backend)? };
    if capacity > format.rate / 5 {
        return Err(CaptureError::new(FailureKind::UnsupportedFormat));
    }
    let mut cursor = 0;
    fill_tone(&client, &render, format, capacity, &mut cursor)?;
    unsafe {
        client.Start().map_err(backend)?;
    }
    let running = RunningClient {
        client,
        shared: None,
    };
    acknowledgements
        .try_send(Ok(()))
        .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
    let mut paused = false;
    let started = std::time::Instant::now();
    // Guard accidental caller abandonment with a hard diagnostic duration bound.
    while !stop.load(Ordering::Acquire) && started.elapsed() < Duration::from_secs(65) {
        match commands.try_recv() {
            Ok(PlaybackCommand::Pause) => {
                unsafe {
                    running.client.Stop().map_err(backend)?;
                    running.client.Reset().map_err(backend)?;
                }
                paused = true;
                acknowledgements
                    .try_send(Ok(()))
                    .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
            }
            Ok(PlaybackCommand::Resume) => {
                if !paused {
                    return Err(CaptureError::new(FailureKind::InvalidInput));
                }
                cursor = 0;
                fill_tone(&running.client, &render, format, capacity, &mut cursor)?;
                unsafe {
                    running.client.Start().map_err(backend)?;
                }
                paused = false;
                acknowledgements
                    .try_send(Ok(()))
                    .map_err(|_| CaptureError::new(FailureKind::WorkerFailed))?;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => break,
        }
        if !paused {
            fill_tone(&running.client, &render, format, capacity, &mut cursor)?;
        }
        pump_wait()?;
    }
    drop(render);
    drop(running);
    Ok(())
}
