#[macro_use]
extern crate clap;

use tree_sitter_tags::TagsContext;
use std::fs::File;
use std::io::Write;
use std::error::Error;
use std::fs;
use std::path::Path;

pub mod config;
mod ruby;
mod javascript;
mod rust;

use config::Config;

pub struct App {}

impl App {
    pub fn run() -> Result<i32, Box<dyn Error>> {
        let config = Config::new();
        let exit_code = 0;

        let mut tags_file = File::create(config.tag_path).unwrap();
        let mut context = TagsContext::new();

        let ruby_config = ruby::config();
        let javascript_config = javascript::config();
        let rust_config = rust::config();

        config.files.iter().flat_map(|filename| {
            let contents = fs::read(&filename).unwrap();
            let path = Path::new(filename);

            match path.extension() {
                None => vec![],
                Some(os_str) => {
                    match os_str.to_str() {
                        Some("rb") => ruby::generate_tags(&mut context, &ruby_config, filename, &contents),
                        Some("js") => javascript::generate_tags(&mut context, &javascript_config, filename, &contents),
                        Some("rs") => rust::generate_tags(&mut context, &rust_config, filename, &contents),
                        _ => vec![]
                    }
                }
            }
        }).for_each(|line| tags_file.write_all(line.as_bytes()).unwrap());

        Ok(exit_code)
    }
}
