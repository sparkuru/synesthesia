//! Native system-output capture and bounded, sample-based normalization.
mod capture;
pub mod format;
pub mod normalize;
pub use capture::{
    CaptureError, CaptureHandle, CaptureObservation, ControlledPlayback, FailureKind, RenderDevice,
    render_devices,
};
#[cfg(windows)]
mod native;
