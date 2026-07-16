use crate::{error::AppResult, focus::FocusSnapshot, permissions::PermissionStatus};

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod unsupported;
#[cfg(target_os = "windows")]
pub mod windows;

pub fn capture_focus() -> AppResult<FocusSnapshot> {
    #[cfg(target_os = "macos")]
    return macos::capture_focus();
    #[cfg(target_os = "windows")]
    return windows::capture_focus();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return unsupported::capture_focus();
}

pub fn restore_focus(snapshot: &FocusSnapshot) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    return macos::restore_focus(snapshot);
    #[cfg(target_os = "windows")]
    return windows::restore_focus(snapshot);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return unsupported::restore_focus(snapshot);
}

pub fn permission_status() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    return macos::permission_status();
    #[cfg(target_os = "windows")]
    return windows::permission_status();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return unsupported::permission_status();
}

pub fn open_permission_settings(permission: &str) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    return macos::open_permission_settings(permission);
    #[cfg(target_os = "windows")]
    return windows::open_permission_settings(permission);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return unsupported::open_permission_settings(permission);
}

pub fn play_start_sound() {
    #[cfg(target_os = "macos")]
    macos::play_start_sound();
    #[cfg(target_os = "windows")]
    windows::play_start_sound();
}

pub fn play_stop_sound() {
    #[cfg(target_os = "macos")]
    macos::play_stop_sound();
    #[cfg(target_os = "windows")]
    windows::play_stop_sound();
}
