//! Sound through Audio Queue Services, which converts it to the rate of the device. It is part of
//! every macOS, so it is linked to. The queue takes buffers and gives them back once they are
//! heard, on a thread of its own; what is held is what is in the buffers it has.

use std::ffi::c_void;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// The buffers the queue has at most, each a part of [`super::LATENCY`]. As many as this, so that
/// what is yet to be heard is known within a few milliseconds.
const BUFFERS: u32 = 20;

/// `kAudioFormatLinearPCM`.
const LINEAR_PCM: u32 = u32::from_be_bytes(*b"lpcm");
/// `kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked`.
const SIGNED_PACKED: u32 = (1 << 2) | (1 << 3);

/// `AudioStreamBasicDescription`.
#[repr(C)]
struct StreamDescription {
    sample_rate: f64,
    format_id: u32,
    format_flags: u32,
    bytes_per_packet: u32,
    frames_per_packet: u32,
    bytes_per_frame: u32,
    channels_per_frame: u32,
    bits_per_channel: u32,
    reserved: u32,
}

/// `AudioQueueBuffer`.
#[repr(C)]
struct QueueBuffer {
    capacity: u32,
    data: *mut c_void,
    size: u32,
    user_data: *mut c_void,
    packet_description_capacity: u32,
    packet_descriptions: *mut c_void,
    packet_description_count: u32,
}

type Queue = *mut c_void;
type Callback = unsafe extern "C" fn(*mut c_void, Queue, *mut QueueBuffer);

#[link(name = "AudioToolbox", kind = "framework")]
unsafe extern "C" {
    fn AudioQueueNewOutput(
        format: *const StreamDescription,
        callback: Callback,
        user_data: *mut c_void,
        run_loop: *const c_void,
        run_loop_mode: *const c_void,
        flags: u32,
        queue: *mut Queue,
    ) -> i32;
    fn AudioQueueAllocateBuffer(queue: Queue, size: u32, buffer: *mut *mut QueueBuffer) -> i32;
    fn AudioQueueEnqueueBuffer(
        queue: Queue,
        buffer: *mut QueueBuffer,
        packet_descriptions: u32,
        descriptions: *const c_void,
    ) -> i32;
    fn AudioQueueStart(queue: Queue, start_time: *const c_void) -> i32;
    fn AudioQueuePause(queue: Queue) -> i32;
    fn AudioQueueStop(queue: Queue, immediate: u8) -> i32;
    fn AudioQueueDispose(queue: Queue, immediate: u8) -> i32;
}

/// A buffer of the queue, which is only touched by whoever holds it.
struct Buffer(*mut QueueBuffer);

// SAFETY: as above.
unsafe impl Send for Buffer {}

/// What the queue's thread and the writer share.
struct Shared {
    state: Mutex<State>,
    /// A buffer came back.
    returned: Condvar,
}

struct State {
    /// The buffers the queue doesn't have.
    free: Vec<Buffer>,
    /// The frames in the buffers it has.
    held: u64,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        // Nothing that holds it can panic in between.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The queue gives back a buffer, heard or flushed.
unsafe extern "C" fn returned(user_data: *mut c_void, _queue: Queue, buffer: *mut QueueBuffer) {
    // SAFETY: `user_data` is the `Shared` of the output, which outlives the queue, and the buffer
    // is one of the queue's.
    let (shared, frames) = unsafe { (&*user_data.cast::<Shared>(), (*buffer).size / 4) };
    let mut state = shared.lock();
    state.held = state.held.saturating_sub(u64::from(frames));
    state.free.push(Buffer(buffer));
    shared.returned.notify_all();
}

/// Sound going out, as 16 bits in two channels.
pub struct Output {
    queue: Queue,
    shared: Arc<Shared>,
    /// The frames that fit in a buffer.
    frames_per_buffer: usize,
    running: bool,
    paused: bool,
    pub rate: u32,
}

// SAFETY: the queue can be called from any thread; it is used from one at a time.
unsafe impl Send for Output {}

impl Output {
    /// `None` when there is nowhere for sound to go, which is no reason not to show the picture.
    pub fn open(rate: u32) -> Option<Self> {
        super::device()?;
        let format = StreamDescription {
            sample_rate: f64::from(rate),
            format_id: LINEAR_PCM,
            format_flags: SIGNED_PACKED,
            bytes_per_packet: 4,
            frames_per_packet: 1,
            bytes_per_frame: 4,
            channels_per_frame: 2,
            bits_per_channel: 16,
            reserved: 0,
        };
        let frames_per_buffer =
            ((u128::from(rate) * super::LATENCY.as_millis()) / 1000 / u128::from(BUFFERS)).max(1);
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                free: Vec::new(),
                held: 0,
            }),
            returned: Condvar::new(),
        });
        let mut queue = std::ptr::null_mut();
        // SAFETY: valid pointers. Without a run loop, the queue calls back on a thread of its own.
        let made = unsafe {
            AudioQueueNewOutput(
                &format,
                returned,
                Arc::as_ptr(&shared).cast_mut().cast(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut queue,
            )
        };
        if made != 0 || queue.is_null() {
            return None;
        }
        // From here, dropping it disposes of the queue.
        let output = Self {
            queue,
            shared,
            frames_per_buffer: frames_per_buffer as usize,
            running: false,
            paused: false,
            rate,
        };
        for _ in 0..BUFFERS {
            let mut buffer = std::ptr::null_mut();
            // SAFETY: a queue, and a valid pointer.
            let made = unsafe {
                AudioQueueAllocateBuffer(output.queue, frames_per_buffer as u32 * 4, &mut buffer)
            };
            if made != 0 || buffer.is_null() {
                return None;
            }
            output.shared.lock().free.push(Buffer(buffer));
        }
        Some(output)
    }

    /// Hands over pairs of samples, and waits for there to be room. Returns whether that worked.
    pub fn write(&mut self, mut samples: &[i16]) -> bool {
        while !samples.is_empty() {
            let Some(Buffer(buffer)) = self.free_buffer() else {
                return false;
            };
            let frames = (samples.len() / 2).min(self.frames_per_buffer);
            // SAFETY: the queue gave the buffer back, and it holds `frames_per_buffer` frames.
            let enqueued = unsafe {
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr(),
                    (*buffer).data.cast::<i16>(),
                    frames * 2,
                );
                (*buffer).size = frames as u32 * 4;
                // Counted before the queue has it, which may give it back at once.
                self.shared.lock().held += frames as u64;
                AudioQueueEnqueueBuffer(self.queue, buffer, 0, std::ptr::null()) == 0
            };
            if !enqueued {
                let mut state = self.shared.lock();
                state.held = state.held.saturating_sub(frames as u64);
                state.free.push(Buffer(buffer));
                return false;
            }
            samples = &samples[frames * 2..];
        }
        self.start()
    }

    /// A buffer the queue doesn't have, waiting for one. `None` when none comes back while it plays.
    fn free_buffer(&mut self) -> Option<Buffer> {
        let mut state = self.shared.lock();
        if let Some(buffer) = state.free.pop() {
            return Some(buffer);
        }
        // All of them are written before it is started: as ALSA, it starts now.
        drop(state);
        if !self.start() {
            return None;
        }
        state = self.shared.lock();
        let mut since = Instant::now();
        loop {
            if let Some(buffer) = state.free.pop() {
                return Some(buffer);
            }
            state = self
                .shared
                .returned
                .wait_timeout(state, super::LATENCY)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
            // While paused, nothing is heard, and that is no fault. Playing, a buffer comes back
            // within what they all hold, unless the device is gone.
            if self.paused {
                since = Instant::now();
            } else if state.free.is_empty() && since.elapsed() > super::LATENCY * 4 {
                return None;
            }
        }
    }

    /// Starts it, unless it has been or is paused.
    fn start(&mut self) -> bool {
        if self.running || self.paused {
            return true;
        }
        // SAFETY: a queue; it starts now.
        self.running = unsafe { AudioQueueStart(self.queue, std::ptr::null()) } == 0;
        self.running
    }

    /// How many seconds of what was written are yet to be heard.
    pub fn delay(&self) -> f64 {
        self.shared.lock().held as f64 / self.rate as f64
    }

    /// Forgets what is yet to be heard.
    pub fn flush(&mut self) {
        // SAFETY: a queue. Stopped at once, it gives back every buffer it has.
        unsafe { AudioQueueStop(self.queue, 1) };
        self.running = false;
        self.paused = false;
        let state = self.shared.lock();
        let _ = self
            .shared
            .returned
            .wait_timeout_while(state, Duration::from_secs(1), |s| {
                s.free.len() < BUFFERS as usize
            });
    }

    /// Stops or goes on. Returns whether the device can: it always can.
    pub fn pause(&mut self, pause: bool) -> bool {
        if pause {
            if self.running {
                // SAFETY: a queue. What it has stays.
                unsafe { AudioQueuePause(self.queue) };
                self.running = false;
            }
            self.paused = true;
        } else {
            self.paused = false;
            if self.shared.lock().held > 0 {
                self.start();
            }
        }
        true
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        // SAFETY: a queue, disposed of once, at once, with its buffers: it calls back no more after
        // this, so `shared` can go.
        unsafe { AudioQueueDispose(self.queue, 1) };
    }
}
