use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, read_dir},
    path::Path,
    str::FromStr,
};

/// This enum represents the languages that the application supports.
#[derive(Debug, PartialEq)]
enum Language {
    English,
    Spanish,
    Portuguese,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Portuguese => write!(f, "portuguese"),
            Language::Spanish => write!(f, "spanish"),
            Language::English => write!(f, "english"),
        }
    }
}

struct Translator {
    translations: HashMap<String, String>,
}

impl Translator {
    /// This function creates a new Translator instance.
    pub fn new() -> Self {
        let mut translator = Translator {
            translations: HashMap::new(),
        };

        if let Ok(entries) = read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "yaml") {
                    if let Err(e) = translator.load_yaml_file(&path) {
                        eprintln!("Failed to load YAML file {:?}: {}", path, e);
                    }
                }
            }
        }

        Translator {
            translations: HashMap::new(),
        }
    }

    fn add_translation(&mut self, key: String, language: Language, value: String) {
        let key_and_language: String = format!("{}-{}", key, language);
        self.translations.insert(key_and_language, value);
    }

    /// This function returns the localized text for a given key and language.
    pub fn get_translation(&self, key: String, language: Language) -> Option<&String> {
        let key_and_language: String = format!("{}-{}", key, language);
        self.translations.get(&key_and_language)
    }

    fn load_yaml_file(&mut self, path: &std::path::PathBuf) {
        // Load keys into hashmap

        let languages = vec!["english", "spanish", "portuguese"];

        let file_name = path.to_str();
        if file_name.is_none() {
            panic!("??¿¿¿ gus¿¿¿");
        }

        let lang = String::from_str(file_name.unwrap()).unwrap();
        if lang.contains(languages.contains(languages)) {
            if let Ok(file) = fs::read_to_string(path) {}
        }

        self.translations
    }
}
