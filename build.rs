use std::process::Command;

fn main() {
    let tailwind_cmd = "npx tailwindcss -i templates/input.css -o dist/app.css";

    Command::new("sh")
        .arg("-c")
        .arg(tailwind_cmd)
        .status()
        .expect("error running tailwind");

    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=input.css");
}
