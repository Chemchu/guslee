use std::{
    io::{self, Write},
    process,
};

fn main() {
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=static/style.css");
    println!("cargo:rerun-if-changed=templates");

    #[cfg(unix)]
    let shell = "sh";
    #[cfg(windows)]
    let shell = "cmd";

    #[cfg(unix)]
    let flag = "-c";
    #[cfg(windows)]
    let flag = "/C";

    #[cfg(unix)]
    let command = "tailwindcss -i static/style.css -o static/compiled.css --minify; prettier --write './**/*.html'";

    #[cfg(windows)]
    let command = "tailwindcss -i static\\style.css -o static\\compiled.css --minify; prettier --write './**/*.html'";

    match process::Command::new(shell).arg(flag).arg(command).output() {
        Ok(output) => {
            if !output.status.success() {
                let _ = io::stdout().write_all(&output.stdout);
                let _ = io::stdout().write_all(&output.stderr);
                println!("Could not complete build step: TailwindCSS binary not found");
            } else {
                let _ = io::stdout().write_all(&output.stdout);
                println!("Build step completed successfully!")
            }
        }
        Err(e) => println!("Could not complete build step because of: {:?}", e),
    };
}
