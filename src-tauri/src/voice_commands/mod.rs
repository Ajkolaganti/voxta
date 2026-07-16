pub mod executor;
pub mod history;
pub mod parser;

pub use executor::{process_transcript, CommandProcessingResult};
pub use parser::{CommandPhraseStyle, RecognizedCommand, VoiceCommand};
