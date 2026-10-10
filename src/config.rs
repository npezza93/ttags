use clap::ArgMatches;
use pathdiff::diff_paths;
use std::env;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use sugar_path::SugarPath;
use walkdir::WalkDir;

use crate::cli;
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
        let matches = cli::build_cli().get_matches();

        let lsp = matches.subcommand_name() == Some("lsp");
        let files = Self::fetch_files(&matches, lsp).unwrap_or_else(|error| {
            eprintln!("Failed reading file list: {error}");
            std::process::exit(1);
        });
        let tag_path = Self::path_to_string(Self::fetch_tag_file(&matches));
        let relative_path = Self::path_to_string(Self::fetch_relative_path(&matches));
        let append = matches.get_flag("append") || lsp;

        Self {
            files,
            tag_path,
            relative_path,
            append,
            lsp,
        }
    }

    pub fn path_relative_to_file(&self, filename: &str) -> String {
        Self::path_to_string(diff_paths(filename, &self.relative_path).unwrap())
    }

    pub fn output(&self) -> BufWriter<Box<dyn Write>> {
        BufWriter::new(if self.going_to_stdout() {
            Box::new(io::stdout())
        } else {
            let mut options = OpenOptions::new();
            let options = options.write(true).read(true).create(true);

            Box::new(options.open(&self.tag_path).unwrap())
        })
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

        contents
            .trim()
            .split('\n')
            .filter(|line| !line.starts_with("!_") && !line.is_empty())
            .map(Tag::parse)
            .collect()
    }

    pub fn going_to_stdout(&self) -> bool {
        Path::new(&self.tag_path).file_name() == Some(OsStr::new("-"))
    }

    fn fetch_files(matches: &ArgMatches, lsp: bool) -> io::Result<Vec<String>> {
        if lsp {
            return Ok(vec![]);
        }

        let mut paths: Vec<String> = matches
            .get_many::<String>("files")
            .into_iter()
            .flatten()
            .cloned()
            .collect();

        if let Some(filename) = matches.get_one::<String>("files_from") {
            let reader: Box<dyn BufRead> = if filename == "-" {
                Box::new(io::stdin().lock())
            } else {
                Box::new(BufReader::new(File::open(filename)?))
            };
            paths.extend(Self::read_file_list(reader, matches.get_flag("null"))?);
        }

        Ok(paths
            .iter()
            .flat_map(|f| {
                let path = Path::new(f).absolutize().into_owned();

                if path.is_dir() {
                    WalkDir::new(path)
                        .into_iter()
                        .filter(|entry| entry.as_ref().unwrap().path().is_file())
                        .map(|entry| Self::path_to_string(entry.unwrap().path().to_path_buf()))
                        .collect()
                } else {
                    vec![Self::path_to_string(path)]
                }
            })
            .collect())
    }

    fn read_file_list(reader: impl BufRead, null: bool) -> io::Result<Vec<String>> {
        if null {
            reader
                .split(0)
                .filter_map(|path| match path {
                    Ok(path) if path.is_empty() => None,
                    path => Some(path.and_then(|path| {
                        String::from_utf8(path)
                            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
                    })),
                })
                .collect()
        } else {
            reader
                .lines()
                .filter(|path| !matches!(path, Ok(path) if path.is_empty()))
                .collect()
        }
    }

    fn fetch_tag_file(matches: &ArgMatches) -> PathBuf {
        Path::new(matches.get_one::<String>("tag_file").unwrap())
            .absolutize()
            .into_owned()
    }

    fn fetch_relative_path(matches: &ArgMatches) -> PathBuf {
        let tag_file = Self::fetch_tag_file(matches);

        if !matches.get_flag("relative") && tag_file.as_path().file_name() != Some(OsStr::new("-"))
        {
            tag_file.parent().unwrap().to_path_buf()
        } else {
            env::current_dir().unwrap()
        }
    }

    fn path_to_string(path: PathBuf) -> String {
        path.into_os_string().into_string().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::cli;

    #[test]
    fn lsp_does_not_read_file_lists() {
        let matches = cli::build_cli()
            .try_get_matches_from([
                "ttags",
                "--files-from",
                "missing-file-list",
                "--null",
                "lsp",
            ])
            .unwrap();
        assert_eq!(matches.subcommand_name(), Some("lsp"));
        assert!(Config::fetch_files(&matches, true).unwrap().is_empty());
    }
}
