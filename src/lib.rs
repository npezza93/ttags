#[macro_use]
extern crate clap;

use tree_sitter_tags::TagsContext;
use std::io::Write;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::str;
use sugar_path::SugarPath;

pub mod config;
pub mod tag;
pub mod ruby;
pub mod javascript;
pub mod rust;

use config::Config;

pub struct App {}

impl App {
    pub fn run() -> Result<i32, Box<dyn Error>> {
        let config = Config::new();
        let exit_code = 0;

        let mut context = TagsContext::new();

        let ruby_config = ruby::config();
        let javascript_config = javascript::config();
        let rust_config = rust::config();

        let tags = config.files.iter().flat_map(|filename| {
            match fs::read(&filename) {
                Ok(contents) => {
                    let path = Path::new(filename);

                    match path.extension() {
                        Some(os_str) => {
                            match os_str.to_str() {
                                Some("rb") => ruby::generate_tags(&mut context, &ruby_config, filename, &contents),
                                Some("js") => javascript::generate_tags(&mut context, &javascript_config, filename, &contents),
                                Some("rs") => rust::generate_tags(&mut context, &rust_config, filename, &contents),
                                _ => vec![]
                            }
                        },
                        None => vec![]
                    }
                },
                Err(_) => {
                    if config.appending() {
                        vec![]
                    } else {
                        println!("{} not found", filename);
                        exit(1)
                    }
                }
            }
        });

        let mut output = config.output();
        if config.appending() {
            let filtered = config.current_tag_contents().split('\n').filter(|line| {
                let path = Path::new(&line.split('\t').collect::<Vec<&str>>()[1]).resolve().into_os_string().into_string().unwrap();

                !config.files.contains(&path)
            }).collect::<Vec<&str>>().join("\n");

            config.clear_tag_file();
            output.write_all(filtered.as_bytes()).unwrap();
            output.write_all(b"\n").unwrap();
        }

        tags.for_each(|line| output.write_all(&line.as_bytes(&config)).unwrap());

        Ok(exit_code)
    }
}
