use crate::{
    ai_cleanup,
    audio::AudioRecorderHandle,
    config::AppConfig,
    error::{AppError, AppResult},
    focus::{FocusSnapshot, FocusedApplicationTracker, PlatformFocusTracker},
    state::{RecordingState, RecordingStateMachine},
    streaming::StreamingSession,
    text_injection::{PlatformTextInjector, TextInjector},
    transcription::cleanup::cleanup_transcript,
    transcription::TranscriptionEngine,
    voice_commands,
};
use parking_lot::{Mutex, RwLock};
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub status: RecordingState,
    pub enabled: bool,
    pub current_model: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlayPayload {
    status: RecordingState,
    message: String,
    preview: Option<String>,
}

#[derive(Debug, Clone)]
struct ActiveStreamingSession {
    session_id: Uuid,
    cancelled: Arc<AtomicBool>,
}

pub struct DictationRuntime {
    config: Arc<RwLock<AppConfig>>,
    state: Arc<Mutex<RecordingStateMachine>>,
    active_focus: Mutex<Option<FocusSnapshot>>,
    audio: AudioRecorderHandle,
    last_message: Mutex<Option<String>>,
    focus_tracker: Arc<dyn FocusedApplicationTracker>,
    text_injector: Arc<dyn TextInjector>,
    transcriber: Arc<dyn TranscriptionEngine>,
    app_handle: Mutex<Option<AppHandle>>,
    streaming_session: Mutex<Option<ActiveStreamingSession>>,
}

impl DictationRuntime {
    pub fn new(
        config: Arc<RwLock<AppConfig>>,
        transcriber: Arc<dyn TranscriptionEngine>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config,
            state: Arc::new(Mutex::new(RecordingStateMachine::default())),
            active_focus: Mutex::new(None),
            audio: AudioRecorderHandle::spawn(),
            last_message: Mutex::new(None),
            focus_tracker: Arc::new(PlatformFocusTracker),
            text_injector: Arc::new(PlatformTextInjector),
            transcriber,
            app_handle: Mutex::new(None),
            streaming_session: Mutex::new(None),
        })
    }

    pub fn set_app_handle(&self, app_handle: AppHandle) {
        *self.app_handle.lock() = Some(app_handle);
    }

    pub fn status(&self) -> RuntimeStatus {
        let config = self.config.read().clone();
        RuntimeStatus {
            status: self.state.lock().state(),
            enabled: config.enabled,
            current_model: config.model,
            message: self.last_message.lock().clone(),
        }
    }

    pub fn report_error(&self, message: &str) {
        self.fail_and_idle(message.to_string());
    }

    pub fn start_recording(self: &Arc<Self>) -> AppResult<()> {
        let config = self.config.read().clone();
        if !config.enabled {
            return Ok(());
        }

        self.set_message("Starting recording...");
        self.state.lock().start_recording()?;
        let focus = match self.focus_tracker.capture() {
            Ok(focus) => focus,
            Err(err) => {
                self.fail_and_idle(user_message(&err));
                return Err(err);
            }
        };

        match self.audio.start(&config.microphone_id) {
            Ok(()) => {}
            Err(err) => {
                self.fail_and_idle(user_message(&err));
                return Err(err);
            }
        };

        *self.active_focus.lock() = Some(focus);
        self.log("recording started");
        self.show_overlay(RecordingState::Recording, "Listening...");
        if config.streaming.enabled {
            self.start_streaming_preview(config.clone());
        }
        if config.play_sounds {
            crate::platform::play_start_sound();
        }
        Ok(())
    }

    pub fn cancel_recording(&self) -> AppResult<()> {
        self.cancel_streaming_preview();
        self.audio.cancel();
        let _ = self.active_focus.lock().take();
        let result = self.state.lock().cancel();
        self.log("recording cancelled");
        self.set_message("Recording cancelled.");
        self.show_overlay(RecordingState::Cancelled, "Cancelled");
        self.hide_overlay_after(Duration::from_millis(350));
        self.state.lock().finish();
        result
    }

    pub fn stop_recording(self: &Arc<Self>) -> AppResult<()> {
        self.cancel_streaming_preview();
        let focus = self.active_focus.lock().take().unwrap_or_default();
        self.state.lock().start_transcribing()?;

        let config = self.config.read().clone();
        if config.play_sounds {
            crate::platform::play_stop_sound();
        }
        self.show_overlay(RecordingState::Transcribing, "Transcribing...");

        let audio = match self.audio.stop() {
            Ok(audio) => audio,
            Err(err) => {
                self.fail_and_idle(user_message(&err));
                return Err(err);
            }
        };
        if audio.is_too_short() || audio.is_silent() {
            self.set_message("No speech detected.");
            self.show_overlay(RecordingState::Idle, "No speech detected");
            self.hide_overlay_after(Duration::from_millis(900));
            self.state.lock().finish();
            return Ok(());
        }

        self.log("recording stopped; transcription queued");
        let runtime = self.clone();
        tauri::async_runtime::spawn(async move {
            runtime.finish_dictation(audio, focus, config).await;
        });
        Ok(())
    }

    async fn finish_dictation(
        self: Arc<Self>,
        audio: crate::audio::AudioBuffer,
        focus: FocusSnapshot,
        config: AppConfig,
    ) {
        let result = async {
            let transcript = self.transcriber.transcribe(audio, &config).await?;
            if transcript.trim().is_empty() {
                self.set_message("No speech detected.");
                self.show_overlay(RecordingState::Idle, "No speech detected");
                self.hide_overlay_after(Duration::from_millis(900));
                return Ok(());
            }

            self.state.lock().start_applying_commands()?;
            self.show_overlay(RecordingState::ApplyingCommands, "Applying commands...");
            let command_result =
                voice_commands::process_transcript(&transcript, &config.voice_commands);
            if command_result.cancelled {
                self.set_message("Dictation cancelled.");
                self.show_overlay(RecordingState::Cancelled, "Cancelled");
                self.hide_overlay_after(Duration::from_millis(450));
                return Ok(());
            }

            let mut final_config = config.clone();
            let mut text = cleanup_transcript(&command_result.text, &final_config);
            if text.trim().is_empty() {
                self.set_message("No text to insert.");
                self.show_overlay(RecordingState::Idle, "No text to insert");
                self.hide_overlay_after(Duration::from_millis(900));
                return Ok(());
            }

            if config.ai_cleanup.enabled {
                self.state.lock().start_cleaning_up()?;
                self.show_overlay(RecordingState::CleaningUp, "Cleaning up...");
                final_config.trailing_space = false;
                match ai_cleanup::cleanup_text(text.trim(), &config.ai_cleanup).await {
                    Ok(Some(cleaned)) => {
                        text = cleanup_transcript(&cleaned, &config);
                    }
                    Ok(None) => {}
                    Err(err) => {
                        self.set_message(
                            "AI cleanup unavailable. Original transcription inserted.",
                        );
                        self.log(&format!("AI cleanup unavailable: {err}"));
                    }
                }
            }

            self.state.lock().start_inserting()?;
            self.show_overlay(RecordingState::Inserting, "Inserting...");
            self.focus_tracker.restore(&focus)?;
            self.text_injector
                .insert_text(&text, &focus, config.insertion_mode)
                .await?;
            self.set_message("Dictation inserted.");
            self.log("dictation inserted");
            Ok::<(), AppError>(())
        }
        .await;

        match result {
            Ok(()) => {
                self.state.lock().finish();
                self.hide_overlay();
            }
            Err(err) => {
                self.fail_and_idle(user_message(&err));
            }
        }
    }

    fn start_streaming_preview(self: &Arc<Self>, config: AppConfig) {
        let session_id = Uuid::new_v4();
        let cancelled = Arc::new(AtomicBool::new(false));
        *self.streaming_session.lock() = Some(ActiveStreamingSession {
            session_id,
            cancelled: cancelled.clone(),
        });

        let runtime = self.clone();
        tauri::async_runtime::spawn(async move {
            runtime
                .run_streaming_preview(session_id, cancelled, config)
                .await;
        });
    }

    fn cancel_streaming_preview(&self) {
        if let Some(session) = self.streaming_session.lock().take() {
            session.cancelled.store(true, Ordering::SeqCst);
        }
    }

    async fn run_streaming_preview(
        self: Arc<Self>,
        session_id: Uuid,
        cancelled: Arc<AtomicBool>,
        config: AppConfig,
    ) {
        let interval_ms = config.streaming.interval_ms.clamp(500, 2_000);
        let mut session = StreamingSession::new(session_id);
        let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));

        loop {
            interval.tick().await;
            if cancelled.load(Ordering::SeqCst) || !self.is_streaming_session_current(session_id) {
                session.cancel();
                return;
            }
            if self.state.lock().state() != RecordingState::Recording {
                return;
            }

            let audio = match self.audio.snapshot() {
                Ok(Some(audio)) => audio,
                Ok(None) => return,
                Err(err) => {
                    self.log(&format!("live preview snapshot failed: {err}"));
                    self.show_overlay(
                        RecordingState::Recording,
                        "Live preview unavailable; final transcription will continue.",
                    );
                    return;
                }
            };

            let Some(chunk) = session.next_chunk(&audio, config.streaming.quality) else {
                continue;
            };
            if chunk.is_too_short() || chunk.is_silent() {
                continue;
            }

            let mut preview_config = config.clone();
            preview_config.trailing_space = false;
            let text = match self.transcriber.transcribe(chunk, &preview_config).await {
                Ok(text) => text,
                Err(err) => {
                    self.log(&format!("live preview transcription failed: {err}"));
                    self.show_overlay(
                        RecordingState::Recording,
                        "Live preview unavailable; final transcription will continue.",
                    );
                    return;
                }
            };

            if cancelled.load(Ordering::SeqCst) || !self.is_streaming_session_current(session_id) {
                return;
            }

            let processed = voice_commands::process_transcript(&text, &config.voice_commands);
            if processed.cancelled {
                let _ = self.cancel_recording();
                return;
            }
            let preview = cleanup_transcript(
                &processed.text,
                &AppConfig {
                    trailing_space: false,
                    ..config.clone()
                },
            );
            let preview = session.update_preview(&preview);
            if !preview.is_empty() {
                self.show_overlay_with_preview(
                    RecordingState::Recording,
                    "Listening...",
                    Some(preview),
                );
            }
        }
    }

    fn is_streaming_session_current(&self, session_id: Uuid) -> bool {
        self.streaming_session
            .lock()
            .as_ref()
            .map(|session| session.session_id == session_id)
            .unwrap_or(false)
    }

    fn show_overlay(&self, status: RecordingState, message: &str) {
        self.show_overlay_with_preview(status, message, None);
    }

    fn show_overlay_with_preview(
        &self,
        status: RecordingState,
        message: &str,
        preview: Option<String>,
    ) {
        let Some(app) = self.app_handle.lock().clone() else {
            return;
        };
        let Some(window) = app.get_webview_window("overlay") else {
            return;
        };

        if let Err(err) = window.emit(
            "voxta-overlay",
            OverlayPayload {
                status,
                message: message.to_string(),
                preview,
            },
        ) {
            self.log(&format!("failed to update overlay: {err}"));
        }
        if let Err(err) = window.show() {
            self.log(&format!("failed to show overlay: {err}"));
        }
    }

    fn show_error(&self, message: &str) {
        self.set_message(message);
        self.log(&format!("error: {message}"));
        self.show_overlay(RecordingState::Error, message);
    }

    fn hide_overlay(&self) {
        let Some(app) = self.app_handle.lock().clone() else {
            return;
        };
        if let Some(window) = app.get_webview_window("overlay") {
            if let Err(err) = window.hide() {
                self.log(&format!("failed to hide overlay: {err}"));
            }
        }
    }

    fn hide_overlay_after(&self, duration: Duration) {
        let Some(app) = self.app_handle.lock().clone() else {
            return;
        };
        thread::spawn(move || {
            thread::sleep(duration);
            if let Some(window) = app.get_webview_window("overlay") {
                if let Err(err) = window.hide() {
                    eprintln!("[Voxta] failed to hide overlay: {err}");
                }
            }
        });
    }

    fn fail_and_idle(&self, message: String) {
        self.state.lock().fail();
        self.show_error(&message);
        self.hide_overlay_after(Duration::from_secs(2));
        self.state.lock().finish();
    }

    fn set_message(&self, message: &str) {
        *self.last_message.lock() = Some(message.to_string());
    }

    fn log(&self, message: &str) {
        eprintln!("[Voxta] {message}");
    }
}

fn user_message(err: &AppError) -> String {
    match err {
        AppError::Audio(message) => {
            if message.contains("no default microphone") || message.contains("unavailable") {
                "Microphone unavailable. Check the selected input device and microphone permission."
                    .to_string()
            } else {
                format!("Microphone error: {message}")
            }
        }
        AppError::Permission(message) => format!("Permission required: {message}"),
        AppError::Model(message) => format!("Model error: {message}"),
        AppError::Transcription(message) => format!("Transcription failed: {message}"),
        AppError::TextInsertion(message) => format!("Text insertion failed: {message}"),
        AppError::Shortcut(message) => format!("Global shortcut error: {message}"),
        AppError::State(message) => format!("Dictation state error: {message}"),
        AppError::Platform(message) => format!("Platform error: {message}"),
        AppError::Config(message) => format!("Configuration error: {message}"),
        AppError::Io(message) => format!("I/O error: {message}"),
        AppError::Json(message) => format!("Settings file error: {message}"),
        AppError::Network(message) => format!("Download interrupted: {message}"),
    }
}
