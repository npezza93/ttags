use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use sugar_path::SugarPath;
use std::env;
use pathdiff::diff_paths;
use std::io::{self, Write, BufWriter, BufReader, Read};
use std::fs::{File, OpenOptions};
use walkdir::WalkDir;

use crate::tag::Tag;

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
    pub lsp: bool,
}

impl Config {
    pub fn new() -> Self {
        let matches = Self::menu().get_matches();

        let lsp           = matches.subcommand_name() == Some("lsp");
        let files         = Self::fetch_files(&matches, lsp);
        let tag_path      = Self::path_to_string(Self::fetch_tag_file(&matches));
        let relative_path = Self::path_to_string(Self::fetch_relative_path(&matches));
        let append        = matches.is_present("append") || lsp;

        Self { files, tag_path, relative_path, append, lsp }
    }

    pub fn path_relative_to_file(&self, filename: &str) -> String {
        Self::path_to_string(diff_paths(filename, &self.relative_path).unwrap())
    }

    pub fn output(&self) -> BufWriter<Box<dyn Write>> {
        BufWriter::new(
            if self.going_to_stdout() {
                Box::new(io::stdout())
            } else {
                let mut options = OpenOptions::new();
                let options = options.write(true).read(true).create(true);

                Box::new(options.open(&self.tag_path).unwrap())
            }
        )
    }

    pub fn appending(&self) -> bool {
        self.append && !self.going_to_stdout()
    }

    pub fn clear_tag_file(&self) {
        if !self.going_to_stdout() {
            File::create(&self.tag_path).expect("Failed clearing file");
        }
    }

    pub fn current_tag_contents(&self) -> Vec<Tag> {
        let file = File::open(&self.tag_path).expect("Failed reading tags file");
        let mut reader = BufReader::new(file);
        let mut contents = String::new();

        reader.read_to_string(&mut contents).unwrap();

        contents.trim().split('\n').
            filter(|line| !line.starts_with("!_")).
            map(Tag::parse).collect()
    }

    pub fn going_to_stdout(&self) -> bool {
        Path::new(&self.tag_path).file_name() == Some(OsStr::new("-"))
    }

    fn menu<'a>() -> App<'a, 'a> {
        App::new("ttags")
            .version(crate_version!())
            .author(crate_authors!())
            .setting(AppSettings::SubcommandsNegateReqs)
            .arg(Self::files_arg())
            .arg(Self::tag_file_arg())
            .arg(Self::relative_arg())
            .arg(Self::append_arg())
            .subcommand(Self::lsp_subcommand())
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
            .required(false)
            .help("File to write tags to. Use '-' to output to stdout")
            .default_value("./tags")
    }

    fn relative_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("relative")
            .long("relative")
            .short("r")
            .takes_value(false)
            .help("Should paths be relative to cwd? By default, it's relative to tag-file")
    }

    fn append_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("append")
            .short("a")
            .long("append")
            .takes_value(false)
            .help("Append tags to existing file")
    }

    fn lsp_subcommand<'a>() -> App<'a, 'a> {
        SubCommand::with_name("lsp")
            .about("Creates lsp server")
            .setting(AppSettings::DisableVersion)
    }

    fn fetch_files(matches: &ArgMatches<'_>, lsp: bool) -> Vec<String> {
        if lsp {
            vec![]
        } else {
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
    }

    fn fetch_tag_file(matches: &ArgMatches<'_>) -> PathBuf {
        Path::new(matches.value_of("tag_file").unwrap()).resolve()
    }

    fn fetch_relative_path(matches: &ArgMatches<'_>) -> PathBuf {
        let tag_file = Self::fetch_tag_file(matches);

        if !matches.is_present("relative") &&
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
