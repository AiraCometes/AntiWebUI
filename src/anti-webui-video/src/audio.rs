//! Sound going out, as 16 bits in two channels, through what each system has: ALSA on Linux and
//! the other Unix systems, WASAPI on Windows, and Audio Queue Services on macOS. Each [`Output`]
//! behaves as ALSA does: writing waits for room, the delay is what is yet to be heard, and it starts
//! by itself once written to. See "音声" in `docs/design/24-video.md`.

use std::time::Duration;

#[cfg(not(any(windows, target_vendor = "apple")))]
mod alsa;
#[cfg(target_vendor = "apple")]
mod audio_queue;
#[cfg(windows)]
mod wasapi;

#[cfg(not(any(windows, target_vendor = "apple")))]
pub use alsa::Output;
#[cfg(target_vendor = "apple")]
pub use audio_queue::Output;
#[cfg(windows)]
pub use wasapi::Output;

/// How much is in the device at most. Pausing and the volume react within this.
const LATENCY: Duration = Duration::from_millis(100);

/// Where sound goes: what `ANTIWEBUI_AUDIO_DEVICE` says, or `default`. `None` for `none`, which is
/// nowhere. Only ALSA knows names of devices; elsewhere, any other name is the default.
fn device() -> Option<String> {
    let device = std::env::var("ANTIWEBUI_AUDIO_DEVICE").unwrap_or_else(|_| "default".into());
    (device != "none").then_some(device)
}
