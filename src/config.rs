use clap::{App, Arg};

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Config {
    pub files: Vec<String>,
    pub tag_file: String
}

impl Config {
    pub fn new() -> Self {
        let matches = Self::menu().get_matches();

        let files: Vec<String> =
            matches.values_of("files").unwrap().map(|f| f.to_string()).collect();
        let tag_file: String = matches.value_of("tag_file").unwrap().to_string();

        Self {
            files,
            tag_file,
        }
    }

    pub fn menu<'a>() -> App<'a, 'a> {
        App::new("ttags")
            .version(clap::crate_version!())
            .author(crate_authors!())
            .arg(Self::files_arg())
            .arg(Self::tag_file_arg())
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
}
