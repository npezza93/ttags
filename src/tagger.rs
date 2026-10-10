use npezza93_tree_sitter_tags::{TagsConfiguration, TagsContext};
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use crate::c;
use crate::config::Config;
use crate::cpp;
use crate::d;
use crate::haskell;
use crate::javascript;
use crate::nix;
use crate::ruby;
use crate::rust;
use crate::swift;
use crate::tag::Tag;

#[derive(Clone, Copy)]
enum Language {
    D,
    Ruby,
    JavaScript,
    Rust,
    Haskell,
    Nix,
    Swift,
    C,
    Cpp,
}

impl Language {
    fn from_extension(extension: &std::ffi::OsStr) -> Option<Self> {
        match extension.to_str()? {
            "d" | "di" => Some(Self::D),
            "rb" => Some(Self::Ruby),
            "js" => Some(Self::JavaScript),
            "rs" => Some(Self::Rust),
            "hs" => Some(Self::Haskell),
            "nix" => Some(Self::Nix),
            "swift" => Some(Self::Swift),
            "c" | "h" | "i" => Some(Self::C),
            "cc" | "cpp" | "CPP" | "cxx" | "c++" | "cp" | "C" | "cppm" | "ixx" | "ii" | "H"
            | "hh" | "hpp" | "HPP" | "hxx" | "h++" | "tcc" => Some(Self::Cpp),
            _ => None,
        }
    }
}

pub struct Tagger<'a> {
    pub context: TagsContext,
    pub d_config: TagsConfiguration,
    pub ruby_config: TagsConfiguration,
    pub javascript_config: TagsConfiguration,
    pub rust_config: TagsConfiguration,
    pub haskell_config: TagsConfiguration,
    pub nix_config: TagsConfiguration,
    pub swift_config: TagsConfiguration,
    pub c_config: TagsConfiguration,
    pub cpp_config: TagsConfiguration,
    pub config: &'a Config,
}

impl Tagger<'_> {
    pub fn new(config: &'_ Config) -> Tagger<'_> {
        let context = TagsContext::new();
        let d_config = d::config();
        let ruby_config = ruby::config();
        let javascript_config = javascript::config();
        let rust_config = rust::config();
        let haskell_config = haskell::config();
        let nix_config = nix::config();
        let swift_config = swift::config();
        let c_config = c::config();
        let cpp_config = cpp::config();

        Tagger {
            config,
            context,
            d_config,
            ruby_config,
            javascript_config,
            rust_config,
            haskell_config,
            nix_config,
            swift_config,
            c_config,
            cpp_config,
        }
    }

    pub fn run(&mut self, files: &[String]) {
        let tags: Vec<Tag> = files
            .iter()
            .flat_map(|filename| self.read_and_parse(filename))
            .collect();

        self.write(tags);
    }

    pub fn read_and_parse(&mut self, filename: &str) -> Vec<Tag> {
        if Path::new(filename)
            .extension()
            .is_some_and(|extension| Language::from_extension(extension).is_none())
        {
            return vec![];
        }

        match fs::read(filename) {
            Ok(contents) => self.parse(filename, &contents),
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

        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.is_file() {
                    match path.extension() {
                        Some(os_str) => Language::from_extension(os_str)
                            .map(|language| self.type_mapping(language, filename, contents))
                            .unwrap_or_default(),
                        None => {
                            let regex =
                                Regex::new(r"^#!\s*/usr/bin/env\s+(?P<command>.*)").unwrap();

                            match std::str::from_utf8(contents) {
                                Ok(str_slice) => match regex.captures(str_slice) {
                                    Some(regex_match) => match regex_match.name("command") {
                                        Some(command) => {
                                            if command.as_str() == "ruby" {
                                                self.type_mapping(
                                                    Language::Ruby,
                                                    filename,
                                                    contents,
                                                )
                                            } else {
                                                vec![]
                                            }
                                        }
                                        None => vec![],
                                    },
                                    None => vec![],
                                },
                                Err(_) => vec![],
                            }
                        }
                    }
                } else {
                    vec![]
                }
            }
            Err(_e) => {
                vec![]
            }
        }
    }

    fn type_mapping(&mut self, kind: Language, filename: &str, contents: &[u8]) -> Vec<Tag> {
        match kind {
            Language::D => d::generate_tags(&mut self.context, &self.d_config, filename, contents),
            Language::Ruby => {
                ruby::generate_tags(&mut self.context, &self.ruby_config, filename, contents)
            }
            Language::JavaScript => javascript::generate_tags(
                &mut self.context,
                &self.javascript_config,
                filename,
                contents,
            ),
            Language::Rust => {
                rust::generate_tags(&mut self.context, &self.rust_config, filename, contents)
            }
            Language::Haskell => {
                haskell::generate_tags(&mut self.context, &self.haskell_config, filename, contents)
            }
            Language::Nix => {
                nix::generate_tags(&mut self.context, &self.nix_config, filename, contents)
            }
            Language::Swift => {
                swift::generate_tags(&mut self.context, &self.swift_config, filename, contents)
            }
            Language::C => c::generate_tags(&mut self.context, &self.c_config, filename, contents),
            Language::Cpp => {
                c::generate_tags(&mut self.context, &self.cpp_config, filename, contents)
            }
        }
    }

    pub fn write(&mut self, mut tags: Vec<Tag>) {
        let mut output = self.config.output();
        if self.config.appending() {
            tags.extend(
                self.config
                    .current_tag_contents()
                    .iter()
                    .filter(|tag| !self.config.files.contains(&tag.filename))
                    .cloned()
                    .collect::<Vec<Tag>>(),
            );
        }

        self.config.clear_tag_file();
        tags.par_sort_by_key(|tag| tag.name.clone());

        if !self.config.going_to_stdout() {
            output.write_all("!_TAG_FILE_FORMAT\t2\t/extended format; --format=1 will not append ;\" to lines/\n".as_bytes()).unwrap();
            output
                .write_all("!_TAG_FILE_SORTED\t1\t/0=unsorted, 1=sorted, 2=foldcase/\n".as_bytes())
                .unwrap();
        }
        tags.iter()
            .for_each(|tag| output.write_all(&tag.as_bytes(self.config)).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::Language;
    use std::ffi::OsStr;

    #[test]
    fn preserves_supported_extensions() {
        for extension in [
            "d", "di", "rb", "js", "rs", "hs", "nix", "swift", "c", "h", "i", "cc", "cpp", "CPP",
            "cxx", "c++", "cp", "C", "cppm", "ixx", "ii", "H", "hh", "hpp", "HPP", "hxx", "h++",
            "tcc",
        ] {
            assert!(
                Language::from_extension(OsStr::new(extension)).is_some(),
                "{extension}"
            );
        }
        for extension in ["png", "json", "md", "txt", "RB", "RS", ""] {
            assert!(
                Language::from_extension(OsStr::new(extension)).is_none(),
                "{extension}"
            );
        }
    }
}
