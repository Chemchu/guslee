use actix_web::http::header::{LanguageTag, Preference};
use strum_macros::{Display, EnumString};

pub mod translator;

#[derive(Debug, PartialEq, Display, EnumString)]
pub enum Message {
    Me,
    News,
    Contact,
    Experience,
    SelfDescription,
    SoftwareEngineer,
    SoftwareEngineerTitle,
    Articles,
    ArticlesDescription,
}

/// This enum represents the languages that the application supports.
#[derive(Debug, PartialEq, Display, Clone)]
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

pub fn to_language(langs: &Vec<Preference<LanguageTag>>) -> &str {
    let mut languages: Vec<&str> = vec![];
    for l in langs {
        if l.item().unwrap().primary_language() == "en" {
            languages.push("en");
        }

        if l.item().unwrap().primary_language() == "es" {
            languages.push("es");
        }

        if l.item().unwrap().primary_language() == "pt" {
            languages.push("pt");
        }
    }

    if languages.is_empty() {
        languages.push("en");
    }

    languages.first().unwrap()
}
