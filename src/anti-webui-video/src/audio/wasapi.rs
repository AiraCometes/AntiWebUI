//! Sound through WASAPI, shared with the rest of the system, which converts it to the rate of the
//! device. It is part of every Windows, so it is linked to.

use std::time::Duration;
use windows::Win32::Media::Audio::{
    AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM,
    AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY, IAudioClient, IAudioRenderClient, IMMDeviceEnumerator,
    MMDeviceEnumerator, WAVE_FORMAT_PCM, WAVEFORMATEX, eConsole, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx};

/// How long to wait for room before looking again: a twentieth of what the device holds.
const POLL: Duration = Duration::from_millis(5);

/// Sound going out, as 16 bits in two channels.
pub struct Output {
    client: IAudioClient,
    render: IAudioRenderClient,
    /// How many frames the device holds.
    capacity: u32,
    /// What the system adds to what it holds, in seconds.
    latency: f64,
    /// Started, and not stopped since.
    running: bool,
    paused: bool,
    pub rate: u32,
}

// SAFETY: WASAPI's objects are made in the multithreaded apartment, and can be used from any thread
// in it, which a thread that hasn't said otherwise is. It is used from one thread at a time.
unsafe impl Send for Output {}

impl Output {
    /// `None` when there is nowhere for sound to go, which is no reason not to show the picture.
    pub fn open(rate: u32) -> Option<Self> {
        super::device()?;
        let format = WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_PCM as u16,
            nChannels: 2,
            nSamplesPerSec: rate,
            nAvgBytesPerSec: rate * 4,
            nBlockAlign: 4,
            wBitsPerSample: 16,
            cbSize: 0,
        };
        // In units of 100 ns.
        let held = (super::LATENCY.as_nanos() / 100) as i64;
        // SAFETY: the calls are as WASAPI says, with what they ask for.
        unsafe {
            // When the thread is in an apartment already, that one is kept, and the objects work
            // from there as well.
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let devices: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
            let device = devices.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
            let client: IAudioClient = device.Activate(CLSCTX_ALL, None).ok()?;
            client
                .Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                    held,
                    0,
                    &format,
                    None,
                )
                .ok()?;
            let capacity = client.GetBufferSize().ok()?;
            let latency = client.GetStreamLatency().map_or(0.0, |l| l as f64 / 1e7);
            let render = client.GetService().ok()?;
            Some(Self {
                client,
                render,
                capacity,
                latency,
                running: false,
                paused: false,
                rate,
            })
        }
    }

    /// Hands over pairs of samples, and waits for there to be room. Returns whether that worked.
    pub fn write(&mut self, mut samples: &[i16]) -> bool {
        while !samples.is_empty() {
            // SAFETY: an initialized client.
            let Ok(held) = (unsafe { self.client.GetCurrentPadding() }) else {
                return false;
            };
            let room = self.capacity.saturating_sub(held);
            if room == 0 {
                // Full before it was started: as ALSA, it starts now.
                if !self.start() {
                    return false;
                }
                std::thread::sleep(POLL);
                continue;
            }
            let frames = room.min((samples.len() / 2) as u32);
            // SAFETY: there is room for `frames`, which the buffer is that long for, and it is
            // given back before anything else is asked of the client.
            unsafe {
                let Ok(buffer) = self.render.GetBuffer(frames) else {
                    return false;
                };
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr(),
                    buffer.cast::<i16>(),
                    frames as usize * 2,
                );
                if self.render.ReleaseBuffer(frames, 0).is_err() {
                    return false;
                }
            }
            samples = &samples[frames as usize * 2..];
        }
        self.start()
    }

    /// Starts it, unless it has been or is paused.
    fn start(&mut self) -> bool {
        if self.running || self.paused {
            return true;
        }
        // SAFETY: an initialized client.
        self.running = unsafe { self.client.Start() }.is_ok();
        self.running
    }

    /// How many seconds of what was written are yet to be heard.
    pub fn delay(&self) -> f64 {
        // SAFETY: an initialized client.
        let held = unsafe { self.client.GetCurrentPadding() }.unwrap_or(0);
        held as f64 / self.rate as f64 + if held > 0 { self.latency } else { 0.0 }
    }

    /// Forgets what is yet to be heard.
    pub fn flush(&mut self) {
        // SAFETY: an initialized client. Resetting needs it stopped.
        unsafe {
            let _ = self.client.Stop();
            let _ = self.client.Reset();
        }
        self.running = false;
        self.paused = false;
    }

    /// Stops or goes on. Returns whether the device can: it always can.
    pub fn pause(&mut self, pause: bool) -> bool {
        if pause {
            if self.running {
                // SAFETY: an initialized client. What it holds stays.
                let _ = unsafe { self.client.Stop() };
                self.running = false;
            }
            self.paused = true;
        } else {
            self.paused = false;
            // SAFETY: an initialized client.
            if unsafe { self.client.GetCurrentPadding() }.is_ok_and(|held| held > 0) {
                self.start();
            }
        }
        true
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        // SAFETY: an initialized client.
        let _ = unsafe { self.client.Stop() };
    }
}
