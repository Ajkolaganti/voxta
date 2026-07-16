use super::TranscriptionEngine;
use crate::{
    audio::AudioBuffer,
    config::AppConfig,
    error::{AppError, AppResult},
    models::ModelManager,
    transcription::cleanup::normalize_spaces,
};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::sync::Arc;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Clone)]
pub struct WhisperCppEngine {
    model_manager: Arc<ModelManager>,
    cached_context: Arc<Mutex<Option<CachedContext>>>,
}

struct CachedContext {
    model_id: String,
    context: Arc<WhisperContext>,
}

impl WhisperCppEngine {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self {
            model_manager,
            cached_context: Arc::new(Mutex::new(None)),
        }
    }

    fn context_for_model(&self, model_id: &str) -> AppResult<Arc<WhisperContext>> {
        if let Some(cached) = self.cached_context.lock().as_ref() {
            if cached.model_id == model_id {
                return Ok(cached.context.clone());
            }
        }

        let model_path = self.model_manager.installed_model_path(model_id)?;
        let model_path = model_path
            .to_str()
            .ok_or_else(|| AppError::Model("model path is not valid UTF-8".to_string()))?;
        let context = Arc::new(
            WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
                .map_err(|err| AppError::Transcription(format!("could not load model: {err}")))?,
        );
        *self.cached_context.lock() = Some(CachedContext {
            model_id: model_id.to_string(),
            context: context.clone(),
        });
        Ok(context)
    }
}

#[async_trait]
impl TranscriptionEngine for WhisperCppEngine {
    async fn transcribe(&self, audio: AudioBuffer, config: &AppConfig) -> AppResult<String> {
        if audio.is_too_short() {
            return Ok(String::new());
        }
        if audio.is_silent() {
            return Ok(String::new());
        }

        let context = self.context_for_model(&config.model)?;
        let pcm = audio.to_whisper_mono();
        let config = config.clone();

        tokio::task::spawn_blocking(move || {
            let mut state = context.create_state().map_err(|err| {
                AppError::Transcription(format!("could not create whisper state: {err}"))
            })?;

            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_print_progress(false);
            params.set_print_special(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);
            params.set_translate(false);
            params.set_n_threads(thread_count());

            if config.language != "auto" {
                params.set_language(Some(&config.language));
            }

            state
                .full(params, &pcm)
                .map_err(|err| AppError::Transcription(format!("transcription failed: {err}")))?;

            let segments = state.full_n_segments().map_err(|err| {
                AppError::Transcription(format!("could not read segments: {err}"))
            })?;
            let mut text = String::new();
            for index in 0..segments {
                let segment = state.full_get_segment_text(index).map_err(|err| {
                    AppError::Transcription(format!("could not read segment {index}: {err}"))
                })?;
                text.push_str(segment.trim());
                text.push(' ');
            }

            Ok(normalize_spaces(text.trim()).trim().to_string())
        })
        .await
        .map_err(|err| AppError::Transcription(format!("transcription task failed: {err}")))?
    }
}

fn thread_count() -> i32 {
    std::thread::available_parallelism()
        .map(|count| count.get().clamp(1, 8) as i32)
        .unwrap_or(4)
}
