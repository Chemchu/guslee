use std::collections::HashMap;
use std::process::Command;

pub const GIT_LOG_ARGS: [&str; 6] = [
    "log",
    "--format=%x00%cs",
    "--name-only",
    "--relative",
    "--",
    ".",
];

pub const DATES_FILE_VAR: &str = "GARDEN_DATES_FILE";

pub fn last_updated_dates(documents_path: &str) -> HashMap<String, String> {
    let from_git = Command::new("git")
        .arg("-C")
        .arg(documents_path)
        .args(GIT_LOG_ARGS)
        .output()
        .ok()
        .filter(|output| output.status.success() && !output.stdout.is_empty())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned());

    let log = from_git.or_else(|| {
        let path = std::env::var(DATES_FILE_VAR).ok()?;
        std::fs::read_to_string(path).ok()
    });

    log.map(|log| parse_git_log(&log)).unwrap_or_default()
}

fn parse_git_log(log: &str) -> HashMap<String, String> {
    let mut dates = HashMap::new();

    for commit in log.split('\0').filter(|c| !c.trim().is_empty()) {
        let mut lines = commit.lines();
        let Some(date) = lines.next().map(str::trim) else {
            continue;
        };
        for file in lines.map(str::trim).filter(|f| !f.is_empty()) {
            dates
                .entry(file.to_string())
                .or_insert_with(|| date.to_string());
        }
    }

    dates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_newest_date_of_each_file() {
        let log =
            "\x002025-10-02\n\nwelcome.md\njournal/a.md\n\x002025-09-01\n\nwelcome.md\nhello.md\n";

        let dates = parse_git_log(log);

        assert_eq!(dates["welcome.md"], "2025-10-02");
        assert_eq!(dates["journal/a.md"], "2025-10-02");
        assert_eq!(dates["hello.md"], "2025-09-01");
        assert_eq!(dates.len(), 3);
    }

    #[test]
    fn no_history_means_no_dates() {
        assert!(parse_git_log("").is_empty());
        let dir = tempfile::tempdir().unwrap();
        assert!(last_updated_dates(dir.path().to_str().unwrap()).is_empty());
    }
}
