//! slotPlus music cart: one song, decoded a piece at a time and turned into the cart's
//! own stereo stream at `OUT_RATE`, whatever rate and channels the file has.

use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

/// The rate the cart plays at.
pub const OUT_RATE: u32 = 44_100;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

pub struct Song {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track: u32,
    rate: u32,
    pub tags: Tags,
    /// How long the song is, when the file says.
    pub length_ms: Option<u64>,
    /// Decoded stereo frames at the file's rate, not yet resampled.
    pending: Vec<[f32; 2]>,
    at: usize,
    /// Where between `pending[at]` and the next frame the next output frame falls.
    frac: f64,
    /// Output frames given out so far: where the song is.
    played: u64,
    ended: bool,
}

impl Song {
    pub fn open(path: &Path) -> Result<Song, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let stream = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let mut probed = symphonia::default::get_probe()
            .format(
                &hint,
                stream,
                &FormatOptions {
                    enable_gapless: true,
                    ..Default::default()
                },
                &MetadataOptions::default(),
            )
            .map_err(|e| e.to_string())?;
        let mut tags = Tags::default();
        let mut read_tags = |rev: &symphonia::core::meta::MetadataRevision| {
            for tag in rev.tags() {
                let value = Some(tag.value.to_string()).filter(|v| !v.trim().is_empty());
                match tag.std_key {
                    Some(StandardTagKey::TrackTitle) => tags.title = tags.title.take().or(value),
                    Some(StandardTagKey::Artist) => tags.artist = tags.artist.take().or(value),
                    Some(StandardTagKey::Album) => tags.album = tags.album.take().or(value),
                    _ => {}
                }
            }
        };
        if let Some(rev) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
            read_tags(rev);
        }
        let mut format = probed.format;
        if let Some(rev) = format.metadata().current() {
            read_tags(rev);
        }
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("no audio in the file")?;
        let params = &track.codec_params;
        let rate = params.sample_rate.unwrap_or(OUT_RATE).max(1);
        let length_ms = params.n_frames.map(|n| n * 1000 / rate as u64);
        let decoder = symphonia::default::get_codecs()
            .make(params, &DecoderOptions::default())
            .map_err(|e| e.to_string())?;
        Ok(Song {
            track: track.id,
            format,
            decoder,
            rate,
            tags,
            length_ms,
            pending: Vec::new(),
            at: 0,
            frac: 0.0,
            played: 0,
            ended: false,
        })
    }

    pub fn ended(&self) -> bool {
        self.ended && self.at + 1 >= self.pending.len()
    }

    /// Where the song is, in milliseconds.
    pub fn position_ms(&self) -> u64 {
        self.played * 1000 / OUT_RATE as u64
    }

    /// Decode the next packet onto `pending`. False once the song has run out.
    fn decode_more(&mut self) -> bool {
        if self.ended {
            return false;
        }
        loop {
            let packet = match self.format.next_packet() {
                Ok(p) => p,
                Err(_) => {
                    self.ended = true;
                    return false;
                }
            };
            if packet.track_id() != self.track {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(d) => d,
                // A damaged packet: skip it and carry on, as a player would.
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                Err(_) => {
                    self.ended = true;
                    return false;
                }
            };
            let spec = *decoded.spec();
            let channels = spec.channels.count().max(1);
            let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
            buf.copy_interleaved_ref(decoded);
            // Drop what's already been played through, so `pending` stays small.
            if self.at > 0 {
                self.pending.drain(..self.at);
                self.at = 0;
            }
            for frame in buf.samples().chunks(channels) {
                let l = frame[0];
                let r = frame.get(1).copied().unwrap_or(l);
                self.pending.push([l, r]);
            }
            return true;
        }
    }

    /// Fill `out` with interleaved stereo at `OUT_RATE`; whatever's left after the song ends
    /// is silence. Returns how many frames came from the song.
    pub fn fill(&mut self, out: &mut [i16]) -> usize {
        let step = self.rate as f64 / OUT_RATE as f64;
        let mut made = 0;
        for frame in out.chunks_mut(2) {
            while self.at + 1 >= self.pending.len() {
                if !self.decode_more() {
                    break;
                }
            }
            if self.at + 1 >= self.pending.len() {
                frame.fill(0);
                continue;
            }
            let (a, b) = (self.pending[self.at], self.pending[self.at + 1]);
            let t = self.frac as f32;
            for c in 0..2 {
                let v = a[c] + (b[c] - a[c]) * t;
                frame[c] = (v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            }
            self.frac += step;
            let whole = self.frac.floor();
            self.at += whole as usize;
            self.frac -= whole;
            self.played += 1;
            made += 1;
        }
        made
    }

    /// Jump to `ms` into the song, as near as the file allows.
    pub fn seek_ms(&mut self, ms: u64) {
        let to = SeekTo::Time {
            time: Time::new(ms / 1000, (ms % 1000) as f64 / 1000.0),
            track_id: Some(self.track),
        };
        if let Ok(got) = self.format.seek(SeekMode::Coarse, to) {
            self.decoder.reset();
            self.pending.clear();
            self.at = 0;
            self.frac = 0.0;
            self.ended = false;
            self.played = got.actual_ts * OUT_RATE as u64 / self.rate as u64;
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A WAV file of a sine at `rate`, `secs` long, in `channels`.
    pub(crate) fn wav(path: &Path, rate: u32, channels: u16, secs: f32) {
        let frames = (rate as f32 * secs) as u32;
        let data_len = frames * channels as u32 * 2;
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFF");
        b.extend_from_slice(&(36 + data_len).to_le_bytes());
        b.extend_from_slice(b"WAVEfmt ");
        b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&channels.to_le_bytes());
        b.extend_from_slice(&rate.to_le_bytes());
        b.extend_from_slice(&(rate * channels as u32 * 2).to_le_bytes());
        b.extend_from_slice(&(channels * 2).to_le_bytes());
        b.extend_from_slice(&16u16.to_le_bytes());
        b.extend_from_slice(b"data");
        b.extend_from_slice(&data_len.to_le_bytes());
        for n in 0..frames {
            let v =
                ((n as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 12_000.0) as i16;
            for _ in 0..channels {
                b.extend_from_slice(&v.to_le_bytes());
            }
        }
        std::fs::write(path, b).unwrap();
    }

    #[test]
    fn a_song_at_another_rate_and_in_mono_plays_as_stereo_at_the_cart_s_rate() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("tone.wav");
        wav(&path, 22_050, 1, 1.0);
        let mut song = Song::open(&path).unwrap();
        assert_eq!(song.length_ms, Some(1000));
        let mut out = vec![0i16; 2 * OUT_RATE as usize * 2];
        let made = song.fill(&mut out);
        assert!(
            (OUT_RATE as usize - 3..=OUT_RATE as usize).contains(&made),
            "one second of song is one second at the cart's rate: {made}"
        );
        assert!(song.ended());
        assert_eq!(out[100], out[101], "mono plays on both sides");
        assert!(out[..2 * made].iter().any(|v| v.abs() > 8_000), "it's loud");
        assert!(out[2 * made + 10..].iter().all(|v| *v == 0), "then silence");
    }

    #[test]
    fn seeking_moves_where_the_song_is() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("tone.wav");
        wav(&path, 44_100, 2, 3.0);
        let mut song = Song::open(&path).unwrap();
        song.seek_ms(2_000);
        assert!((1_900..=2_100).contains(&song.position_ms()));
        let mut out = vec![0i16; 2 * 44_100];
        let made = song.fill(&mut out);
        assert!((44_000..=44_100).contains(&made) || song.ended());
    }

    #[test]
    fn a_file_that_is_not_music_says_so() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("junk.mp3");
        std::fs::write(&path, b"not music at all").unwrap();
        assert!(Song::open(&path).is_err());
    }
}
