use npezza93_tree_sitter_tags::{TagsConfiguration, TagsContext};
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
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

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
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

    fn configuration(self) -> TagsConfiguration {
        match self {
            Self::D => d::config(),
            Self::Ruby => ruby::config(),
            Self::JavaScript => javascript::config(),
            Self::Rust => rust::config(),
            Self::Haskell => haskell::config(),
            Self::Nix => nix::config(),
            Self::Swift => swift::config(),
            Self::C => c::config(),
            Self::Cpp => cpp::config(),
        }
    }

    fn generate_tags(
        self,
        context: &mut TagsContext,
        config: &TagsConfiguration,
        filename: &str,
        contents: &[u8],
    ) -> Vec<Tag> {
        match self {
            Self::D => d::generate_tags(context, config, filename, contents),
            Self::Ruby => ruby::generate_tags(context, config, filename, contents),
            Self::JavaScript => javascript::generate_tags(context, config, filename, contents),
            Self::Rust => rust::generate_tags(context, config, filename, contents),
            Self::Haskell => haskell::generate_tags(context, config, filename, contents),
            Self::Nix => nix::generate_tags(context, config, filename, contents),
            Self::Swift => swift::generate_tags(context, config, filename, contents),
            Self::C | Self::Cpp => c::generate_tags(context, config, filename, contents),
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
        match read_source(filename, self.config.appending()).unwrap_or_else(file_not_found) {
            Some((language, contents)) => {
                let config = match language {
                    Language::D => &self.d_config,
                    Language::Ruby => &self.ruby_config,
                    Language::JavaScript => &self.javascript_config,
                    Language::Rust => &self.rust_config,
                    Language::Haskell => &self.haskell_config,
                    Language::Nix => &self.nix_config,
                    Language::Swift => &self.swift_config,
                    Language::C => &self.c_config,
                    Language::Cpp => &self.cpp_config,
                };
                language.generate_tags(&mut self.context, config, filename, &contents)
            }
            None => vec![],
        }
    }

    pub fn write(&mut self, tags: Vec<Tag>) {
        write_tags(self.config, tags);
    }
}

// Keep the public Tagger's eager, caller-replaceable configurations intact. The
// CLI owns these private parsers instead, so only languages in each chunk need
// to be initialized. TagsConfiguration is not Send or Sync; construct and drop
// it on the worker that uses it rather than sharing it between threads.
struct FileParser {
    context: TagsContext,
    configurations: HashMap<Language, TagsConfiguration>,
}

impl FileParser {
    fn new() -> Self {
        Self {
            context: TagsContext::new(),
            configurations: HashMap::new(),
        }
    }

    fn parse_files<'a>(
        &mut self,
        files: &'a [String],
        appending: bool,
    ) -> Result<Vec<Tag>, &'a str> {
        let mut tags = Vec::new();
        for filename in files {
            if let Some((language, contents)) = read_source(filename, appending)? {
                let configuration = self
                    .configurations
                    .entry(language)
                    .or_insert_with(|| language.configuration());
                tags.extend(language.generate_tags(
                    &mut self.context,
                    configuration,
                    filename,
                    &contents,
                ));
            }
        }
        Ok(tags)
    }
}

const MAX_PARSER_WORKERS: usize = 8;
const TARGET_FILES_PER_WORKER: usize = 64;

pub(crate) fn run_batch(config: &Config) {
    let files = &config.files;
    let chunks = if files.len() <= TARGET_FILES_PER_WORKER {
        vec![FileParser::new().parse_files(files, config.appending())]
    } else {
        // Limit duplicate grammar/query state even on hosts with many CPUs.
        // Coarse chunks also avoid initializing a new parser for every file.
        let workers = files
            .len()
            .div_ceil(TARGET_FILES_PER_WORKER)
            .min(MAX_PARSER_WORKERS)
            .min(rayon::current_num_threads());
        files
            .par_chunks(files.len().div_ceil(workers))
            .map(|chunk| FileParser::new().parse_files(chunk, config.appending()))
            .collect::<Vec<_>>()
    };

    // Indexed parallel collection retains input order. Resolve failures here
    // so the first missing input is deterministic; stable sorting also keeps
    // equal-name tags in their original file and per-file order.
    let tags = chunks
        .into_iter()
        .flat_map(|chunk| chunk.unwrap_or_else(file_not_found))
        .collect();
    write_tags(config, tags);
}

fn file_not_found<T>(filename: &str) -> T {
    println!("{} not found", filename);
    exit(1)
}

fn read_source(filename: &str, appending: bool) -> Result<Option<(Language, Vec<u8>)>, &str> {
    let path = Path::new(filename);
    let extension = path.extension();
    if extension.is_some_and(|extension| Language::from_extension(extension).is_none()) {
        return Ok(None);
    }

    let contents = match fs::read(path) {
        Ok(contents) => contents,
        Err(_) if appending => return Ok(None),
        Err(_) => return Err(filename),
    };
    if !fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
        return Ok(None);
    }

    let language = match extension {
        Some(extension) => Language::from_extension(extension),
        None => {
            let regex = Regex::new(r"^#!\s*/usr/bin/env\s+(?P<command>.*)").unwrap();
            std::str::from_utf8(&contents)
                .ok()
                .and_then(|contents| regex.captures(contents))
                .and_then(|captures| captures.name("command"))
                .filter(|command| command.as_str() == "ruby")
                .map(|_| Language::Ruby)
        }
    };
    Ok(language.map(|language| (language, contents)))
}

fn write_tags(config: &Config, mut tags: Vec<Tag>) {
    let mut output = config.output();
    if config.appending() {
        tags.extend(
            config
                .current_tag_contents()
                .iter()
                .filter(|tag| !config.files.contains(&tag.filename))
                .cloned()
                .collect::<Vec<Tag>>(),
        );
    }

    config.clear_tag_file();
    tags.par_sort_by_key(|tag| tag.name.clone());

    if !config.going_to_stdout() {
        output.write_all("!_TAG_FILE_FORMAT\t2\t/extended format; --format=1 will not append ;\" to lines/\n".as_bytes()).unwrap();
        output
            .write_all("!_TAG_FILE_SORTED\t1\t/0=unsorted, 1=sorted, 2=foldcase/\n".as_bytes())
            .unwrap();
    }
    tags.iter()
        .for_each(|tag| output.write_all(&tag.as_bytes(config)).unwrap());
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
