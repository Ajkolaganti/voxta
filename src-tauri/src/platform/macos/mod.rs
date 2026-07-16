use crate::{
    error::{AppError, AppResult},
    focus::FocusSnapshot,
    permissions::{PermissionGrant, PermissionStatus},
};
use std::process::Command;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[link(name = "AudioToolbox", kind = "framework")]
extern "C" {
    fn AudioServicesPlaySystemSound(in_system_sound_id: u32);
}

pub fn capture_focus() -> AppResult<FocusSnapshot> {
    if unsafe { !AXIsProcessTrusted() } {
        return Err(AppError::Permission(
            "macOS Accessibility permission is required to track the focused app and insert text"
                .to_string(),
        ));
    }

    let app_name = run_osascript(
        "tell application \"System Events\" to get name of first application process whose frontmost is true",
    )
    .ok();
    let process_id = run_osascript(
        "tell application \"System Events\" to get unix id of first application process whose frontmost is true",
    )
    .ok()
    .and_then(|pid| pid.parse::<u32>().ok());
    let window_title = run_osascript(
        "tell application \"System Events\" to tell first application process whose frontmost is true to get name of front window",
    )
    .ok();
    let secure_input = is_secure_focused_element();

    Ok(FocusSnapshot {
        app_name,
        window_title,
        process_id,
        native_window_id: None,
        secure_input,
    })
}

pub fn restore_focus(snapshot: &FocusSnapshot) -> AppResult<()> {
    let Some(app_name) = snapshot.app_name.as_ref() else {
        return Ok(());
    };

    let escaped = app_name.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("tell application \"{escaped}\" to activate");
    run_osascript(&script).map(|_| ())
}

pub fn permission_status() -> PermissionStatus {
    let accessibility = unsafe {
        if AXIsProcessTrusted() {
            PermissionGrant::Granted
        } else {
            PermissionGrant::Denied
        }
    };

    PermissionStatus {
        microphone: PermissionGrant::Unknown,
        accessibility,
    }
}

pub fn open_permission_settings(permission: &str) -> AppResult<()> {
    let url = match permission {
        "microphone" => {
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
        }
        "accessibility" => {
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
        }
        _ => {
            return Err(AppError::Permission(format!(
                "unknown macOS permission pane: {permission}"
            )))
        }
    };

    Command::new("open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|err| AppError::Permission(format!("could not open System Settings: {err}")))
}

fn run_osascript(script: &str) -> AppResult<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|err| AppError::Platform(format!("could not run osascript: {err}")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(AppError::Platform(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

fn is_secure_focused_element() -> bool {
    let script = r#"
tell application "System Events"
  set frontApp to first application process whose frontmost is true
  try
    set focusedElement to focused UI element of frontApp
    set roleValue to ""
    set subroleValue to ""
    set descriptionValue to ""
    try
      set roleValue to role of focusedElement as string
    end try
    try
      set subroleValue to subrole of focusedElement as string
    end try
    try
      set descriptionValue to description of focusedElement as string
    end try
    set combinedValue to roleValue & " " & subroleValue & " " & descriptionValue
    if combinedValue contains "secure" or combinedValue contains "password" then
      return "true"
    end if
  end try
end tell
return "false"
"#;

    run_osascript(script)
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn play_start_sound() {
    unsafe { AudioServicesPlaySystemSound(1104) }
}

pub fn play_stop_sound() {
    unsafe { AudioServicesPlaySystemSound(1105) }
}
