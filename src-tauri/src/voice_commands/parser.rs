use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VoiceCommand {
    NewLine,
    NewParagraph,
    DeleteLastWord,
    DeleteLastSentence,
    Undo,
    ClearDictation,
    CancelDictation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPhraseStyle {
    Natural,
    Prefix,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecognizedCommand {
    pub id: String,
    pub command: VoiceCommand,
    pub start_token: usize,
    pub end_token: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub original: String,
    pub normalized: String,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    input
        .split_whitespace()
        .filter_map(|word| {
            let normalized = normalize_word(word);
            if normalized.is_empty() {
                None
            } else {
                Some(Token {
                    original: word.to_string(),
                    normalized,
                })
            }
        })
        .collect()
}

pub fn recognize_at(
    tokens: &[Token],
    index: usize,
    style: CommandPhraseStyle,
    prefix: &str,
) -> Option<RecognizedCommand> {
    let prefix_words = tokenize(prefix)
        .into_iter()
        .map(|token| token.normalized)
        .collect::<Vec<_>>();

    let mut offset = index;
    if style == CommandPhraseStyle::Prefix {
        if prefix_words.is_empty() || !matches_normalized_words(tokens, offset, &prefix_words) {
            return None;
        }
        offset += prefix_words.len();
    }

    for (command, words) in command_phrases() {
        if matches_words(tokens, offset, words) {
            let end = offset + words.len();
            return Some(RecognizedCommand {
                id: format!("{index}:{end}:{command:?}"),
                command: *command,
                start_token: index,
                end_token: end,
            });
        }
    }

    None
}

fn command_phrases() -> &'static [(VoiceCommand, &'static [&'static str])] {
    &[
        (
            VoiceCommand::DeleteLastSentence,
            &["delete", "last", "sentence"],
        ),
        (VoiceCommand::DeleteLastWord, &["delete", "last", "word"]),
        (VoiceCommand::NewParagraph, &["new", "paragraph"]),
        (VoiceCommand::CancelDictation, &["cancel", "dictation"]),
        (VoiceCommand::ClearDictation, &["clear", "dictation"]),
        (VoiceCommand::NewLine, &["new", "line"]),
        (VoiceCommand::Undo, &["undo"]),
    ]
}

fn matches_words(tokens: &[Token], index: usize, words: &[&str]) -> bool {
    if index + words.len() > tokens.len() {
        return false;
    }

    words
        .iter()
        .enumerate()
        .all(|(offset, word)| tokens[index + offset].normalized == *word)
}

fn matches_normalized_words(tokens: &[Token], index: usize, words: &[String]) -> bool {
    if index + words.len() > tokens.len() {
        return false;
    }

    words
        .iter()
        .enumerate()
        .all(|(offset, word)| tokens[index + offset].normalized == *word)
}

fn normalize_word(input: &str) -> String {
    trim_surrounding_punctuation(input).to_ascii_lowercase()
}

fn trim_surrounding_punctuation(input: &str) -> &str {
    input.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '\'')
}

#[cfg(test)]
mod tests {
    use super::{recognize_at, tokenize, CommandPhraseStyle, VoiceCommand};

    #[test]
    fn recognizes_prefixed_command_with_punctuation() {
        let tokens = tokenize("hello Voxta, delete last word.");
        let command = recognize_at(&tokens, 1, CommandPhraseStyle::Prefix, "Voxta").unwrap();
        assert_eq!(command.command, VoiceCommand::DeleteLastWord);
        assert_eq!(command.start_token, 1);
    }

    #[test]
    fn rejects_unprefixed_command_in_prefix_mode() {
        let tokens = tokenize("delete last word");
        assert!(recognize_at(&tokens, 0, CommandPhraseStyle::Prefix, "Voxta").is_none());
    }

    #[test]
    fn recognizes_natural_command() {
        let tokens = tokenize("hello new line world");
        let command = recognize_at(&tokens, 1, CommandPhraseStyle::Natural, "Voxta").unwrap();
        assert_eq!(command.command, VoiceCommand::NewLine);
    }

    #[test]
    fn avoids_longer_word_false_positive() {
        let tokens = tokenize("newsletter update");
        assert!(recognize_at(&tokens, 0, CommandPhraseStyle::Natural, "Voxta").is_none());
    }
}
