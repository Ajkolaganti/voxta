use crate::{config::AppConfig, error::AppError};
use parking_lot::{Mutex, RwLock};
use rdev::{grab, Event, EventType, Key};
use std::{
    collections::HashSet,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutEvent {
    Start,
    Stop,
    Cancel,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    modifiers: Vec<Modifier>,
    trigger: Key,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

impl Shortcut {
    pub fn parse(input: &str) -> Result<Self, AppError> {
        let mut modifiers = Vec::new();
        let mut trigger = None;

        for raw in input.split('+') {
            let part = raw.trim().to_ascii_lowercase();
            match part.as_str() {
                "ctrl" | "control" => modifiers.push(Modifier::Ctrl),
                "alt" | "option" => modifiers.push(Modifier::Alt),
                "shift" => modifiers.push(Modifier::Shift),
                "meta" | "cmd" | "command" | "super" | "win" | "windows" => {
                    modifiers.push(Modifier::Meta)
                }
                "space" => trigger = Some(Key::Space),
                "escape" | "esc" => trigger = Some(Key::Escape),
                key if key.len() == 1 => {
                    let ch = key.chars().next().unwrap();
                    trigger = letter_key(ch).or_else(|| digit_key(ch));
                }
                _ => {
                    return Err(AppError::Shortcut(format!(
                        "unsupported shortcut key: {raw}"
                    )))
                }
            }
        }

        let trigger = trigger.ok_or_else(|| {
            AppError::Shortcut("shortcut must include a non-modifier key".to_string())
        })?;

        modifiers.sort_by_key(|modifier| *modifier as u8);
        modifiers.dedup();

        if modifiers.is_empty() {
            return Err(AppError::Shortcut(
                "shortcut must include at least one modifier".to_string(),
            ));
        }

        Ok(Self { modifiers, trigger })
    }

    fn matches(&self, pressed: &HashSet<Key>) -> bool {
        pressed.contains(&self.trigger)
            && self
                .modifiers
                .iter()
                .all(|modifier| modifier_is_pressed(*modifier, pressed))
    }

    fn contains_key(&self, key: Key) -> bool {
        self.trigger == key
            || key_modifier(key).is_some_and(|modifier| self.modifiers.contains(&modifier))
    }
}

#[derive(Debug)]
pub struct ShortcutState {
    pressed: HashSet<Key>,
    active: bool,
}

impl ShortcutState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            active: false,
        }
    }

    pub fn handle(&mut self, event: EventType, shortcut: &Shortcut) -> Option<ShortcutEvent> {
        match event {
            EventType::KeyPress(Key::Escape) if self.active => {
                self.active = false;
                self.pressed.clear();
                Some(ShortcutEvent::Cancel)
            }
            EventType::KeyPress(key) => {
                self.pressed.insert(key);
                if !self.active && shortcut.matches(&self.pressed) {
                    self.active = true;
                    Some(ShortcutEvent::Start)
                } else {
                    None
                }
            }
            EventType::KeyRelease(key) => {
                let was_active = self.active;
                self.pressed.remove(&key);
                if was_active && key == shortcut.trigger {
                    self.active = false;
                    Some(ShortcutEvent::Stop)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn should_swallow(&self, event: &EventType, shortcut: &Shortcut) -> bool {
        match event {
            EventType::KeyPress(key) | EventType::KeyRelease(key) => {
                self.active && (*key == Key::Escape || shortcut.contains_key(*key))
            }
            _ => false,
        }
    }
}

impl Default for ShortcutState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GlobalShortcutManager {
    receiver: Receiver<ShortcutEvent>,
}

impl GlobalShortcutManager {
    pub fn start(config: Arc<RwLock<AppConfig>>) -> Self {
        let (sender, receiver) = mpsc::channel();
        spawn_grabber(config, sender);
        Self { receiver }
    }

    pub fn into_receiver(self) -> Receiver<ShortcutEvent> {
        self.receiver
    }
}

fn spawn_grabber(config: Arc<RwLock<AppConfig>>, sender: Sender<ShortcutEvent>) {
    thread::spawn(move || {
        let state = Mutex::new(ShortcutState::new());
        let callback_sender = sender.clone();
        let callback = move |event: Event| -> Option<Event> {
            let shortcut_text = config.read().shortcut.clone();
            let shortcut = match Shortcut::parse(&shortcut_text) {
                Ok(shortcut) => shortcut,
                Err(err) => {
                    let _ = callback_sender.send(ShortcutEvent::Error(err.to_string()));
                    return Some(event);
                }
            };

            let event_type = event.event_type;
            let mut state = state.lock();
            let shortcut_event = state.handle(event_type, &shortcut);
            if let Some(shortcut_event) = shortcut_event {
                let _ = callback_sender.send(shortcut_event);
            }

            if state.should_swallow(&event_type, &shortcut) {
                None
            } else {
                Some(event)
            }
        };

        if let Err(err) = grab(callback) {
            let _ = sender.send(ShortcutEvent::Error(format!(
                "global shortcut listener failed: {err:?}"
            )));
        }
    });
}

fn modifier_is_pressed(modifier: Modifier, pressed: &HashSet<Key>) -> bool {
    match modifier {
        Modifier::Ctrl => {
            pressed.contains(&Key::ControlLeft) || pressed.contains(&Key::ControlRight)
        }
        Modifier::Alt => pressed.contains(&Key::Alt) || pressed.contains(&Key::AltGr),
        Modifier::Shift => pressed.contains(&Key::ShiftLeft) || pressed.contains(&Key::ShiftRight),
        Modifier::Meta => pressed.contains(&Key::MetaLeft) || pressed.contains(&Key::MetaRight),
    }
}

fn key_modifier(key: Key) -> Option<Modifier> {
    match key {
        Key::ControlLeft | Key::ControlRight => Some(Modifier::Ctrl),
        Key::Alt | Key::AltGr => Some(Modifier::Alt),
        Key::ShiftLeft | Key::ShiftRight => Some(Modifier::Shift),
        Key::MetaLeft | Key::MetaRight => Some(Modifier::Meta),
        _ => None,
    }
}

fn letter_key(ch: char) -> Option<Key> {
    match ch.to_ascii_lowercase() {
        'a' => Some(Key::KeyA),
        'b' => Some(Key::KeyB),
        'c' => Some(Key::KeyC),
        'd' => Some(Key::KeyD),
        'e' => Some(Key::KeyE),
        'f' => Some(Key::KeyF),
        'g' => Some(Key::KeyG),
        'h' => Some(Key::KeyH),
        'i' => Some(Key::KeyI),
        'j' => Some(Key::KeyJ),
        'k' => Some(Key::KeyK),
        'l' => Some(Key::KeyL),
        'm' => Some(Key::KeyM),
        'n' => Some(Key::KeyN),
        'o' => Some(Key::KeyO),
        'p' => Some(Key::KeyP),
        'q' => Some(Key::KeyQ),
        'r' => Some(Key::KeyR),
        's' => Some(Key::KeyS),
        't' => Some(Key::KeyT),
        'u' => Some(Key::KeyU),
        'v' => Some(Key::KeyV),
        'w' => Some(Key::KeyW),
        'x' => Some(Key::KeyX),
        'y' => Some(Key::KeyY),
        'z' => Some(Key::KeyZ),
        _ => None,
    }
}

fn digit_key(ch: char) -> Option<Key> {
    match ch {
        '0' => Some(Key::Num0),
        '1' => Some(Key::Num1),
        '2' => Some(Key::Num2),
        '3' => Some(Key::Num3),
        '4' => Some(Key::Num4),
        '5' => Some(Key::Num5),
        '6' => Some(Key::Num6),
        '7' => Some(Key::Num7),
        '8' => Some(Key::Num8),
        '9' => Some(Key::Num9),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Shortcut, ShortcutEvent, ShortcutState};
    use rdev::{EventType, Key};

    #[test]
    fn parses_shortcut() {
        let shortcut = Shortcut::parse("Ctrl+Alt+Space").unwrap();
        assert_eq!(shortcut.trigger, Key::Space);
    }

    #[test]
    fn emits_start_once_and_stop_on_release() {
        let shortcut = Shortcut::parse("Ctrl+Alt+Space").unwrap();
        let mut state = ShortcutState::new();
        assert_eq!(
            state.handle(EventType::KeyPress(Key::ControlLeft), &shortcut),
            None
        );
        assert_eq!(state.handle(EventType::KeyPress(Key::Alt), &shortcut), None);
        assert_eq!(
            state.handle(EventType::KeyPress(Key::Space), &shortcut),
            Some(ShortcutEvent::Start)
        );
        assert_eq!(
            state.handle(EventType::KeyPress(Key::Space), &shortcut),
            None
        );
        assert_eq!(
            state.handle(EventType::KeyRelease(Key::Space), &shortcut),
            Some(ShortcutEvent::Stop)
        );
    }

    #[test]
    fn escape_cancels_active_recording() {
        let shortcut = Shortcut::parse("Ctrl+Alt+Space").unwrap();
        let mut state = ShortcutState::new();
        state.handle(EventType::KeyPress(Key::ControlLeft), &shortcut);
        state.handle(EventType::KeyPress(Key::Alt), &shortcut);
        state.handle(EventType::KeyPress(Key::Space), &shortcut);
        assert_eq!(
            state.handle(EventType::KeyPress(Key::Escape), &shortcut),
            Some(ShortcutEvent::Cancel)
        );
    }
}
