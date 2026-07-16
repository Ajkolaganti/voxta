pub mod cleanup;
#[cfg(not(feature = "whisper"))]
pub mod whisper {
    use super::TranscriptionEngine;
    use crate::{
        audio::AudioBuffer,
        config::AppConfig,
        error::{AppError, AppResult},
        models::ModelManager,
    };
    use async_trait::async_trait;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct WhisperCppEngine {
        _model_manager: Arc<ModelManager>,
    }

    impl WhisperCppEngine {
        pub fn new(model_manager: Arc<ModelManager>) -> Self {
            Self {
                _model_manager: model_manager,
            }
        }
    }

    #[async_trait]
    impl TranscriptionEngine for WhisperCppEngine {
        async fn transcribe(&self, _audio: AudioBuffer, _config: &AppConfig) -> AppResult<String> {
            Err(AppError::Transcription(
                "Voxta was built without the `whisper` feature; local transcription is unavailable in this build.".to_string(),
            ))
        }
    }
}
#[cfg(feature = "whisper")]
pub mod whisper;

use crate::{audio::AudioBuffer, config::AppConfig, error::AppResult};
use async_trait::async_trait;

#[async_trait]
pub trait TranscriptionEngine: Send + Sync {
    async fn transcribe(&self, audio: AudioBuffer, config: &AppConfig) -> AppResult<String>;
}
