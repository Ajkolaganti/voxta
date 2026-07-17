use crate::{
    config::{AppConfig, ShortcutBehavior},
    error::AppError,
};
#[cfg(target_os = "macos")]
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
#[cfg(target_os = "macos")]
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventField,
};
use parking_lot::{Mutex, RwLock};
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
    trigger: KeyCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum KeyCode {
    ControlLeft,
    ControlRight,
    Alt,
    AltGr,
    ShiftLeft,
    ShiftRight,
    MetaLeft,
    MetaRight,
    Space,
    Escape,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyEvent {
    Press(KeyCode),
    Release(KeyCode),
}

impl KeyEvent {
    fn key(self) -> KeyCode {
        match self {
            KeyEvent::Press(key) | KeyEvent::Release(key) => key,
        }
    }
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
                "space" => trigger = Some(KeyCode::Space),
                "escape" | "esc" => trigger = Some(KeyCode::Escape),
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

    fn matches(&self, pressed: &HashSet<KeyCode>) -> bool {
        pressed.contains(&self.trigger)
            && self
                .modifiers
                .iter()
                .all(|modifier| modifier_is_pressed(*modifier, pressed))
    }

    fn contains_key(&self, key: KeyCode) -> bool {
        self.trigger == key
            || key_modifier(key).is_some_and(|modifier| self.modifiers.contains(&modifier))
    }
}

#[derive(Debug)]
pub struct ShortcutState {
    pressed: HashSet<KeyCode>,
    active: bool,
}

impl ShortcutState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            active: false,
        }
    }

    fn handle(
        &mut self,
        event: KeyEvent,
        shortcut: &Shortcut,
        behavior: ShortcutBehavior,
    ) -> Option<ShortcutEvent> {
        match behavior {
            ShortcutBehavior::Hold => self.handle_hold(event, shortcut),
            ShortcutBehavior::Toggle => self.handle_toggle(event, shortcut),
        }
    }

    fn handle_hold(&mut self, event: KeyEvent, shortcut: &Shortcut) -> Option<ShortcutEvent> {
        match event {
            KeyEvent::Press(KeyCode::Escape) if self.active => {
                self.active = false;
                self.pressed.clear();
                Some(ShortcutEvent::Cancel)
            }
            KeyEvent::Press(key) => {
                self.pressed.insert(key);
                if !self.active && shortcut.matches(&self.pressed) {
                    self.active = true;
                    Some(ShortcutEvent::Start)
                } else {
                    None
                }
            }
            KeyEvent::Release(key) => {
                let was_active = self.active;
                self.pressed.remove(&key);
                if was_active && key == shortcut.trigger {
                    self.active = false;
                    Some(ShortcutEvent::Stop)
                } else {
                    None
                }
            }
        }
    }

    fn handle_toggle(&mut self, event: KeyEvent, shortcut: &Shortcut) -> Option<ShortcutEvent> {
        match event {
            KeyEvent::Press(KeyCode::Escape) if self.active => {
                self.active = false;
                self.pressed.clear();
                Some(ShortcutEvent::Cancel)
            }
            KeyEvent::Press(key) => {
                let was_already_pressed = !self.pressed.insert(key);
                if was_already_pressed {
                    return None;
                }

                if shortcut.matches(&self.pressed) {
                    if self.active {
                        self.active = false;
                        self.pressed.clear();
                        Some(ShortcutEvent::Stop)
                    } else {
                        self.active = true;
                        Some(ShortcutEvent::Start)
                    }
                } else {
                    None
                }
            }
            KeyEvent::Release(key) => {
                self.pressed.remove(&key);
                None
            }
        }
    }

    fn should_swallow(&self, event: KeyEvent, shortcut: &Shortcut) -> bool {
        let key = event.key();
        self.active && (key == KeyCode::Escape || shortcut.contains_key(key))
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

#[cfg(target_os = "macos")]
fn spawn_grabber(config: Arc<RwLock<AppConfig>>, sender: Sender<ShortcutEvent>) {
    thread::spawn(move || {
        let error_sender = sender.clone();
        if let Err(err) = run_macos_event_tap(config, sender) {
            let _ = error_sender.send(ShortcutEvent::Error(format!(
                "global shortcut listener failed: {err}"
            )));
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn spawn_grabber(config: Arc<RwLock<AppConfig>>, sender: Sender<ShortcutEvent>) {
    thread::spawn(move || {
        let state = Mutex::new(ShortcutState::new());
        let callback_sender = sender.clone();
        let callback = move |event: rdev::Event| -> Option<rdev::Event> {
            let Some(key_event) = rdev_key_event(event.event_type) else {
                return Some(event);
            };

            if process_shortcut_event(key_event, &config, &state, &callback_sender) {
                None
            } else {
                Some(event)
            }
        };

        if let Err(err) = rdev::grab(callback) {
            let _ = sender.send(ShortcutEvent::Error(format!(
                "global shortcut listener failed: {err:?}"
            )));
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn process_shortcut_event(
    event: KeyEvent,
    config: &Arc<RwLock<AppConfig>>,
    state: &Mutex<ShortcutState>,
    sender: &Sender<ShortcutEvent>,
) -> bool {
    let (shortcut_text, behavior) = {
        let config = config.read();
        (config.shortcut.clone(), config.shortcut_behavior)
    };
    let shortcut = match Shortcut::parse(&shortcut_text) {
        Ok(shortcut) => shortcut,
        Err(err) => {
            let _ = sender.send(ShortcutEvent::Error(err.to_string()));
            return false;
        }
    };

    let mut state = state.lock();
    let shortcut_event = state.handle(event, &shortcut, behavior);
    let should_swallow = shortcut_event.is_some() || state.should_swallow(event, &shortcut);
    if let Some(shortcut_event) = shortcut_event {
        let _ = sender.send(shortcut_event);
    }

    should_swallow
}

#[cfg(target_os = "macos")]
fn run_macos_event_tap(
    config: Arc<RwLock<AppConfig>>,
    sender: Sender<ShortcutEvent>,
) -> Result<(), &'static str> {
    if crate::platform::macos::input_monitoring_status()
        != crate::permissions::PermissionGrant::Granted
    {
        let _ = crate::platform::macos::request_input_monitoring();
    }

    let state = Mutex::new(ShortcutState::new());
    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![
            CGEventType::KeyDown,
            CGEventType::KeyUp,
            CGEventType::FlagsChanged,
        ],
        move |_proxy, event_type, event| {
            let key_event = macos_key_event(event_type, event);
            if key_event.is_none() && !matches!(event_type, CGEventType::FlagsChanged) {
                return Some(event.clone());
            }

            let forwarded = event.clone();
            if process_macos_shortcut_event(key_event, event.get_flags(), &config, &state, &sender)
            {
                forwarded.set_type(CGEventType::Null);
            }
            Some(forwarded)
        },
    )
    .map_err(|_| {
        "event tap unavailable; grant Accessibility and Input Monitoring permission to Voxta, then restart"
    })?;

    let current = CFRunLoop::get_current();
    let loop_source = tap
        .mach_port
        .create_runloop_source(0)
        .map_err(|_| "failed to create macOS event tap run loop source")?;
    unsafe {
        current.add_source(&loop_source, kCFRunLoopCommonModes);
    }
    tap.enable();
    CFRunLoop::run_current();
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_key_event(event_type: CGEventType, event: &CGEvent) -> Option<KeyEvent> {
    let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
    match event_type {
        CGEventType::KeyDown => macos_keycode_to_key(keycode).map(KeyEvent::Press),
        CGEventType::KeyUp => macos_keycode_to_key(keycode).map(KeyEvent::Release),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn process_macos_shortcut_event(
    event: Option<KeyEvent>,
    flags: CGEventFlags,
    config: &Arc<RwLock<AppConfig>>,
    state: &Mutex<ShortcutState>,
    sender: &Sender<ShortcutEvent>,
) -> bool {
    let (shortcut_text, behavior) = {
        let config = config.read();
        (config.shortcut.clone(), config.shortcut_behavior)
    };
    let shortcut = match Shortcut::parse(&shortcut_text) {
        Ok(shortcut) => shortcut,
        Err(err) => {
            let _ = sender.send(ShortcutEvent::Error(err.to_string()));
            return false;
        }
    };

    let mut state = state.lock();
    sync_macos_modifiers(&mut state.pressed, flags);

    let Some(event) = event else {
        return false;
    };

    let shortcut_event = state.handle(event, &shortcut, behavior);
    let should_swallow = shortcut_event.is_some() || state.should_swallow(event, &shortcut);
    if let Some(shortcut_event) = shortcut_event {
        let _ = sender.send(shortcut_event);
    }

    should_swallow
}

#[cfg(target_os = "macos")]
fn sync_macos_modifiers(pressed: &mut HashSet<KeyCode>, flags: CGEventFlags) {
    set_modifier_group(
        pressed,
        flags.contains(CGEventFlags::CGEventFlagControl),
        &[KeyCode::ControlLeft, KeyCode::ControlRight],
        KeyCode::ControlLeft,
    );
    set_modifier_group(
        pressed,
        flags.contains(CGEventFlags::CGEventFlagAlternate),
        &[KeyCode::Alt, KeyCode::AltGr],
        KeyCode::Alt,
    );
    set_modifier_group(
        pressed,
        flags.contains(CGEventFlags::CGEventFlagShift),
        &[KeyCode::ShiftLeft, KeyCode::ShiftRight],
        KeyCode::ShiftLeft,
    );
    set_modifier_group(
        pressed,
        flags.contains(CGEventFlags::CGEventFlagCommand),
        &[KeyCode::MetaLeft, KeyCode::MetaRight],
        KeyCode::MetaLeft,
    );
}

#[cfg(target_os = "macos")]
fn set_modifier_group(
    pressed: &mut HashSet<KeyCode>,
    active: bool,
    keys: &[KeyCode],
    canonical: KeyCode,
) {
    if active {
        pressed.insert(canonical);
    } else {
        for key in keys {
            pressed.remove(key);
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_keycode_to_key(keycode: u16) -> Option<KeyCode> {
    match keycode {
        0 => Some(KeyCode::KeyA),
        1 => Some(KeyCode::KeyS),
        2 => Some(KeyCode::KeyD),
        3 => Some(KeyCode::KeyF),
        4 => Some(KeyCode::KeyH),
        5 => Some(KeyCode::KeyG),
        6 => Some(KeyCode::KeyZ),
        7 => Some(KeyCode::KeyX),
        8 => Some(KeyCode::KeyC),
        9 => Some(KeyCode::KeyV),
        11 => Some(KeyCode::KeyB),
        12 => Some(KeyCode::KeyQ),
        13 => Some(KeyCode::KeyW),
        14 => Some(KeyCode::KeyE),
        15 => Some(KeyCode::KeyR),
        16 => Some(KeyCode::KeyY),
        17 => Some(KeyCode::KeyT),
        18 => Some(KeyCode::Num1),
        19 => Some(KeyCode::Num2),
        20 => Some(KeyCode::Num3),
        21 => Some(KeyCode::Num4),
        22 => Some(KeyCode::Num6),
        23 => Some(KeyCode::Num5),
        25 => Some(KeyCode::Num9),
        26 => Some(KeyCode::Num7),
        28 => Some(KeyCode::Num8),
        29 => Some(KeyCode::Num0),
        31 => Some(KeyCode::KeyO),
        32 => Some(KeyCode::KeyU),
        34 => Some(KeyCode::KeyI),
        35 => Some(KeyCode::KeyP),
        37 => Some(KeyCode::KeyL),
        38 => Some(KeyCode::KeyJ),
        40 => Some(KeyCode::KeyK),
        45 => Some(KeyCode::KeyN),
        46 => Some(KeyCode::KeyM),
        49 => Some(KeyCode::Space),
        53 => Some(KeyCode::Escape),
        54 => Some(KeyCode::MetaRight),
        55 => Some(KeyCode::MetaLeft),
        56 => Some(KeyCode::ShiftLeft),
        58 => Some(KeyCode::Alt),
        59 => Some(KeyCode::ControlLeft),
        60 => Some(KeyCode::ShiftRight),
        61 => Some(KeyCode::AltGr),
        62 => Some(KeyCode::ControlRight),
        _ => None,
    }
}

#[cfg(not(target_os = "macos"))]
fn rdev_key_event(event: rdev::EventType) -> Option<KeyEvent> {
    match event {
        rdev::EventType::KeyPress(key) => rdev_key_to_key(key).map(KeyEvent::Press),
        rdev::EventType::KeyRelease(key) => rdev_key_to_key(key).map(KeyEvent::Release),
        _ => None,
    }
}

#[cfg(not(target_os = "macos"))]
fn rdev_key_to_key(key: rdev::Key) -> Option<KeyCode> {
    match key {
        rdev::Key::ControlLeft => Some(KeyCode::ControlLeft),
        rdev::Key::ControlRight => Some(KeyCode::ControlRight),
        rdev::Key::Alt => Some(KeyCode::Alt),
        rdev::Key::AltGr => Some(KeyCode::AltGr),
        rdev::Key::ShiftLeft => Some(KeyCode::ShiftLeft),
        rdev::Key::ShiftRight => Some(KeyCode::ShiftRight),
        rdev::Key::MetaLeft => Some(KeyCode::MetaLeft),
        rdev::Key::MetaRight => Some(KeyCode::MetaRight),
        rdev::Key::Space => Some(KeyCode::Space),
        rdev::Key::Escape => Some(KeyCode::Escape),
        rdev::Key::KeyA => Some(KeyCode::KeyA),
        rdev::Key::KeyB => Some(KeyCode::KeyB),
        rdev::Key::KeyC => Some(KeyCode::KeyC),
        rdev::Key::KeyD => Some(KeyCode::KeyD),
        rdev::Key::KeyE => Some(KeyCode::KeyE),
        rdev::Key::KeyF => Some(KeyCode::KeyF),
        rdev::Key::KeyG => Some(KeyCode::KeyG),
        rdev::Key::KeyH => Some(KeyCode::KeyH),
        rdev::Key::KeyI => Some(KeyCode::KeyI),
        rdev::Key::KeyJ => Some(KeyCode::KeyJ),
        rdev::Key::KeyK => Some(KeyCode::KeyK),
        rdev::Key::KeyL => Some(KeyCode::KeyL),
        rdev::Key::KeyM => Some(KeyCode::KeyM),
        rdev::Key::KeyN => Some(KeyCode::KeyN),
        rdev::Key::KeyO => Some(KeyCode::KeyO),
        rdev::Key::KeyP => Some(KeyCode::KeyP),
        rdev::Key::KeyQ => Some(KeyCode::KeyQ),
        rdev::Key::KeyR => Some(KeyCode::KeyR),
        rdev::Key::KeyS => Some(KeyCode::KeyS),
        rdev::Key::KeyT => Some(KeyCode::KeyT),
        rdev::Key::KeyU => Some(KeyCode::KeyU),
        rdev::Key::KeyV => Some(KeyCode::KeyV),
        rdev::Key::KeyW => Some(KeyCode::KeyW),
        rdev::Key::KeyX => Some(KeyCode::KeyX),
        rdev::Key::KeyY => Some(KeyCode::KeyY),
        rdev::Key::KeyZ => Some(KeyCode::KeyZ),
        rdev::Key::Num0 => Some(KeyCode::Num0),
        rdev::Key::Num1 => Some(KeyCode::Num1),
        rdev::Key::Num2 => Some(KeyCode::Num2),
        rdev::Key::Num3 => Some(KeyCode::Num3),
        rdev::Key::Num4 => Some(KeyCode::Num4),
        rdev::Key::Num5 => Some(KeyCode::Num5),
        rdev::Key::Num6 => Some(KeyCode::Num6),
        rdev::Key::Num7 => Some(KeyCode::Num7),
        rdev::Key::Num8 => Some(KeyCode::Num8),
        rdev::Key::Num9 => Some(KeyCode::Num9),
        _ => None,
    }
}

fn modifier_is_pressed(modifier: Modifier, pressed: &HashSet<KeyCode>) -> bool {
    match modifier {
        Modifier::Ctrl => {
            pressed.contains(&KeyCode::ControlLeft) || pressed.contains(&KeyCode::ControlRight)
        }
        Modifier::Alt => pressed.contains(&KeyCode::Alt) || pressed.contains(&KeyCode::AltGr),
        Modifier::Shift => {
            pressed.contains(&KeyCode::ShiftLeft) || pressed.contains(&KeyCode::ShiftRight)
        }
        Modifier::Meta => {
            pressed.contains(&KeyCode::MetaLeft) || pressed.contains(&KeyCode::MetaRight)
        }
    }
}

fn key_modifier(key: KeyCode) -> Option<Modifier> {
    match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => Some(Modifier::Ctrl),
        KeyCode::Alt | KeyCode::AltGr => Some(Modifier::Alt),
        KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(Modifier::Shift),
        KeyCode::MetaLeft | KeyCode::MetaRight => Some(Modifier::Meta),
        _ => None,
    }
}

fn letter_key(ch: char) -> Option<KeyCode> {
    match ch.to_ascii_lowercase() {
        'a' => Some(KeyCode::KeyA),
        'b' => Some(KeyCode::KeyB),
        'c' => Some(KeyCode::KeyC),
        'd' => Some(KeyCode::KeyD),
        'e' => Some(KeyCode::KeyE),
        'f' => Some(KeyCode::KeyF),
        'g' => Some(KeyCode::KeyG),
        'h' => Some(KeyCode::KeyH),
        'i' => Some(KeyCode::KeyI),
        'j' => Some(KeyCode::KeyJ),
        'k' => Some(KeyCode::KeyK),
        'l' => Some(KeyCode::KeyL),
        'm' => Some(KeyCode::KeyM),
        'n' => Some(KeyCode::KeyN),
        'o' => Some(KeyCode::KeyO),
        'p' => Some(KeyCode::KeyP),
        'q' => Some(KeyCode::KeyQ),
        'r' => Some(KeyCode::KeyR),
        's' => Some(KeyCode::KeyS),
        't' => Some(KeyCode::KeyT),
        'u' => Some(KeyCode::KeyU),
        'v' => Some(KeyCode::KeyV),
        'w' => Some(KeyCode::KeyW),
        'x' => Some(KeyCode::KeyX),
        'y' => Some(KeyCode::KeyY),
        'z' => Some(KeyCode::KeyZ),
        _ => None,
    }
}

fn digit_key(ch: char) -> Option<KeyCode> {
    match ch {
        '0' => Some(KeyCode::Num0),
        '1' => Some(KeyCode::Num1),
        '2' => Some(KeyCode::Num2),
        '3' => Some(KeyCode::Num3),
        '4' => Some(KeyCode::Num4),
        '5' => Some(KeyCode::Num5),
        '6' => Some(KeyCode::Num6),
        '7' => Some(KeyCode::Num7),
        '8' => Some(KeyCode::Num8),
        '9' => Some(KeyCode::Num9),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyCode, KeyEvent, Shortcut, ShortcutEvent, ShortcutState};
    use crate::config::ShortcutBehavior;

    #[test]
    fn parses_shortcut() {
        let shortcut = Shortcut::parse("Ctrl+Alt+Space").unwrap();
        assert_eq!(shortcut.trigger, KeyCode::Space);
    }

    #[test]
    fn emits_start_once_and_stop_on_release() {
        let shortcut = Shortcut::parse("Ctrl+Alt+Space").unwrap();
        let mut state = ShortcutState::new();
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::ControlLeft),
                &shortcut,
                ShortcutBehavior::Hold
            ),
            None
        );
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::Alt),
                &shortcut,
                ShortcutBehavior::Hold
            ),
            None
        );
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::Space),
                &shortcut,
                ShortcutBehavior::Hold
            ),
            Some(ShortcutEvent::Start)
        );
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::Space),
                &shortcut,
                ShortcutBehavior::Hold
            ),
            None
        );
        assert_eq!(
            state.handle(
                KeyEvent::Release(KeyCode::Space),
                &shortcut,
                ShortcutBehavior::Hold
            ),
            Some(ShortcutEvent::Stop)
        );
    }

    #[test]
    fn toggle_mode_starts_and_stops_on_shortcut_presses() {
        let shortcut = Shortcut::parse("Ctrl+Alt+Space").unwrap();
        let mut state = ShortcutState::new();
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::ControlLeft),
                &shortcut,
                ShortcutBehavior::Toggle
            ),
            None
        );
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::Alt),
                &shortcut,
                ShortcutBehavior::Toggle
            ),
            None
        );
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::Space),
                &shortcut,
                ShortcutBehavior::Toggle
            ),
            Some(ShortcutEvent::Start)
        );
        assert_eq!(
            state.handle(
                KeyEvent::Release(KeyCode::Space),
                &shortcut,
                ShortcutBehavior::Toggle
            ),
            None
        );
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::Space),
                &shortcut,
                ShortcutBehavior::Toggle
            ),
            Some(ShortcutEvent::Stop)
        );
    }

    #[test]
    fn toggle_mode_ignores_key_repeat_until_released() {
        let shortcut = Shortcut::parse("Ctrl+Alt+Space").unwrap();
        let mut state = ShortcutState::new();
        state.handle(
            KeyEvent::Press(KeyCode::ControlLeft),
            &shortcut,
            ShortcutBehavior::Toggle,
        );
        state.handle(
            KeyEvent::Press(KeyCode::Alt),
            &shortcut,
            ShortcutBehavior::Toggle,
        );
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::Space),
                &shortcut,
                ShortcutBehavior::Toggle
            ),
            Some(ShortcutEvent::Start)
        );
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::Space),
                &shortcut,
                ShortcutBehavior::Toggle
            ),
            None
        );
    }

    #[test]
    fn escape_cancels_active_recording() {
        let shortcut = Shortcut::parse("Ctrl+Alt+Space").unwrap();
        let mut state = ShortcutState::new();
        state.handle(
            KeyEvent::Press(KeyCode::ControlLeft),
            &shortcut,
            ShortcutBehavior::Hold,
        );
        state.handle(
            KeyEvent::Press(KeyCode::Alt),
            &shortcut,
            ShortcutBehavior::Hold,
        );
        state.handle(
            KeyEvent::Press(KeyCode::Space),
            &shortcut,
            ShortcutBehavior::Hold,
        );
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::Escape),
                &shortcut,
                ShortcutBehavior::Hold
            ),
            Some(ShortcutEvent::Cancel)
        );
    }

    #[test]
    fn stale_modifiers_do_not_trigger_without_the_shortcut_trigger() {
        let shortcut = Shortcut::parse("Ctrl+Alt+Space").unwrap();
        let mut state = ShortcutState::new();
        state.pressed.insert(KeyCode::ControlLeft);
        state.pressed.insert(KeyCode::Alt);
        assert_eq!(
            state.handle(
                KeyEvent::Press(KeyCode::KeyA),
                &shortcut,
                ShortcutBehavior::Hold
            ),
            None
        );
        assert_eq!(
            state.handle(
                KeyEvent::Release(KeyCode::KeyA),
                &shortcut,
                ShortcutBehavior::Hold
            ),
            None
        );
    }
}
