use clap_complete::{Shell, generate_to};
use std::env;
use std::path::PathBuf;

include!("src/cli.rs");

fn main() {
    let mut app = build_cli();
    let mut outdir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR variable was not set"));

    // This will output the completions in the target/release or target/debug directories.
    outdir.pop();
    outdir.pop();
    outdir.pop();

    generate_to(Shell::Bash, &mut app, "ttags", &outdir).unwrap();
    generate_to(Shell::Fish, &mut app, "ttags", &outdir).unwrap();
    generate_to(Shell::Zsh, &mut app, "ttags", outdir).unwrap();
}
