use crate::{
    error::{AppError, AppResult},
    focus::FocusSnapshot,
    permissions::{PermissionGrant, PermissionStatus},
};

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, MessageBeep, SetForegroundWindow,
    },
};

pub fn capture_focus() -> AppResult<FocusSnapshot> {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = GetForegroundWindow();
        let window_title = window_title(hwnd);
        Ok(FocusSnapshot {
            app_name: None,
            window_title,
            process_id: None,
            native_window_id: Some(hwnd.0 as isize),
            secure_input: false,
        })
    }

    #[cfg(not(target_os = "windows"))]
    Ok(FocusSnapshot::default())
}

pub fn restore_focus(snapshot: &FocusSnapshot) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = snapshot
            .native_window_id
            .map(|id| HWND(id as _))
            .unwrap_or_else(GetForegroundWindow);
        if hwnd.0 != 0 {
            let restored = SetForegroundWindow(hwnd);
            if !restored.as_bool() {
                return Err(AppError::Platform(
                    "could not restore focus to the previous window; it may be elevated or unavailable"
                        .to_string(),
                ));
            }
        }
    }
    Ok(())
}

pub fn permission_status() -> PermissionStatus {
    PermissionStatus {
        microphone: PermissionGrant::Unknown,
        accessibility: PermissionGrant::NotRequired,
        input_monitoring: PermissionGrant::NotRequired,
    }
}

pub fn open_permission_settings(permission: &str) -> AppResult<()> {
    let uri = match permission {
        "microphone" => "ms-settings:privacy-microphone",
        "accessibility" => "ms-settings:easeofaccess-keyboard",
        _ => {
            return Err(AppError::Permission(format!(
                "unknown Windows permission pane: {permission}"
            )))
        }
    };

    std::process::Command::new("cmd")
        .args(["/C", "start", "", uri])
        .spawn()
        .map(|_| ())
        .map_err(|err| AppError::Permission(format!("could not open Windows Settings: {err}")))
}

#[cfg(target_os = "windows")]
unsafe fn window_title(hwnd: HWND) -> Option<String> {
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 {
        return None;
    }
    let mut buffer = vec![0_u16; len as usize + 1];
    let copied = GetWindowTextW(hwnd, &mut buffer);
    if copied <= 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buffer[..copied as usize]))
}

pub fn play_start_sound() {
    #[cfg(target_os = "windows")]
    unsafe {
        let _ = MessageBeep(0xFFFFFFFF);
    }
}

pub fn play_stop_sound() {
    #[cfg(target_os = "windows")]
    unsafe {
        let _ = MessageBeep(0xFFFFFFFF);
    }
}
