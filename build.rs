use std::{
    io::{self, Write},
    process,
};

fn main() {
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=static/style.css");

    #[cfg(unix)]
    let shell = "sh";
    #[cfg(windows)]
    let shell = "cmd";

    #[cfg(unix)]
    let flag = "-c";
    #[cfg(windows)]
    let flag = "/C";

    #[cfg(unix)]
    let command =
        "npx tailwindcss -c tailwind.config.js -i static/style.css -o static/compiled.css";
    #[cfg(windows)]
    let command =
        "npx tailwindcss -c tailwind.config.js -i static\\style.css -o static\\compiled.css";

    match process::Command::new(shell).arg(flag).arg(command).output() {
        Ok(output) => {
            if !output.status.success() {
                let _ = io::stdout().write_all(&output.stdout);
                let _ = io::stdout().write_all(&output.stderr);
                panic!("Tailwind error");
            } else {
                let _ = io::stdout().write_all(&output.stdout);
                println!("Tailwind compiled successfully!")
            }
        }
        Err(e) => panic!("Tailwind error: {:?}", e),
    };
}
