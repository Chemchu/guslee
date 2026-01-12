use std::{collections::HashMap, fs};

pub fn read_env_file() -> HashMap<String, String> {
    let mut env_vars: HashMap<String, String> = HashMap::new();
    let content = fs::read_to_string(format!("{}/.env", env!("CARGO_MANIFEST_DIR"))).ok();

    match content {
        Some(c) => {
            for line in c.lines() {
                let cleaned_line = line.replace("\"", "");
                let mut split_line = cleaned_line.split("=");

                let key = split_line.next();
                let value = split_line.next();

                if key.is_none() || value.is_none() {
                    panic!("ENV variable missing. Please follow 'Key=Value' pattern")
                }

                env_vars.insert(key.unwrap().to_string(), value.unwrap().to_string());
            }
            env_vars
        }
        None => load_vars_from_environment(),
    }
}

// This function tries to load the environment variables already present in the system.
// It is useful if the environment in which the app is deployed does not have a .env file defined
fn load_vars_from_environment() -> HashMap<String, String> {
    let mut env_vars: HashMap<String, String> = HashMap::new();

    let lichess_api_token = std::env::var("LICHESS_API_TOKEN").unwrap();
    let lichess_username = std::env::var("LICHESS_USERNAME").unwrap();

    env_vars.insert("LICHESS_API_TOKEN".to_string(), lichess_api_token);
    env_vars.insert("LICHESS_USERNAME".to_string(), lichess_username);

    env_vars
}
