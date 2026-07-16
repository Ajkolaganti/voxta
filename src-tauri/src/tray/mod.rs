use crate::{
    error::{AppError, AppResult},
    AppState,
};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Runtime,
};
use tauri_plugin_autostart::ManagerExt;

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let start_pause = MenuItem::with_id(
        app,
        "toggle-enabled",
        "Start or pause Voxta",
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let current_model = MenuItem::with_id(
        app,
        "current-model",
        "Current model: base",
        false,
        None::<&str>,
    )?;
    let launch = MenuItem::with_id(
        app,
        "toggle-launch",
        "Toggle launch at login",
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &start_pause,
            &settings,
            &separator,
            &current_model,
            &launch,
            &separator,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id("main")
        .tooltip("Voxta")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => show_settings(app),
            "toggle-enabled" => toggle_enabled(app),
            "toggle-launch" => toggle_launch_at_login(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

pub fn show_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        if let Err(err) = window.show() {
            eprintln!("[Voxta] could not show settings window: {err}");
        }
        if let Err(err) = window.set_focus() {
            eprintln!("[Voxta] could not focus settings window: {err}");
        }
    }
}

pub fn sync_autostart<R: Runtime>(app: &AppHandle<R>, enabled: bool) -> AppResult<()> {
    let autostart = app.autolaunch();
    let result = if enabled {
        autostart.enable()
    } else {
        autostart.disable()
    };
    if let Err(err) = result {
        return Err(AppError::Platform(format!(
            "launch-at-login update failed: {err}"
        )));
    }
    Ok(())
}

fn toggle_enabled<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let mut config = state.config.write();
    config.enabled = !config.enabled;
    if let Err(err) = state.config_store.save(&config) {
        eprintln!("[Voxta] could not save enabled state: {err}");
    }
}

fn toggle_launch_at_login<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let mut config = state.config.write();
    config.launch_at_login = !config.launch_at_login;
    if let Err(err) = sync_autostart(app, config.launch_at_login) {
        eprintln!("[Voxta] {err}");
    }
    if let Err(err) = state.config_store.save(&config) {
        eprintln!("[Voxta] could not save launch-at-login setting: {err}");
    }
}
