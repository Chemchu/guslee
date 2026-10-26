use std::collections::HashMap;

const REQUIRED_VARS: [&str; 9] = [
    "LICHESS_API_TOKEN",
    "LICHESS_USERNAME",
    "SPOTIFY_CLIENT_ID",
    "SPOTIFY_CLIENT_SECRET",
    "SPOTIFY_USER_ID",
    "SPOTIFY_REFRESH_TOKEN",
    "STEAM_API_KEY",
    "STEAM_ID",
    "REPO_SOURCE",
];

// Loads the required variables from a `.env` file (searched from the working directory
// upwards) if present, falling back to the system environment for anything it does not
// define. This lets deployments without a `.env` file keep working.
pub fn read_env_file() -> HashMap<String, String> {
    match dotenvy::dotenv() {
        Ok(path) => log::info!("Loaded environment variables from {}", path.display()),
        Err(e) if e.not_found() => log::info!("No .env file found, using system environment"),
        Err(e) => panic!("Failed to parse .env file: {}", e),
    }

    collect_required_vars(|var| std::env::var(var).ok()).unwrap_or_else(|missing_vars| {
        panic!("Missing environment variables: {}", missing_vars.join(", "))
    })
}

// Looks up every required variable, returning all the missing ones at once on failure
fn collect_required_vars(
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<HashMap<String, String>, Vec<&'static str>> {
    let mut vars = HashMap::new();
    let mut missing_vars = Vec::new();

    for var in REQUIRED_VARS {
        match lookup(var) {
            Some(value) => {
                vars.insert(var.to_string(), value);
            }
            None => missing_vars.push(var),
        }
    }

    if missing_vars.is_empty() {
        Ok(vars)
    } else {
        Err(missing_vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_every_required_variable() {
        let vars = collect_required_vars(|var| Some(format!("{var}-value"))).unwrap();

        assert_eq!(vars.len(), REQUIRED_VARS.len());
        assert_eq!(vars["STEAM_ID"], "STEAM_ID-value");
    }

    #[test]
    fn reports_all_missing_variables_at_once() {
        let missing =
            collect_required_vars(|var| (!var.starts_with("SPOTIFY")).then(|| "value".to_string()))
                .unwrap_err();

        assert_eq!(
            missing,
            vec![
                "SPOTIFY_CLIENT_ID",
                "SPOTIFY_CLIENT_SECRET",
                "SPOTIFY_USER_ID",
                "SPOTIFY_REFRESH_TOKEN"
            ]
        );
    }
}
