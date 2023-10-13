use npezza93_tree_sitter_tags::{TagsContext, TagsConfiguration};
use std::io::Write;
use std::fs;
use std::path::Path;
use std::process::exit;
use rayon::prelude::*;

use crate::config::Config;
use crate::tag::Tag;
use crate::ruby;
use crate::javascript;
use crate::rust;
use crate::haskell;
use crate::nix;

pub struct Tagger<'a> {
    pub context: TagsContext,
    pub ruby_config: TagsConfiguration,
    pub javascript_config: TagsConfiguration,
    pub rust_config: TagsConfiguration,
    pub haskell_config: TagsConfiguration,
    pub nix_config: TagsConfiguration,
    pub config: &'a Config,
}

impl Tagger<'_> {
    pub fn new(config: &'_ Config) -> Tagger<'_> {
        let context       = TagsContext::new();
        let ruby_config       = ruby::config();
        let javascript_config = javascript::config();
        let rust_config       = rust::config();
        let haskell_config    = haskell::config();
        let nix_config        = nix::config();

        Tagger { 
            config,
            context,
            ruby_config,
            javascript_config,
            rust_config,
            haskell_config,
            nix_config
        }
    }

    pub fn run(&mut self, files: &[String]) {
        let tags: Vec<Tag> = files.iter().flat_map(|filename| {
            self.read_and_parse(filename)
        }).collect();

        self.write(tags);
    }

    pub fn read_and_parse(&mut self, filename: &str) -> Vec<Tag> {
        match fs::read(filename) {
            Ok(contents) => {
                self.parse(filename, &contents)
            },
            Err(_) => {
                if self.config.appending() {
                    vec![]
                } else {
                    println!("{} not found", filename);
                    exit(1)
                }
            }
        }
    }

    fn parse(&mut self, filename: &str, contents: &[u8]) -> Vec<Tag> {
        let path = Path::new(filename);

        match path.extension() {
            Some(os_str) => {
                match os_str.to_str() {
                    Some("rb") => ruby::generate_tags(&mut self.context, &self.ruby_config, filename, contents),
                    Some("js") => javascript::generate_tags(&mut self.context, &self.javascript_config, filename, contents),
                    Some("rs") => rust::generate_tags(&mut self.context, &self.rust_config, filename, contents),
                    Some("hs") => haskell::generate_tags(&mut self.context, &self.haskell_config, filename, contents),
                    Some("nix") => nix::generate_tags(&mut self.context, &self.nix_config, filename, contents),
                    _ => vec![]
                }
            },
            None => vec![]
        }
    }

    pub fn write(&mut self, mut tags: Vec<Tag>) {
        let mut output = self.config.output();
        if self.config.appending() {
            tags.extend(self.config.current_tag_contents().iter().filter(|tag| {
                !self.config.files.contains(&tag.filename)
            }).cloned().collect::<Vec<Tag>>());
        }

        self.config.clear_tag_file();
        tags.par_sort_by_key(|tag| tag.name.clone());

        if !self.config.going_to_stdout() {
            output.write_all("!_TAG_FILE_FORMAT\t2\t/extended format; --format=1 will not append ;\" to lines/\n".as_bytes()).unwrap();
            output.write_all("!_TAG_FILE_SORTED\t1\t/0=unsorted, 1=sorted, 2=foldcase/\n".as_bytes()).unwrap();
        }
        tags.iter().for_each(|tag| {
            output.write_all(&tag.as_bytes(self.config)).unwrap()
        });
    }
}
