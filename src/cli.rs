use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("ttags")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommand_negates_reqs(true)
        .subcommand_precedence_over_arg(true)
        .arg(files_arg())
        .arg(tag_file_arg())
        .arg(relative_arg())
        .arg(append_arg())
        .subcommand(lsp_subcommand())
}

fn files_arg() -> Arg {
    Arg::new("files")
        .num_args(1..)
        .help("Specify files to parse for tags")
        .required(true)
}

fn tag_file_arg() -> Arg {
    Arg::new("tag_file")
        .short('f')
        .long("tag-file")
        .value_name("FILE|-")
        .action(ArgAction::Set)
        .required(false)
        .help("File to write tags to. Use '-' to output to stdout")
        .default_value("./tags")
}

fn relative_arg() -> Arg {
    Arg::new("relative")
        .long("relative")
        .short('r')
        .action(ArgAction::SetTrue)
        .help("Should paths be relative to cwd? By default, it's relative to tag-file")
}

fn append_arg() -> Arg {
    Arg::new("append")
        .short('a')
        .long("append")
        .action(ArgAction::SetTrue)
        .help("Append tags to existing file")
}

fn lsp_subcommand() -> Command {
    Command::new("lsp")
        .about("Creates lsp server")
        .disable_version_flag(true)
}

#[cfg(test)]
mod tests {
    use super::build_cli;

    #[test]
    fn accepts_files_and_output_options() {
        let matches = build_cli()
            .try_get_matches_from(["ttags", "first.rb", "second.rs", "-arf", "-"])
            .unwrap();
        assert_eq!(
            matches
                .get_many::<String>("files")
                .unwrap()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["first.rb", "second.rs"]
        );
        assert_eq!(matches.get_one::<String>("tag_file").unwrap(), "-");
        assert!(matches.get_flag("append"));
        assert!(matches.get_flag("relative"));
    }

    #[test]
    fn requires_files_except_for_lsp() {
        assert!(build_cli().try_get_matches_from(["ttags"]).is_err());
        let matches = build_cli().try_get_matches_from(["ttags", "lsp"]).unwrap();
        assert_eq!(matches.subcommand_name(), Some("lsp"));
        assert_eq!(matches.get_one::<String>("tag_file").unwrap(), "./tags");
    }

    #[test]
    fn recognizes_lsp_after_files() {
        let matches = build_cli()
            .try_get_matches_from(["ttags", "sample.rb", "lsp"])
            .unwrap();
        assert_eq!(matches.subcommand_name(), Some("lsp"));
    }

    #[test]
    fn accepts_paths_after_option_terminator() {
        let matches = build_cli()
            .try_get_matches_from(["ttags", "--", "-name.rb", "lsp"])
            .unwrap();
        assert_eq!(
            matches
                .get_many::<String>("files")
                .unwrap()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["-name.rb", "lsp"]
        );
    }
}
