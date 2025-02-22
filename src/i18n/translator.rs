use std::{
    collections::HashMap,
    fs::{self, read_dir},
    str::FromStr,
};

use super::{Language, Message};

pub struct Translator {
    translations: HashMap<String, String>,
}

impl Translator {
    /// This function creates a new Translator instance.
    pub fn new() -> Self {
        let mut translator = Translator {
            translations: HashMap::new(),
        };

        if let Ok(entries) = read_dir("./src/i18n") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "yaml") {
                    translator.load_yaml_file(&path);
                }
            }
        }

        translator
    }

    fn add_translation(&mut self, key: &str, language: &Language, value: &str) {
        let key_and_language: String = format!("{}-{}", key, language);
        self.translations
            .insert(key_and_language, value.to_string());
    }

    /// This function returns the localized text for a given key in Spanish.
    pub fn get_translation(&self, key: &str) -> String {
        let message_key = Message::from_str(key).expect("Failed to convert the &str into Message");
        self.get_translation_by_language(&message_key, &Language::Spanish)
            .expect("Could not translate the message")
            .to_string()
    }

    /// This function returns the localized text for a given key and language.
    fn get_translation_by_language(&self, key: &Message, language: &Language) -> Option<&String> {
        let key_and_language: String = format!("{}-{}", key, language);
        self.translations.get(&key_and_language)
    }

    /// This function tries to load the contents of a YAML file and add its contento into the
    /// translations map
    fn load_yaml_file(&mut self, path: &std::path::PathBuf) {
        let languages = Language::all();
        let file_name = String::from_str(path.to_str().expect("Failed to convert path to string"))
            .expect("Failed to convert slice into String");

        for language in languages {
            if !file_name.contains(&language.to_string()) {
                /* // This panic indicates that there's a new language YAML file but the Language
                // enum was not updated
                panic!("Language file defined when no such language is defined") */
                continue;
            }

            if let Ok(file) = fs::read_to_string(path) {
                let locale_map: HashMap<String, String> = serde_yaml::from_str(file.as_str())
                    .expect("Failed to transform YAML to Struct");

                locale_map
                    .iter()
                    .for_each(|(key, value)| self.add_translation(key, &language, value))
            }
        }
    }
}
