#[macro_use]
extern crate clap;

use tree_sitter_tags::TagsContext;
use std::io::Write;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::exit;
use rayon::prelude::*;

pub mod config;
pub mod tag;
pub mod ruby;
pub mod javascript;
pub mod rust;

use config::Config;
use crate::tag::Tag;

pub struct App {}

impl App {
    pub fn run() -> Result<i32, Box<dyn Error>> {
        let mut context       = TagsContext::new();
        let config            = Config::new();
        let ruby_config       = ruby::config();
        let javascript_config = javascript::config();
        let rust_config       = rust::config();

        let mut tags: Vec<Tag> = config.files.iter().flat_map(|filename| {
            match fs::read(filename) {
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
        }).collect();

        let mut output = config.output();
        if config.appending() {
            tags.extend(config.current_tag_contents().iter().filter(|tag| {
                !config.files.contains(&tag.filename)
            }).cloned().collect::<Vec<Tag>>());
        }

        config.clear_tag_file();
        tags.par_sort_by_key(|tag| tag.name.clone());
        tags.iter().for_each(|tag| {
            output.write_all(&tag.as_bytes(&config)).unwrap()
        });

        Ok(0)
    }
}
