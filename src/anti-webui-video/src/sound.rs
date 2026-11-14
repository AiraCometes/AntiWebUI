//! The sound of a file in order, from wherever it is told to go, for programs that put sound
//! together themselves: an editor that mixes the sound of its clips, or what writes a video out.
//! Nothing here keeps time or plays anything. See "音声の読み出し" in `docs/design/24-video.md`.

use crate::ffi::*;
use crate::player::{
    Decoder, Input, Packet, Resampler, StreamInfo, alloc_frame, open_decoder, open_input,
    packet_stream,
};
use anti_webui_core::video::{VideoError, VideoSettings};
use std::ffi::c_int;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// How many pieces in a row may fail before it gives up.
const MAX_FAILURES: u32 = 32;
/// How far off what a piece says it starts at may be from where the one before ended before that
/// is believed: closer than this, the sound follows on, whatever rounding in the file says.
const GAP: f64 = 0.02;

/// What the sound of a file is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioInfo {
    /// In seconds, if the file says.
    pub duration: Option<f64>,
}

/// Sound that follows on, as pairs of samples (left, right) at the rate of the reader.
#[derive(Clone, Debug)]
pub struct AudioChunk {
    /// In seconds from the start of the file: from the start of its pictures if it has any, so
    /// that it lines up with [`VideoReader`](crate::VideoReader).
    pub start: f64,
    pub samples: Vec<f32>,
}

/// Reads the sound of a file in order, as two channels of `f32` at one rate whatever is in it.
pub struct AudioReader {
    // The decoder goes before what it reads from.
    decoder: Decoder,
    input: Input,
    ffmpeg: Arc<Ffmpeg>,
    audio: StreamInfo,
    resampler: Option<Resampler>,
    /// `AV_SAMPLE_FMT_FLT`.
    flt: c_int,
    rate: u32,
    info: AudioInfo,
    /// When the file starts, in the time of the sound.
    origin: f64,
    frame: *mut Opaque,
    stop: Arc<AtomicBool>,
    /// The decoder has been told that there is nothing more.
    drained: bool,
    ended: bool,
    /// Where it last went.
    sought: f64,
    /// Where what comes next starts, from what came before it.
    next: Option<f64>,
    failures: u32,
}

// SAFETY: what FFmpeg has of it is used by one thread at a time, and FFmpeg doesn't mind which.
unsafe impl Send for AudioReader {}

impl AudioReader {
    /// Opens the sound in `source`, a path or what FFmpeg can read, to come out at `rate`
    /// samples a second.
    pub fn open(source: &str, settings: &VideoSettings, rate: u32) -> Result<Self, VideoError> {
        let ffmpeg = ffmpeg(settings)?;
        let stop = Arc::new(AtomicBool::new(false));
        let input = open_input(&ffmpeg, &stop, source)?;
        let Some(audio) = input.audio else {
            return Err(VideoError::UnsupportedCodec(format!(
                "{source}: no sound in it that can be decoded"
            )));
        };
        let l = ffmpeg.layout;
        // SAFETY: the input is open, and the layout says where the fields are.
        let (duration, origin) = unsafe {
            let duration: i64 = read(input.raw, l.format_duration);
            let duration = (duration != AV_NOPTS_VALUE && duration > 0)
                .then(|| duration as f64 / AV_TIME_BASE);
            // The pictures' start, as `VideoReader` has it, or else the sound's own.
            let first = input.video.unwrap_or(audio);
            let start: i64 = read(first.stream, l.stream_start_time);
            let origin = if start != AV_NOPTS_VALUE {
                start as f64 * first.time_base
            } else {
                let start: i64 = read(input.raw, l.format_start_time);
                if start == AV_NOPTS_VALUE {
                    0.0
                } else {
                    start as f64 / AV_TIME_BASE
                }
            };
            (duration, origin)
        };
        let decoder = open_decoder(&ffmpeg, &audio, audio.decoder, None, 1)?;
        // SAFETY: a valid string.
        let flt = unsafe { (ffmpeg.f.av_get_sample_fmt)(c"flt".as_ptr()) };
        let frame = alloc_frame(&ffmpeg);
        if frame.is_null() {
            return Err(VideoError::Decode("out of memory".into()));
        }
        Ok(Self {
            decoder,
            input,
            ffmpeg,
            audio,
            resampler: None,
            flt,
            rate: rate.max(1),
            info: AudioInfo { duration },
            origin,
            frame,
            stop,
            drained: false,
            ended: false,
            sought: 0.0,
            next: None,
            failures: 0,
        })
    }

    pub fn info(&self) -> AudioInfo {
        self.info
    }

    /// Samples a second of what it gives.
    pub fn rate(&self) -> u32 {
        self.rate
    }

    /// Goes to the last point it can start from at or before `seconds`: [`next_chunk`](Self::next_chunk)
    /// goes on from there. Some files only know roughly where those are, so that can be after
    /// `seconds`.
    pub fn seek(&mut self, seconds: f64) -> Result<(), VideoError> {
        let target = ((seconds + self.origin) / self.audio.time_base).floor() as i64;
        // SAFETY: the input and the decoder are open.
        let code = unsafe {
            let code = (self.ffmpeg.f.av_seek_frame)(
                self.input.raw,
                self.audio.index,
                target,
                AVSEEK_FLAG_BACKWARD,
            );
            (self.ffmpeg.f.avcodec_flush_buffers)(self.decoder.raw);
            code
        };
        // What it held is from before.
        self.resampler = None;
        self.sought = seconds;
        self.drained = false;
        self.ended = false;
        self.next = None;
        if code < 0 {
            return Err(VideoError::Source(format!(
                "can't go to {seconds} s: {}",
                self.ffmpeg.error(code)
            )));
        }
        Ok(())
    }

    /// The sound that comes next, or `None` after the end.
    pub fn next_chunk(&mut self) -> Result<Option<AudioChunk>, VideoError> {
        loop {
            if self.ended {
                return Ok(None);
            }
            // SAFETY: an open decoder and an empty frame.
            let code =
                unsafe { (self.ffmpeg.f.avcodec_receive_frame)(self.decoder.raw, self.frame) };
            if code >= 0 {
                self.failures = 0;
                let chunk = self.convert();
                // SAFETY: a frame.
                unsafe { (self.ffmpeg.f.av_frame_unref)(self.frame) };
                match chunk {
                    Some(chunk) if !chunk.samples.is_empty() => return Ok(Some(chunk)),
                    _ => continue,
                }
            }
            if code == AVERROR_EOF || (code == AVERROR_EAGAIN && self.drained) {
                self.ended = true;
                return Ok(self.rest());
            }
            if code != AVERROR_EAGAIN {
                self.failed(code)?;
                continue;
            }
            let packet = self.read_packet()?;
            let raw = packet
                .as_ref()
                .map_or(std::ptr::null(), |p| p.raw.cast_const());
            self.drained = packet.is_none();
            // SAFETY: an open decoder, and a packet or nothing at the end.
            let code = unsafe { (self.ffmpeg.f.avcodec_send_packet)(self.decoder.raw, raw) };
            if code < 0 && code != AVERROR_EOF && code != AVERROR_EAGAIN {
                self.failed(code)?;
            }
        }
    }

    /// The frame just decoded, at the rate and in the layout it gives.
    fn convert(&mut self) -> Option<AudioChunk> {
        let ffmpeg = self.ffmpeg.clone();
        let (f, l) = (&ffmpeg.f, ffmpeg.layout);
        let frame = self.frame;
        // SAFETY: a frame just decoded, and the layout says where the fields are.
        unsafe {
            let format: c_int = read(frame, l.frame_format);
            let rate: c_int = read(frame, l.frame_sample_rate);
            let count: c_int = read(frame, l.frame_nb_samples);
            let pts: i64 = read(frame, l.frame_best_effort);
            let layout = frame
                .cast::<u8>()
                .add(l.frame_ch_layout)
                .cast::<ChannelLayout>();
            if rate <= 0 {
                return None;
            }
            if self
                .resampler
                .as_ref()
                .is_none_or(|r| r.from != (format, rate))
            {
                let mut stereo = ChannelLayout::empty();
                (f.av_channel_layout_default)(&mut stereo, 2);
                let mut raw = std::ptr::null_mut();
                let code = (f.swr_alloc_set_opts2)(
                    &mut raw,
                    &stereo,
                    self.flt,
                    self.rate as c_int,
                    layout,
                    format,
                    rate,
                    0,
                    std::ptr::null_mut(),
                );
                (f.av_channel_layout_uninit)(&mut stereo);
                if code < 0 || (f.swr_init)(raw) < 0 {
                    (f.swr_free)(&mut raw);
                    return None;
                }
                self.resampler = Some(Resampler {
                    ffmpeg: ffmpeg.clone(),
                    raw,
                    from: (format, rate),
                });
            }
            let room = count as usize * self.rate as usize / rate as usize + 256;
            let mut samples = vec![0.0f32; room * 2];
            let out = [samples.as_mut_ptr().cast::<u8>()];
            let input: *const *const u8 = read(frame, l.frame_extended_data);
            let made = (f.swr_convert)(
                self.resampler.as_ref().expect("just made").raw,
                out.as_ptr(),
                room as c_int,
                input,
                count,
            );
            samples.truncate(made.max(0) as usize * 2);
            let said =
                (pts != AV_NOPTS_VALUE).then_some(pts as f64 * self.audio.time_base - self.origin);
            let start = match (self.next, said) {
                (Some(next), Some(said)) if (said - next).abs() < GAP => next,
                (Some(next), None) => next,
                (_, Some(said)) => said,
                (None, None) => self.sought,
            };
            Some(self.chunk(start, samples))
        }
    }

    /// What the resampler still holds at the end.
    fn rest(&mut self) -> Option<AudioChunk> {
        let resampler = self.resampler.as_ref()?;
        let start = self.next?;
        let room = 4096;
        let mut samples = vec![0.0f32; room * 2];
        let out = [samples.as_mut_ptr().cast::<u8>()];
        // SAFETY: an open resampler, told that nothing more comes.
        let made = unsafe {
            (self.ffmpeg.f.swr_convert)(
                resampler.raw,
                out.as_ptr(),
                room as c_int,
                std::ptr::null(),
                0,
            )
        };
        samples.truncate(made.max(0) as usize * 2);
        (!samples.is_empty()).then(|| self.chunk(start, samples))
    }

    fn chunk(&mut self, start: f64, samples: Vec<f32>) -> AudioChunk {
        self.next = Some(start + (samples.len() / 2) as f64 / self.rate as f64);
        AudioChunk { start, samples }
    }

    /// Reads the next packet of the sound, or `None` at the end.
    fn read_packet(&mut self) -> Result<Option<Packet>, VideoError> {
        loop {
            // SAFETY: nothing to it.
            let raw = unsafe { (self.ffmpeg.f.av_packet_alloc)() };
            if raw.is_null() {
                return Err(VideoError::Source("out of memory".into()));
            }
            let packet = Packet {
                ffmpeg: self.ffmpeg.clone(),
                raw,
                size: 0,
            };
            // SAFETY: the input is open and the packet is empty.
            let code = unsafe { (self.ffmpeg.f.av_read_frame)(self.input.raw, packet.raw) };
            if code == AVERROR_EOF {
                return Ok(None);
            }
            if code < 0 {
                return Err(VideoError::Source(self.ffmpeg.error(code)));
            }
            if packet_stream(&packet, self.ffmpeg.layout) == self.audio.index {
                return Ok(Some(packet));
            }
        }
    }

    fn failed(&mut self, code: c_int) -> Result<(), VideoError> {
        self.failures += 1;
        if self.failures > MAX_FAILURES {
            return Err(VideoError::Decode(self.ffmpeg.error(code)));
        }
        Ok(())
    }
}

impl Drop for AudioReader {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // SAFETY: from `av_frame_alloc`, and freed once.
        unsafe { (self.ffmpeg.f.av_frame_free)(&mut self.frame) };
    }
}
