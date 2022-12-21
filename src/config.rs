use clap::{App, Arg};

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Config {
    pub files: Vec<String>
}

impl Config {
    pub fn new() -> Self {
        let matches = Self::menu().get_matches();

        let files: Vec<String> =
            matches.values_of("files").unwrap().map(|f| f.to_string()).collect();

        Self {
            files,
        }
    }

    pub fn menu<'a>() -> App<'a, 'a> {
        App::new("ttags")
            .version(clap::crate_version!())
            .author(crate_authors!())
            .arg(Self::files_arg())
    }

    fn files_arg<'a>() -> Arg<'a, 'a> {
        Arg::with_name("files")
            .multiple(true)
            .help("Specify files to parse for tags")
            .required(true)
    }
}
