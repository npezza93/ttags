use clap::{App, Arg, ArgMatches};
use std::path::Path;
use std::env;

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
        let tag_path      = Self::fetch_tag_file(&matches);
        let relative_path = Self::fetch_relative_path(&matches);
        let append        = matches.value_of("append") == Some("yes");

        Self { files, tag_path, relative_path, append }
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
            .value_name("FILE")
            .takes_value(true)
            .help("File to write tags to")
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
        matches.values_of("files").unwrap().map(|f| {
            Path::new(&f).canonicalize().unwrap().
                into_os_string().into_string().unwrap()
        }).collect()
    }

    fn fetch_tag_file(matches: &ArgMatches<'_>) -> String {
        Path::new(matches.value_of("tag_file").unwrap()).
            canonicalize().unwrap().into_os_string().into_string().unwrap()
    }

    fn fetch_relative_path(matches: &ArgMatches<'_>) -> String {
        if matches.value_of("relative") == Some("yes") {
            Self::fetch_tag_file(&matches)
        } else {
            env::current_dir().unwrap().into_os_string().into_string().unwrap()
        }
    }
}
