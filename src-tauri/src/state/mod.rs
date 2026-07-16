use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
    Inserting,
    Cancelled,
    Error,
}

#[derive(Debug, Clone)]
pub struct RecordingStateMachine {
    state: RecordingState,
}

impl Default for RecordingStateMachine {
    fn default() -> Self {
        Self {
            state: RecordingState::Idle,
        }
    }
}

impl RecordingStateMachine {
    pub fn state(&self) -> RecordingState {
        self.state
    }

    pub fn start_recording(&mut self) -> AppResult<()> {
        match self.state {
            RecordingState::Idle => {
                self.state = RecordingState::Recording;
                Ok(())
            }
            _ => Err(AppError::State(format!(
                "cannot start recording while {:?}",
                self.state
            ))),
        }
    }

    pub fn start_transcribing(&mut self) -> AppResult<()> {
        match self.state {
            RecordingState::Recording => {
                self.state = RecordingState::Transcribing;
                Ok(())
            }
            _ => Err(AppError::State(format!(
                "cannot transcribe while {:?}",
                self.state
            ))),
        }
    }

    pub fn start_inserting(&mut self) -> AppResult<()> {
        match self.state {
            RecordingState::Transcribing => {
                self.state = RecordingState::Inserting;
                Ok(())
            }
            _ => Err(AppError::State(format!(
                "cannot insert while {:?}",
                self.state
            ))),
        }
    }

    pub fn finish(&mut self) {
        self.state = RecordingState::Idle;
    }

    pub fn cancel(&mut self) -> AppResult<()> {
        match self.state {
            RecordingState::Recording => {
                self.state = RecordingState::Cancelled;
                Ok(())
            }
            _ => Err(AppError::State(format!(
                "cannot cancel while {:?}",
                self.state
            ))),
        }
    }

    pub fn fail(&mut self) {
        self.state = RecordingState::Error;
    }
}

#[cfg(test)]
mod tests {
    use super::{RecordingState, RecordingStateMachine};

    #[test]
    fn follows_valid_recording_flow() {
        let mut machine = RecordingStateMachine::default();
        machine.start_recording().unwrap();
        assert_eq!(machine.state(), RecordingState::Recording);
        machine.start_transcribing().unwrap();
        assert_eq!(machine.state(), RecordingState::Transcribing);
        machine.start_inserting().unwrap();
        assert_eq!(machine.state(), RecordingState::Inserting);
        machine.finish();
        assert_eq!(machine.state(), RecordingState::Idle);
    }

    #[test]
    fn prevents_concurrent_recordings() {
        let mut machine = RecordingStateMachine::default();
        machine.start_recording().unwrap();
        assert!(machine.start_recording().is_err());
    }

    #[test]
    fn supports_cancellation_from_recording_only() {
        let mut machine = RecordingStateMachine::default();
        assert!(machine.cancel().is_err());
        machine.start_recording().unwrap();
        machine.cancel().unwrap();
        assert_eq!(machine.state(), RecordingState::Cancelled);
    }
}
