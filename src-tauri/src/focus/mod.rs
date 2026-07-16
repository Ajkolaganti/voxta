use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusSnapshot {
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub process_id: Option<u32>,
    pub native_window_id: Option<isize>,
    pub secure_input: bool,
}

pub trait FocusedApplicationTracker: Send + Sync {
    fn capture(&self) -> AppResult<FocusSnapshot>;
    fn restore(&self, snapshot: &FocusSnapshot) -> AppResult<()>;
}

#[derive(Debug, Default)]
pub struct PlatformFocusTracker;

impl FocusedApplicationTracker for PlatformFocusTracker {
    fn capture(&self) -> AppResult<FocusSnapshot> {
        crate::platform::capture_focus()
    }

    fn restore(&self, snapshot: &FocusSnapshot) -> AppResult<()> {
        crate::platform::restore_focus(snapshot)
    }
}
