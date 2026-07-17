use crate::config::{VoiceCommandConfig, VoiceCommandMode};

use super::{
    history::CommandHistory,
    parser::{recognize_at, tokenize, CommandPhraseStyle, RecognizedCommand, Token, VoiceCommand},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandProcessingResult {
    pub text: String,
    pub cancelled: bool,
    pub commands: Vec<RecognizedCommand>,
}

pub fn process_transcript(input: &str, config: &VoiceCommandConfig) -> CommandProcessingResult {
    if !config.enabled {
        return CommandProcessingResult {
            text: input.trim().to_string(),
            cancelled: false,
            commands: Vec::new(),
        };
    }

    let style = match config.mode {
        VoiceCommandMode::Natural => CommandPhraseStyle::Natural,
        VoiceCommandMode::Prefix => CommandPhraseStyle::Prefix,
    };
    process_tokens(tokenize(input), style, &config.prefix)
}

fn process_tokens(
    tokens: Vec<Token>,
    style: CommandPhraseStyle,
    prefix: &str,
) -> CommandProcessingResult {
    let mut output = String::new();
    let mut history = CommandHistory::default();
    let mut commands = Vec::new();
    let mut index = 0;
    let mut cancelled = false;

    while index < tokens.len() {
        if let Some(recognized) = recognize_at(&tokens, index, style, prefix) {
            let next_index = recognized.end_token;
            apply_command(
                recognized.command,
                &mut output,
                &mut history,
                &mut cancelled,
            );
            commands.push(recognized);
            index = next_index;
            if cancelled {
                output.clear();
                break;
            }
            continue;
        }

        append_word(&mut output, &tokens[index].original);
        index += 1;
    }

    CommandProcessingResult {
        text: output.trim().to_string(),
        cancelled,
        commands,
    }
}

fn apply_command(
    command: VoiceCommand,
    output: &mut String,
    history: &mut CommandHistory,
    cancelled: &mut bool,
) {
    match command {
        VoiceCommand::NewLine => {
            history.push(output);
            trim_trailing_space(output);
            output.push('\n');
        }
        VoiceCommand::NewParagraph => {
            history.push(output);
            trim_trailing_space(output);
            output.push_str("\n\n");
        }
        VoiceCommand::DeleteLastWord => {
            history.push(output);
            delete_last_word(output);
        }
        VoiceCommand::DeleteLastSentence => {
            history.push(output);
            delete_last_sentence(output);
        }
        VoiceCommand::Undo => {
            if let Some(previous) = history.undo() {
                *output = previous;
            }
        }
        VoiceCommand::ClearDictation => {
            history.push(output);
            output.clear();
        }
        VoiceCommand::CancelDictation => {
            *cancelled = true;
        }
    }
}

fn append_word(output: &mut String, word: &str) {
    if word.is_empty() {
        return;
    }
    if !output.is_empty() && !output.ends_with([' ', '\n']) {
        output.push(' ');
    }
    output.push_str(word);
}

fn delete_last_word(output: &mut String) {
    trim_trailing_space(output);
    let Some(last_boundary) = output
        .char_indices()
        .rev()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
    else {
        output.clear();
        return;
    };
    output.truncate(last_boundary);
    trim_trailing_space(output);
}

fn delete_last_sentence(output: &mut String) {
    trim_trailing_space(output);
    if output.is_empty() {
        return;
    }

    let trimmed = output.trim_end();
    let boundaries = trimmed
        .char_indices()
        .filter_map(|(index, ch)| matches!(ch, '.' | '!' | '?').then_some(index + ch.len_utf8()))
        .collect::<Vec<_>>();

    if boundaries.is_empty() {
        output.clear();
        return;
    }

    let ends_with_boundary = trimmed.ends_with(['.', '!', '?']);
    let keep_boundary = if ends_with_boundary {
        boundaries.get(boundaries.len().saturating_sub(2)).copied()
    } else {
        boundaries.last().copied()
    };

    if let Some(index) = keep_boundary {
        output.truncate(index);
        trim_trailing_space(output);
    } else {
        output.clear();
    }
}

fn trim_trailing_space(output: &mut String) {
    while output.ends_with(' ') {
        output.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::process_transcript;
    use crate::config::{VoiceCommandConfig, VoiceCommandMode};

    fn prefix_config() -> VoiceCommandConfig {
        VoiceCommandConfig {
            enabled: true,
            mode: VoiceCommandMode::Prefix,
            prefix: "Voxta".to_string(),
        }
    }

    fn natural_config() -> VoiceCommandConfig {
        VoiceCommandConfig {
            enabled: true,
            mode: VoiceCommandMode::Natural,
            prefix: "Voxta".to_string(),
        }
    }

    #[test]
    fn applies_new_line_and_paragraph() {
        let result = process_transcript(
            "Send me the report Voxta new line I will review it Voxta new paragraph tomorrow",
            &prefix_config(),
        );
        assert_eq!(
            result.text,
            "Send me the report\nI will review it\n\ntomorrow"
        );
    }

    #[test]
    fn deletes_last_word() {
        let result = process_transcript(
            "The meeting is Friday Monday Voxta delete last word",
            &prefix_config(),
        );
        assert_eq!(result.text, "The meeting is Friday");
    }

    #[test]
    fn deletes_last_sentence() {
        let result = process_transcript(
            "First sentence. Second sentence Voxta delete last sentence",
            &prefix_config(),
        );
        assert_eq!(result.text, "First sentence.");
    }

    #[test]
    fn undo_restores_previous_command_mutation() {
        let result = process_transcript(
            "The meeting is Friday Monday Voxta delete last word Voxta undo",
            &prefix_config(),
        );
        assert_eq!(result.text, "The meeting is Friday Monday");
    }

    #[test]
    fn clear_dictation_continues_listening() {
        let result = process_transcript(
            "Discard this Voxta clear dictation keep this",
            &prefix_config(),
        );
        assert_eq!(result.text, "keep this");
    }

    #[test]
    fn cancel_dictation_marks_cancelled() {
        let result = process_transcript("hello Voxta cancel dictation world", &prefix_config());
        assert!(result.cancelled);
        assert_eq!(result.text, "");
    }

    #[test]
    fn supports_natural_mode() {
        let result = process_transcript("hello new line world", &natural_config());
        assert_eq!(result.text, "hello\nworld");
    }

    #[test]
    fn avoids_accidental_command_in_prefix_mode() {
        let result = process_transcript("please delete last word", &prefix_config());
        assert_eq!(result.text, "please delete last word");
        assert!(result.commands.is_empty());
    }

    #[test]
    fn reports_stable_command_ids_for_overlapping_text() {
        let first = process_transcript("hello Voxta new line world", &prefix_config());
        let second = process_transcript("hello Voxta new line world again", &prefix_config());
        assert_eq!(first.commands[0].id, second.commands[0].id);
    }
}
