use crate::{
    config::InsertionMode,
    error::{AppError, AppResult},
    focus::FocusSnapshot,
};
use arboard::Clipboard;
use async_trait::async_trait;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{thread, time::Duration};

#[async_trait]
pub trait TextInjector: Send + Sync {
    async fn insert_text(
        &self,
        text: &str,
        focus: &FocusSnapshot,
        mode: InsertionMode,
    ) -> AppResult<()>;
}

#[derive(Debug, Default)]
pub struct PlatformTextInjector;

#[async_trait]
impl TextInjector for PlatformTextInjector {
    async fn insert_text(
        &self,
        text: &str,
        focus: &FocusSnapshot,
        mode: InsertionMode,
    ) -> AppResult<()> {
        if text.trim().is_empty() {
            return Ok(());
        }
        if focus.secure_input {
            return Err(AppError::TextInsertion(
                "refusing to insert dictation into a secure input field".to_string(),
            ));
        }

        match mode {
            InsertionMode::Direct => direct_insert(text),
            InsertionMode::Clipboard => clipboard_insert(text),
            InsertionMode::Auto => direct_insert(text).or_else(|_| clipboard_insert(text)),
        }
    }
}

fn direct_insert(text: &str) -> AppResult<()> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|err| {
        AppError::TextInsertion(format!("could not initialize input backend: {err}"))
    })?;
    enigo
        .text(text)
        .map_err(|err| AppError::TextInsertion(format!("direct insertion failed: {err}")))
}

fn clipboard_insert(text: &str) -> AppResult<()> {
    let mut clipboard = Clipboard::new()
        .map_err(|err| AppError::TextInsertion(format!("could not access clipboard: {err}")))?;
    let snapshot = ClipboardSnapshot::capture(&mut clipboard);

    clipboard
        .set_text(text.to_string())
        .map_err(|err| AppError::TextInsertion(format!("could not set clipboard text: {err}")))?;
    paste_shortcut()?;
    thread::sleep(Duration::from_millis(80));
    snapshot.restore(&mut clipboard)?;
    Ok(())
}

fn paste_shortcut() -> AppResult<()> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|err| {
        AppError::TextInsertion(format!("could not initialize input backend: {err}"))
    })?;

    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };

    enigo
        .key(modifier, Direction::Press)
        .and_then(|_| enigo.key(Key::Unicode('v'), Direction::Click))
        .and_then(|_| enigo.key(modifier, Direction::Release))
        .map_err(|err| AppError::TextInsertion(format!("paste shortcut failed: {err}")))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardSnapshot {
    text: Option<String>,
}

impl ClipboardSnapshot {
    pub fn capture(clipboard: &mut Clipboard) -> Self {
        Self {
            text: clipboard.get_text().ok(),
        }
    }

    pub fn restore(self, clipboard: &mut Clipboard) -> AppResult<()> {
        match self.text {
            Some(text) => clipboard.set_text(text).map_err(|err| {
                AppError::TextInsertion(format!("clipboard restoration failed: {err}"))
            })?,
            None => clipboard.clear().map_err(|err| {
                AppError::TextInsertion(format!("clipboard restoration failed: {err}"))
            })?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ClipboardSnapshot;

    #[test]
    fn snapshot_keeps_previous_text_for_restore() {
        let snapshot = ClipboardSnapshot {
            text: Some("before".to_string()),
        };
        assert_eq!(snapshot.text.as_deref(), Some("before"));
    }

    #[test]
    fn empty_snapshot_represents_clear_clipboard() {
        let snapshot = ClipboardSnapshot { text: None };
        assert!(snapshot.text.is_none());
    }
}
