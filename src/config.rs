use clap::{App, Arg, ArgMatches};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use sugar_path::SugarPath;
use std::env;
use pathdiff::diff_paths;
use std::io::{self, Write, BufWriter};
use std::fs::OpenOptions;
use walkdir::WalkDir;

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
        let append        = matches.is_present("append");

        Self { files, tag_path, relative_path, append }
    }

    pub fn path_relative_to_file(&self, filename: &str) -> String {
        Self::path_to_string(diff_paths(&filename, &self.relative_path).unwrap())
    }

    pub fn output(&self) -> BufWriter<Box<dyn Write>> {
        BufWriter::new(
            if self.going_to_stdout() {
                Box::new(io::stdout())
            } else {
                let mut options = OpenOptions::new();
                let options = options.append(self.should_append()).write(true).read(true).create(true);

                Box::new(options.open(&self.tag_path).unwrap())
            }
        )
    }

    pub fn should_append(&self) -> bool {
        self.append && !self.going_to_stdout()
    }

    fn going_to_stdout(&self) -> bool {
        Path::new(&self.tag_path).file_name() == Some(OsStr::new("-"))
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
            .short("a")
            .long("append")
            .takes_value(false)
            .help("Append tags to existing file")
    }

    fn fetch_files(matches: &ArgMatches<'_>) -> Vec<String> {
        matches.values_of("files").unwrap().flat_map(|f| {
            let path = Path::new(&f).resolve();

            if path.is_dir() {
                WalkDir::new(path).into_iter().
                    filter(|entry| entry.as_ref().unwrap().path().is_file()).
                    map(|entry| {
                        Self::path_to_string(entry.unwrap().path().to_path_buf())
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
        let tag_file = Self::fetch_tag_file(matches);

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
