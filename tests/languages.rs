use std::path::Path;
use std::process::Command;

fn assert_tags(source: &str, expected: &str) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/languages");
    let output = Command::new(env!("CARGO_BIN_EXE_ttags"))
        .current_dir(fixtures)
        .args(["--tag-file", "-", source])
        .output()
        .expect("run ttags");

    assert!(
        output.status.success(),
        "{source}: ttags failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{source}: unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("tags must be UTF-8"),
        expected,
        "{source}: tag names, paths, line numbers, kinds and ordering must match"
    );
}

macro_rules! language_fixture {
    ($test:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $test() {
            assert_tags(
                concat!("naïve/", $source),
                include_str!(concat!("fixtures/languages/", $expected)),
            );
        }
    };
}

language_fixture!(c_tags, "sample.c", "sample.c.tags");
language_fixture!(cpp_tags, "sample.cpp", "sample.cpp.tags");
language_fixture!(haskell_tags, "sample.hs", "sample.hs.tags");
language_fixture!(javascript_tags, "sample.js", "sample.js.tags");
language_fixture!(nix_tags, "sample.nix", "sample.nix.tags");
language_fixture!(ruby_tags, "sample.rb", "sample.rb.tags");
language_fixture!(rust_tags, "sample.rs", "sample.rs.tags");
language_fixture!(swift_tags, "sample.swift", "sample.swift.tags");
language_fixture!(
    ruby_metadata_and_delegates,
    "metadata.rb",
    "metadata.rb.tags"
);

language_fixture!(d_tags, "sample.d", "sample.d.tags");
language_fixture!(d_aliases, "aliases.d", "aliases.d.tags");
language_fixture!(d_interface, "interface.di", "interface.di.tags");

#[test]
fn d_fixtures_parse_without_errors() {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_d::LANGUAGE.into())
        .unwrap();
    for (name, source) in [
        (
            "sample.d",
            include_str!("fixtures/languages/naïve/sample.d"),
        ),
        (
            "aliases.d",
            include_str!("fixtures/languages/naïve/aliases.d"),
        ),
        (
            "interface.di",
            include_str!("fixtures/languages/naïve/interface.di"),
        ),
    ] {
        let tree = parser.parse(source, None).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "{name} must exercise valid grammar, not error recovery: {}",
            tree.root_node().to_sexp()
        );
    }
}
