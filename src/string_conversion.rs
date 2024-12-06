// These are tested in tests/string_conversion.rs

use std::char;

pub(crate) fn to_uppercase(value: &str) -> String {
    value.to_uppercase()
}

pub(crate) fn to_lowercase(value: &str) -> String {
    value.to_lowercase()
}

pub(crate) fn to_lower_snake_case(value: &str) -> String {
    WordSeparator { separator: '_' }
        .transform(value)
        .to_lowercase()
}

pub(crate) fn to_upper_snake_case(value: &str) -> String {
    WordSeparator { separator: '_' }
        .transform(value)
        .to_uppercase()
}

pub(crate) fn to_lower_kebab_case(value: &str) -> String {
    WordSeparator { separator: '-' }
        .transform(value)
        .to_lowercase()
}

pub(crate) fn to_lower_camel_case(value: &str) -> String {
    CamelCaser {
        first_letter_case: ChangeLetterCase::Lower,
    }
    .transform(value)
}

pub(crate) fn to_upper_camel_case(value: &str) -> String {
    CamelCaser {
        first_letter_case: ChangeLetterCase::Upper,
    }
    .transform(value)
}

pub(crate) fn capitalize(value: &str) -> String {
    FirstLetterChanger {
        first_letter_case: ChangeLetterCase::Upper,
    }
    .transform(value)
}

pub(crate) fn decapitalize(value: &str) -> String {
    FirstLetterChanger {
        first_letter_case: ChangeLetterCase::Lower,
    }
    .transform(value)
}

pub(crate) fn title_case(value: &str) -> String {
    TitleCaser.transform(value)
}

pub(crate) fn insert_spaces_between_words(value: &str) -> String {
    WordSeparator { separator: ' ' }.transform(value)
}

struct WordSeparator {
    separator: char,
}

impl CharTransform for WordSeparator {
    fn handle(&mut self, output: &mut String, ch: char, char_details: CharDetails) {
        match char_details {
            CharDetails::NonWordCharacter => {}
            CharDetails::MiddleOfWord => {
                output.push(ch);
            }
            CharDetails::StartOfFirstWord => {
                output.push(ch);
            }
            CharDetails::StartOfNonFirstWord => {
                output.push(self.separator);
                output.push(ch);
            }
        }
    }
}

struct TitleCaser;

impl CharTransform for TitleCaser {
    fn handle(&mut self, output: &mut String, ch: char, char_details: CharDetails) {
        match char_details {
            CharDetails::NonWordCharacter => {}
            CharDetails::MiddleOfWord => {
                output.push_cased_char(ch, ChangeLetterCase::Lower);
            }
            CharDetails::StartOfFirstWord => {
                output.push_cased_char(ch, ChangeLetterCase::Upper);
            }
            CharDetails::StartOfNonFirstWord => {
                output.push(' ');
                output.push_cased_char(ch, ChangeLetterCase::Upper);
            }
        }
    }
}

struct CamelCaser {
    first_letter_case: ChangeLetterCase,
}

impl CharTransform for CamelCaser {
    fn handle(&mut self, output: &mut String, ch: char, char_details: CharDetails) {
        match char_details {
            CharDetails::NonWordCharacter => {}
            CharDetails::MiddleOfWord => {
                output.push_cased_char(ch, ChangeLetterCase::Lower);
            }
            CharDetails::StartOfFirstWord => {
                output.push_cased_char(ch, self.first_letter_case);
            }
            CharDetails::StartOfNonFirstWord => {
                output.push_cased_char(ch, ChangeLetterCase::Upper);
            }
        }
    }
}

struct FirstLetterChanger {
    first_letter_case: ChangeLetterCase,
}

impl CharTransform for FirstLetterChanger {
    fn handle(&mut self, output: &mut String, ch: char, char_details: CharDetails) {
        match char_details {
            CharDetails::MiddleOfWord
            | CharDetails::StartOfNonFirstWord
            | CharDetails::NonWordCharacter => {
                output.push(ch);
            }
            CharDetails::StartOfFirstWord => {
                output.push_cased_char(ch, self.first_letter_case);
            }
        }
    }
}

trait CharTransform: Sized {
    fn transform(self, str: &str) -> String {
        StringTransformer::new(str).run(self)
    }

    fn handle(&mut self, output: &mut String, ch: char, char_details: CharDetails);
}

enum CharDetails {
    NonWordCharacter,
    MiddleOfWord,
    StartOfFirstWord,
    StartOfNonFirstWord,
}

struct StringTransformer<'a> {
    chars_iter: core::iter::Peekable<core::str::Chars<'a>>,
    word_state: WordState,
    first_word_started: bool,
    output: String,
}

impl<'a> StringTransformer<'a> {
    fn new(value: &'a str) -> Self {
        Self {
            chars_iter: value.chars().peekable(),
            word_state: WordState::NotInWord,
            first_word_started: false,
            output: String::with_capacity(value.len()),
        }
    }

    fn run(mut self, mut char_mapper: impl CharTransform) -> String {
        while let Some(char) = self.chars_iter.next() {
            let (char_details, next_word_state) = if char.is_alphanumeric() {
                let mut letter_case = if char.is_uppercase() {
                    LetterCase::Uppercased
                } else if char.is_lowercase() {
                    LetterCase::Lowercased
                } else {
                    LetterCase::Uncased
                };

                let mut is_word_start = false;
                match self.word_state {
                    WordState::InWord { last_case } => {
                        match (last_case, letter_case) {
                            (LetterCase::Uppercased, LetterCase::Uppercased) => {
                                if let Some(char) = self.chars_iter.peek() {
                                    if char.is_lowercase() {
                                        // Handle the C right in PostgreSQLConnection => postgre_sql_connection
                                        is_word_start = true
                                    }
                                }
                            }
                            (LetterCase::Lowercased, LetterCase::Uppercased) => {
                                is_word_start = true;
                            }
                            _ => {}
                        }
                        if let LetterCase::Uncased = letter_case {
                            letter_case = last_case;
                        }
                    }
                    WordState::NotInWord => is_word_start = true,
                };

                let char_details = if is_word_start {
                    if self.first_word_started {
                        CharDetails::StartOfNonFirstWord
                    } else {
                        self.first_word_started = true;
                        CharDetails::StartOfFirstWord
                    }
                } else {
                    CharDetails::MiddleOfWord
                };

                let next_word_state = WordState::InWord {
                    last_case: letter_case,
                };

                (char_details, next_word_state)
            } else {
                (CharDetails::NonWordCharacter, WordState::NotInWord)
            };
            self.word_state = next_word_state;
            char_mapper.handle(&mut self.output, char, char_details);
        }
        self.output
    }
}

#[derive(Clone, Copy)]
enum ChangeLetterCase {
    Upper,
    Lower,
}

trait StringExt {
    fn push_cased_char(&mut self, ch: char, case: ChangeLetterCase);
}

impl StringExt for String {
    fn push_cased_char(&mut self, character: char, case: ChangeLetterCase) {
        match case {
            ChangeLetterCase::Upper => {
                self.extend(character.to_uppercase());
            }
            ChangeLetterCase::Lower => {
                self.extend(character.to_lowercase());
            }
        }
    }
}

#[derive(Copy, Clone)]
enum WordState {
    /// Alphanumeric characters
    InWord {
        last_case: LetterCase,
    },
    NotInWord,
}

#[derive(Copy, Clone, Debug)]
enum LetterCase {
    /// Uppercase characters
    Uppercased,
    /// Lowercase characters
    Lowercased,
    /// Alphanumeric characters that are not uppercase or lowercase
    Uncased,
}
