#[macro_use]
extern crate clap;

use std::env;
use std::path::PathBuf;
use clap::Shell;

include!("src/cli.rs");

fn main() {
    let mut app = build_cli();
    let mut outdir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR variable was not set"));

    // This will output the completions in the target/release or target/debug directories.
    outdir.pop();
    outdir.pop();
    outdir.pop();

    app.gen_completions("ttags", Shell::Bash, &outdir);
    app.gen_completions("ttags", Shell::Fish, &outdir);
    app.gen_completions("ttags", Shell::Zsh, outdir);
}
