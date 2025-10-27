pub(crate) trait StringExtensions {
    fn lower_indefinite_articled(&self) -> String;
    fn upper_indefinite_articled(&self) -> String;
}

impl<T: AsRef<str>> StringExtensions for T {
    fn lower_indefinite_articled(&self) -> String {
        let string = self.as_ref();
        let first_char = string.chars().next().unwrap();
        match first_char {
            'a' | 'e' | 'i' | 'o' | 'u' | 'A' | 'E' | 'I' | 'O' | 'U' => format!("an {}", string),
            _ => format!("a {}", string),
        }
    }

    fn upper_indefinite_articled(&self) -> String {
        let string = self.as_ref();
        let first_char = string.chars().next().unwrap();
        match first_char {
            'a' | 'e' | 'i' | 'o' | 'u' | 'A' | 'E' | 'I' | 'O' | 'U' => format!("An {}", string),
            _ => format!("A {}", string),
        }
    }
}
