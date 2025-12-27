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
        None => env_vars,
    }
}
