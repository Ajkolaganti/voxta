use crate::{
    audio::{AudioBuffer, WHISPER_SAMPLE_RATE},
    config::StreamingQuality,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StreamingSession {
    pub session_id: Uuid,
    pub committed_text: String,
    pub unstable_text: String,
    pub processed_samples: usize,
    pub is_cancelled: bool,
}

impl StreamingSession {
    pub fn new(session_id: Uuid) -> Self {
        Self {
            session_id,
            committed_text: String::new(),
            unstable_text: String::new(),
            processed_samples: 0,
            is_cancelled: false,
        }
    }

    pub fn next_chunk(
        &mut self,
        audio: &AudioBuffer,
        quality: StreamingQuality,
    ) -> Option<AudioBuffer> {
        if self.is_cancelled || audio.channels == 0 || audio.samples.is_empty() {
            return None;
        }

        let channels = audio.channels as usize;
        let frames = audio.samples.len() / channels;
        if frames <= self.processed_samples {
            return None;
        }

        let min_step = samples_for_ms(audio.sample_rate, 500);
        if frames.saturating_sub(self.processed_samples) < min_step {
            return None;
        }

        let overlap = samples_for_ms(audio.sample_rate, quality.overlap_ms());
        let max_window = samples_for_ms(audio.sample_rate, quality.window_ms());
        let start_frame = self
            .processed_samples
            .saturating_sub(overlap)
            .min(frames.saturating_sub(1));
        let bounded_start = frames.saturating_sub(max_window).max(start_frame);
        let start = bounded_start * channels;
        let end = frames * channels;

        self.processed_samples = frames;
        Some(AudioBuffer {
            sample_rate: audio.sample_rate,
            channels: audio.channels,
            samples: audio.samples[start..end].to_vec(),
        })
    }

    pub fn update_preview(&mut self, next_text: &str) -> String {
        self.unstable_text = dedupe_preview(&self.unstable_text, next_text);
        let preview = if self.committed_text.is_empty() {
            self.unstable_text.clone()
        } else if self.unstable_text.is_empty() {
            self.committed_text.clone()
        } else {
            format!("{} {}", self.committed_text, self.unstable_text)
        };
        preview.trim().to_string()
    }

    pub fn cancel(&mut self) {
        self.is_cancelled = true;
    }
}

pub trait StreamingQualityExt {
    fn overlap_ms(self) -> u64;
    fn window_ms(self) -> u64;
}

impl StreamingQualityExt for StreamingQuality {
    fn overlap_ms(self) -> u64 {
        match self {
            StreamingQuality::Fast => 200,
            StreamingQuality::Balanced => 350,
            StreamingQuality::Accurate => 500,
        }
    }

    fn window_ms(self) -> u64 {
        match self {
            StreamingQuality::Fast => 2_500,
            StreamingQuality::Balanced => 4_000,
            StreamingQuality::Accurate => 6_000,
        }
    }
}

pub fn samples_for_ms(sample_rate: u32, milliseconds: u64) -> usize {
    let rate = if sample_rate == 0 {
        WHISPER_SAMPLE_RATE
    } else {
        sample_rate
    };
    ((rate as u64 * milliseconds) / 1_000) as usize
}

pub fn dedupe_preview(previous: &str, next: &str) -> String {
    let previous_words = previous.split_whitespace().collect::<Vec<_>>();
    let next_words = next.split_whitespace().collect::<Vec<_>>();
    if previous_words.is_empty() || next_words.is_empty() {
        return next.trim().to_string();
    }

    let max_overlap = previous_words.len().min(next_words.len()).min(12);
    for overlap in (1..=max_overlap).rev() {
        if previous_words[previous_words.len() - overlap..]
            .iter()
            .map(|word| word.to_ascii_lowercase())
            .eq(next_words[..overlap]
                .iter()
                .map(|word| word.to_ascii_lowercase()))
        {
            return previous_words
                .iter()
                .chain(next_words[overlap..].iter())
                .copied()
                .collect::<Vec<_>>()
                .join(" ");
        }
    }

    next.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::{dedupe_preview, samples_for_ms, StreamingSession};
    use crate::{audio::AudioBuffer, config::StreamingQuality};
    use uuid::Uuid;

    #[test]
    fn removes_duplicate_overlap_words() {
        assert_eq!(
            dedupe_preview("hello from the current", "the current preview"),
            "hello from the current preview"
        );
    }

    #[test]
    fn keeps_latest_text_when_no_overlap_exists() {
        assert_eq!(dedupe_preview("old words", "new words"), "new words");
    }

    #[test]
    fn builds_overlapping_audio_chunks() {
        let mut session = StreamingSession::new(Uuid::nil());
        let audio = AudioBuffer {
            sample_rate: 16_000,
            channels: 1,
            samples: vec![0.01; samples_for_ms(16_000, 1_100)],
        };
        let first = session
            .next_chunk(&audio, StreamingQuality::Balanced)
            .unwrap();
        assert_eq!(first.samples.len(), audio.samples.len());

        let audio = AudioBuffer {
            sample_rate: 16_000,
            channels: 1,
            samples: vec![0.01; samples_for_ms(16_000, 1_700)],
        };
        let second = session
            .next_chunk(&audio, StreamingQuality::Balanced)
            .unwrap();
        assert!(second.samples.len() > samples_for_ms(16_000, 500));
    }

    #[test]
    fn cancellation_stops_chunk_generation() {
        let mut session = StreamingSession::new(Uuid::nil());
        session.cancel();
        let audio = AudioBuffer {
            sample_rate: 16_000,
            channels: 1,
            samples: vec![0.01; samples_for_ms(16_000, 1_000)],
        };
        assert!(session.next_chunk(&audio, StreamingQuality::Fast).is_none());
    }
}
