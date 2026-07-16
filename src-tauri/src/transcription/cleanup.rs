use crate::config::AppConfig;

pub fn cleanup_transcript(input: &str, config: &AppConfig) -> String {
    let mut text = normalize_spaces(input.trim());

    text = capitalize_first_letter(&text);

    if config.trailing_space && !text.is_empty() && !text.ends_with(char::is_whitespace) {
        text.push(' ');
    }

    text
}

pub fn normalize_spaces(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut previous_space = false;

    for ch in input.chars() {
        if ch.is_whitespace() {
            if !previous_space {
                output.push(' ');
                previous_space = true;
            }
        } else {
            output.push(ch);
            previous_space = false;
        }
    }

    output
}

pub fn apply_spoken_commands(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut output = String::new();
    let mut index = 0;

    while index < words.len() {
        let current = words[index].to_ascii_lowercase();
        let next = words
            .get(index + 1)
            .map(|word| word.to_ascii_lowercase())
            .unwrap_or_default();

        if current == "new" && next == "paragraph" {
            trim_trailing_space(&mut output);
            output.push_str("\n\n");
            index += 2;
        } else if current == "new" && next == "line" {
            trim_trailing_space(&mut output);
            output.push('\n');
            index += 2;
        } else {
            if !output.is_empty() && !output.ends_with([' ', '\n']) {
                output.push(' ');
            }
            output.push_str(words[index]);
            index += 1;
        }
    }

    output.trim().to_string()
}

fn trim_trailing_space(output: &mut String) {
    while output.ends_with(' ') {
        output.pop();
    }
}

fn capitalize_first_letter(input: &str) -> String {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    if first.is_ascii_lowercase() && looks_like_sentence(input) {
        let mut output = first.to_ascii_uppercase().to_string();
        output.extend(chars);
        output
    } else {
        input.to_string()
    }
}

fn looks_like_sentence(input: &str) -> bool {
    !input.contains("::")
        && !input.contains("=>")
        && !input.contains("->")
        && !input.contains('{')
        && !input.contains('}')
}

#[cfg(test)]
mod tests {
    use super::{apply_spoken_commands, cleanup_transcript, normalize_spaces};
    use crate::config::AppConfig;

    #[test]
    fn normalizes_repeated_whitespace() {
        assert_eq!(
            normalize_spaces(" one\t\t two\n three  "),
            " one two three "
        );
    }

    #[test]
    fn applies_basic_spoken_commands() {
        assert_eq!(
            apply_spoken_commands("hello new line world new paragraph done"),
            "hello\nworld\n\ndone"
        );
    }

    #[test]
    fn cleanup_preserves_code_like_text() {
        let config = AppConfig {
            trailing_space: false,
            ..AppConfig::default()
        };
        assert_eq!(cleanup_transcript("foo -> bar", &config), "foo -> bar");
    }

    #[test]
    fn cleanup_capitalizes_sentence_and_trailing_space() {
        let config = AppConfig::default();
        assert_eq!(cleanup_transcript(" hello world ", &config), "Hello world ");
    }

    #[test]
    fn cleanup_does_not_apply_voice_commands() {
        let config = AppConfig {
            trailing_space: false,
            ..AppConfig::default()
        };
        assert_eq!(
            cleanup_transcript("hello new line world", &config),
            "Hello new line world"
        );
    }
}
