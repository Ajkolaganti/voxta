#[derive(Debug, Clone, Default)]
pub struct CommandHistory {
    snapshots: Vec<String>,
}

impl CommandHistory {
    pub fn push(&mut self, text: &str) {
        self.snapshots.push(text.to_string());
    }

    pub fn undo(&mut self) -> Option<String> {
        self.snapshots.pop()
    }
}
