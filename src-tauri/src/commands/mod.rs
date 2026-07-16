use crate::{
    audio::{list_input_devices, CpalAudioRecorder},
    config::AppConfig,
    error::AppResult,
    models::{ModelDownloadProgress, ModelInfo},
    permissions::PermissionStatus,
    runtime::RuntimeStatus,
    transcription::cleanup::cleanup_transcript,
    AppState,
};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> AppConfig {
    state.config.read().clone()
}

#[tauri::command]
pub fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> AppResult<AppConfig> {
    let config = config.validate();
    state.config_store.save(&config)?;
    *state.config.write() = config.clone();
    crate::tray::sync_autostart(&app, config.launch_at_login)?;
    Ok(config)
}

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> RuntimeStatus {
    state.runtime.status()
}

#[tauri::command]
pub fn list_microphones() -> AppResult<Vec<crate::audio::MicrophoneDevice>> {
    list_input_devices()
}

#[tauri::command]
pub fn list_models(state: State<'_, AppState>) -> Vec<ModelInfo> {
    state.model_manager.list()
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    state: State<'_, AppState>,
    model_id: String,
) -> AppResult<ModelInfo> {
    state
        .model_manager
        .download(&model_id, move |progress: ModelDownloadProgress| {
            if let Err(err) = app.emit("model-download-progress", progress) {
                eprintln!("[Voxta] failed to emit model download progress: {err}");
            }
        })
        .await
}

#[tauri::command]
pub fn delete_model(state: State<'_, AppState>, model_id: String) -> AppResult<Vec<ModelInfo>> {
    state.model_manager.delete(&model_id)
}

#[tauri::command]
pub fn get_permission_status() -> PermissionStatus {
    crate::platform::permission_status()
}

#[tauri::command]
pub fn open_permission_settings(permission: String) -> AppResult<()> {
    crate::platform::open_permission_settings(&permission)
}

#[tauri::command]
pub fn test_microphone(microphone_id: String) -> AppResult<String> {
    let session = CpalAudioRecorder::start(Some(&microphone_id))?;
    std::thread::sleep(Duration::from_millis(650));
    let audio = session.stop();
    if audio.is_silent() {
        Ok("No microphone level detected.".to_string())
    } else {
        Ok("Microphone input detected.".to_string())
    }
}

#[tauri::command]
pub fn cleanup_test_transcript(state: State<'_, AppState>, text: String) -> String {
    cleanup_transcript(&text, &state.config.read())
}
