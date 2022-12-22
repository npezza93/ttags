use clap::{App, Arg, ArgMatches};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use sugar_path::SugarPath;
use std::env;
use pathdiff::diff_paths;
use std::io::{self, Write, BufWriter};
use std::fs::{self, File};

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Config {
    pub files: Vec<String>,
    pub tag_path: String,
    pub relative_path: String,
    pub append: bool,
}

impl Config {
    pub fn new() -> Self {
        let matches = Self::menu().get_matches();

        let files         = Self::fetch_files(&matches);
        let tag_path      = Self::path_to_string(Self::fetch_tag_file(&matches));
        let relative_path = Self::path_to_string(Self::fetch_relative_path(&matches));
        let append        = matches.value_of("append") == Some("yes");

        Self { files, tag_path, relative_path, append }
    }

    pub fn path_relative_to_file(&self, filename: &str) -> String {
        Self::path_to_string(diff_paths(&filename, &self.relative_path).unwrap())
    }

    pub fn output(&self) -> BufWriter<Box<dyn Write>> {
        BufWriter::new(
            if Path::new(&self.tag_path).file_name() == Some(OsStr::new("-")) {
                Box::new(io::stdout())
            } else {
                Box::new(File::create(&self.tag_path).unwrap())
            }
        )
    }

    fn menu<'a>() -> App<'a, 'a> {
        App::new("ttags")
            .version(crate_version!())
            .author(crate_authors!())
            .arg(Self::files_arg())
            .arg(Self::tag_file_arg())
            .arg(Self::relative_arg())
            .arg(Self::append_arg())
    }

    fn files_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("files")
            .multiple(true)
            .help("Specify files to parse for tags")
            .required(true)
    }

    fn tag_file_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("tag_file")
            .short("f")
            .long("tag-file")
            .value_name("FILE|-")
            .takes_value(true)
            .help("File to write tags to. Use '-' to output to stdout")
            .default_value("./tags")
    }

    fn relative_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("relative")
            .long("tag-relative")
            .value_name("yes|no")
            .takes_value(true)
            .possible_values(&["yes", "no"])
            .help("Should paths be relative to location of tag file?")
            .default_value("yes")
    }

    fn append_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("append")
            .value_name("yes|no")
            .short("a")
            .long("append")
            .takes_value(true)
            .possible_values(&["yes", "no"])
            .help("Append tags to existing file")
            .default_value("no")
    }

    fn fetch_files(matches: &ArgMatches<'_>) -> Vec<String> {
        matches.values_of("files").unwrap().flat_map(|f| {
            let path = Path::new(&f).resolve();

            if path.is_dir() {
                fs::read_dir(path).unwrap().map(|entry| {
                    Self::path_to_string(entry.unwrap().path())
                }).collect()
            } else {
                vec![Self::path_to_string(path)]
            }
        }).collect()
    }

    fn fetch_tag_file(matches: &ArgMatches<'_>) -> PathBuf {
        Path::new(matches.value_of("tag_file").unwrap()).resolve()
    }

    fn fetch_relative_path(matches: &ArgMatches<'_>) -> PathBuf {
        let tag_file = Self::fetch_tag_file(&matches);

        if matches.value_of("relative") == Some("yes") &&
             tag_file.as_path().file_name() != Some(OsStr::new("-")) {
            tag_file.parent().unwrap().to_path_buf()
        } else {
            env::current_dir().unwrap()
        }
    }

    fn path_to_string(path: PathBuf) -> String {
        path.into_os_string().into_string().unwrap()
    }
}
