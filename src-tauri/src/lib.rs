pub mod ai_cleanup;
pub mod audio;
pub mod commands;
pub mod config;
pub mod error;
pub mod focus;
pub mod hotkeys;
pub mod models;
pub mod permissions;
pub mod platform;
pub mod runtime;
pub mod state;
pub mod streaming;
pub mod text_injection;
pub mod transcription;
pub mod tray;
pub mod voice_commands;

use crate::{
    config::{AppConfig, ConfigStore},
    hotkeys::{GlobalShortcutManager, ShortcutEvent},
    models::ModelManager,
    runtime::DictationRuntime,
    transcription::{whisper::WhisperCppEngine, TranscriptionEngine},
};
use parking_lot::RwLock;
use std::{env, sync::Arc, thread};
use tauri::Manager;

pub struct AppState {
    pub config_store: ConfigStore,
    pub config: Arc<RwLock<AppConfig>>,
    pub model_manager: Arc<ModelManager>,
    pub runtime: Arc<DictationRuntime>,
}

pub fn run() {
    let config_store = ConfigStore::new().expect("could not initialize configuration");
    let config = match config_store.load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("[Voxta] configuration was reset: {err}");
            AppConfig::default()
        }
    };
    let config = Arc::new(RwLock::new(config));
    let model_manager = Arc::new(ModelManager::new().expect("could not initialize model manager"));
    let transcriber: Arc<dyn TranscriptionEngine> =
        Arc::new(WhisperCppEngine::new(model_manager.clone()));
    let runtime = DictationRuntime::new(config.clone(), transcriber);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .manage(AppState {
            config_store,
            config,
            model_manager,
            runtime,
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_status,
            commands::list_microphones,
            commands::list_models,
            commands::download_model,
            commands::delete_model,
            commands::get_permission_status,
            commands::open_permission_settings,
            commands::test_microphone,
            commands::cleanup_test_transcript,
            commands::test_voice_command_parser,
            commands::cleanup_provider_status,
            commands::save_cleanup_api_key,
            commands::has_cleanup_api_key
        ])
        .setup(|app| {
            let state = app.state::<AppState>();
            state.runtime.set_app_handle(app.handle().clone());
            tray::create_tray(app.handle())?;
            if let Err(err) =
                tray::sync_autostart(app.handle(), state.config.read().launch_at_login)
            {
                eprintln!("[Voxta] launch-at-login setup failed: {err}");
            }
            start_hotkey_loop(state.config.clone(), state.runtime.clone());

            let background = env::args().any(|arg| arg == "--background");
            if !background {
                tray::show_settings(app.handle());
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Voxta")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                tray::show_settings(app);
            }
        });
}

fn start_hotkey_loop(config: Arc<RwLock<AppConfig>>, runtime: Arc<DictationRuntime>) {
    let manager = GlobalShortcutManager::start(config);
    let receiver = manager.into_receiver();

    thread::spawn(move || {
        for event in receiver {
            match event {
                ShortcutEvent::Start => {
                    if let Err(err) = runtime.start_recording() {
                        runtime.report_error(&format!("Could not start recording: {err}"));
                        eprintln!("[Voxta] start recording failed: {err}");
                    }
                }
                ShortcutEvent::Stop => {
                    if let Err(err) = runtime.stop_recording() {
                        runtime.report_error(&format!("Could not stop recording: {err}"));
                        eprintln!("[Voxta] stop recording failed: {err}");
                    }
                }
                ShortcutEvent::Cancel => {
                    if let Err(err) = runtime.cancel_recording() {
                        runtime.report_error(&format!("Could not cancel recording: {err}"));
                        eprintln!("[Voxta] cancel failed: {err}");
                    }
                }
                ShortcutEvent::Error(err) => {
                    runtime.report_error(&format!("Global shortcut error: {err}"));
                    eprintln!("[Voxta] shortcut error: {err}");
                }
            }
        }
    });
}
