use strum_macros::Display;

pub mod translator;

#[derive(Debug, PartialEq, Display)]
pub enum Message {
    Test1,
    Test2,
    Test3,
}

/// This enum represents the languages that the application supports.
#[derive(Debug, PartialEq, Display)]
pub enum Language {
    English,
    Spanish,
    Portuguese,
}

impl Language {
    /// This function returns a list of all supported languages.
    pub fn all() -> Vec<Language> {
        vec![Language::English, Language::Spanish, Language::Portuguese]
    }
}
