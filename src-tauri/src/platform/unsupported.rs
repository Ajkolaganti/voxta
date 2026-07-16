use crate::{
    error::{AppError, AppResult},
    focus::FocusSnapshot,
    permissions::{PermissionGrant, PermissionStatus},
};

pub fn capture_focus() -> AppResult<FocusSnapshot> {
    Ok(FocusSnapshot::default())
}

pub fn restore_focus(_snapshot: &FocusSnapshot) -> AppResult<()> {
    Ok(())
}

pub fn permission_status() -> PermissionStatus {
    PermissionStatus {
        microphone: PermissionGrant::Unknown,
        accessibility: PermissionGrant::Unknown,
    }
}

pub fn open_permission_settings(permission: &str) -> AppResult<()> {
    Err(AppError::Permission(format!(
        "permission settings are not implemented for this platform: {permission}"
    )))
}
