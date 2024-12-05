pub(crate) fn to_uppercase(value: &str) -> String {
    value.to_uppercase()
}

pub(crate) fn to_lowercase(value: &str) -> String {
    value.to_lowercase()
}

pub(crate) fn to_lower_snake_case(value: &str) -> String {
    to_snake_case(value).to_lowercase()
}

pub(crate) fn to_upper_snake_case(value: &str) -> String {
    to_snake_case(value).to_uppercase()
}

fn to_snake_case(value: &str) -> String {
    let mut acc = String::new();
    let mut parse_state = StringParseState::Start;
    for ch in value.chars() {
        let (_, word_start) = parse_state.update(ch);
        match word_start {
            WordStart::FirstWordAtStringStart
            | WordStart::FirstWordAfterNonLetter
            | WordStart::MiddleWordAfterNonLetter
            | WordStart::None => {
                acc.push(ch);
            }
            WordStart::MiddleWordAfterLetter => {
                acc.push('_');
                acc.push(ch);
            }
        }
    }
    acc
}

pub(crate) fn to_lower_camel_case(value: &str) -> String {
    to_camel_case(value, LetterCase::Lower)
}

pub(crate) fn to_upper_camel_case(value: &str) -> String {
    to_camel_case(value, LetterCase::Upper)
}

pub(crate) fn to_camel_case(value: &str, first_letter_case: LetterCase) -> String {
    let mut acc = String::new();
    let mut parse_state = StringParseState::Start;
    for ch in value.chars() {
        let (char_class, word_start) = parse_state.update(ch);
        if let CharClass::NonLetter = char_class {
            continue;
        }
        match word_start {
            WordStart::None => {
                acc.push(ch);
            }
            WordStart::FirstWordAtStringStart | WordStart::FirstWordAfterNonLetter => {
                first_letter_case.apply(&mut acc, ch);
            }
            WordStart::MiddleWordAfterLetter | WordStart::MiddleWordAfterNonLetter => {
                acc.extend(ch.to_uppercase());
            }
        }
    }
    acc
}

pub(crate) fn capitalize(value: &str) -> String {
    change_case_of_first_letter(value, LetterCase::Upper)
}

pub(crate) fn decapitalize(value: &str) -> String {
    change_case_of_first_letter(value, LetterCase::Lower)
}

pub(crate) fn insert_spaces_between_words(value: &str) -> String {
    let mut acc = String::new();
    let mut parse_state = StringParseState::Start;
    for ch in value.chars() {
        let (_, word_start) = parse_state.update(ch);
        match word_start {
            WordStart::MiddleWordAfterLetter => {
                acc.push(' ');
                acc.push(ch);
            }
            WordStart::None
            | WordStart::FirstWordAtStringStart
            | WordStart::FirstWordAfterNonLetter
            | WordStart::MiddleWordAfterNonLetter => {
                acc.push(ch);
            }
        }
    }
    acc
}

pub(crate) enum LetterCase {
    Upper,
    Lower,
}

impl LetterCase {
    fn apply(&self, acc: &mut String, ch: char) {
        match self {
            LetterCase::Upper => {
                acc.extend(ch.to_uppercase());
            }
            LetterCase::Lower => {
                acc.extend(ch.to_lowercase());
            }
        }
    }
}

pub fn change_case_of_first_letter(value: &str, first_letter_case: LetterCase) -> String {
    let mut acc = String::new();
    let mut parse_state = StringParseState::Start;
    for ch in value.chars() {
        let (_, word_start) = parse_state.update(ch);
        match word_start {
            WordStart::FirstWordAtStringStart | WordStart::FirstWordAfterNonLetter => {
                first_letter_case.apply(&mut acc, ch);
            }
            _ => {
                acc.push(ch);
            }
        }
    }
    acc
}

#[derive(Copy, Clone)]
enum StringParseState {
    Start,
    WordUpperCasedSoFar,
    WordLowerCasedSoFar,
    WordUpperCamelCasedSoFar,
    NonLetterBeforeFirstWord,
    NonLetterBetweenWords,
}

#[derive(Copy, Clone)]
enum CharClass {
    Uppercase,
    Lowercase,
    NonLetter,
}

enum WordStart {
    FirstWordAtStringStart,
    FirstWordAfterNonLetter,
    MiddleWordAfterLetter,
    MiddleWordAfterNonLetter,
    None,
}

impl StringParseState {
    fn update(&mut self, char: char) -> (CharClass, WordStart) {
        let char_class = if char.is_uppercase() {
            CharClass::Uppercase
        } else if char.is_lowercase() {
            CharClass::Lowercase
        } else {
            // Assume characters without a case are not letters...
            // This is not at all right in general, but it's good enough for an ident case conversion.
            CharClass::NonLetter
        };
        let (word_start, new_state) = match (*self, char_class) {
            (StringParseState::Start, CharClass::Uppercase) => {
                (WordStart::FirstWordAtStringStart, Self::WordUpperCasedSoFar)
            }
            (StringParseState::Start, CharClass::Lowercase) => {
                (WordStart::FirstWordAtStringStart, Self::WordLowerCasedSoFar)
            }
            (StringParseState::Start, CharClass::NonLetter) => {
                (WordStart::None, StringParseState::NonLetterBeforeFirstWord)
            }
            (StringParseState::WordUpperCasedSoFar, CharClass::Uppercase) => {
                (WordStart::None, Self::WordUpperCasedSoFar)
            }
            (StringParseState::WordUpperCasedSoFar, CharClass::Lowercase) => {
                (WordStart::None, Self::WordUpperCamelCasedSoFar)
            }
            (StringParseState::WordUpperCasedSoFar, CharClass::NonLetter) => {
                (WordStart::None, Self::NonLetterBetweenWords)
            }
            (StringParseState::WordLowerCasedSoFar, CharClass::Uppercase) => {
                (WordStart::MiddleWordAfterLetter, Self::WordUpperCasedSoFar)
            }
            (StringParseState::WordLowerCasedSoFar, CharClass::Lowercase) => {
                (WordStart::None, Self::WordLowerCasedSoFar)
            }
            (StringParseState::WordLowerCasedSoFar, CharClass::NonLetter) => {
                (WordStart::None, Self::NonLetterBetweenWords)
            }
            (StringParseState::WordUpperCamelCasedSoFar, CharClass::Uppercase) => {
                (WordStart::MiddleWordAfterLetter, Self::WordUpperCasedSoFar)
            }
            (StringParseState::WordUpperCamelCasedSoFar, CharClass::Lowercase) => {
                (WordStart::None, Self::WordUpperCamelCasedSoFar)
            }
            (StringParseState::WordUpperCamelCasedSoFar, CharClass::NonLetter) => {
                (WordStart::None, Self::NonLetterBetweenWords)
            }
            (StringParseState::NonLetterBeforeFirstWord, CharClass::Uppercase) => (
                WordStart::FirstWordAfterNonLetter,
                Self::WordUpperCasedSoFar,
            ),
            (StringParseState::NonLetterBeforeFirstWord, CharClass::Lowercase) => (
                WordStart::FirstWordAfterNonLetter,
                Self::WordLowerCasedSoFar,
            ),
            (StringParseState::NonLetterBeforeFirstWord, CharClass::NonLetter) => {
                (WordStart::None, StringParseState::NonLetterBeforeFirstWord)
            }
            (StringParseState::NonLetterBetweenWords, CharClass::Uppercase) => (
                WordStart::MiddleWordAfterNonLetter,
                Self::WordUpperCasedSoFar,
            ),
            (StringParseState::NonLetterBetweenWords, CharClass::Lowercase) => (
                WordStart::MiddleWordAfterNonLetter,
                Self::WordLowerCasedSoFar,
            ),
            (StringParseState::NonLetterBetweenWords, CharClass::NonLetter) => {
                (WordStart::None, Self::NonLetterBetweenWords)
            }
        };
        *self = new_state;
        (char_class, word_start)
    }
}
