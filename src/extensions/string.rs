pub(crate) trait StringExtensions {
    /// NOTE: This isn't foolproof - it is actually pretty rubbish with
    /// type names, where we say things like "an f32" and "a u8" due to
    /// sounding out the first letter of the acronym.
    /// We may wish to just replace this with hardcoded articled names.
    fn indefinite_articled(&self, upper_case_article: bool) -> String;
}

impl<T: AsRef<str>> StringExtensions for T {
    fn indefinite_articled(&self, upper_case_article: bool) -> String {
        let string = self.as_ref();
        let first_char = string.chars().next().unwrap();
        match first_char {
            'a' | 'e' | 'i' | 'o' | 'u' | 'A' | 'E' | 'I' | 'O' | 'U' => {
                if upper_case_article {
                    format!("An {}", string)
                } else {
                    format!("an {}", string)
                }
            }
            _ => {
                if upper_case_article {
                    format!("A {}", string)
                } else {
                    format!("a {}", string)
                }
            }
        }
    }
}
