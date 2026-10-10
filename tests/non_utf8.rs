use std::path::Path;
use std::process::Command;

fn assert_tags(source: &str, expected: &str) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/non_utf8");
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
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

#[test]
fn skips_extensionless_non_utf8_files() {
    // A Git pack header followed by an invalid UTF-8 byte reproduces issue #19.
    // No extension means this input reaches Ruby shebang detection.
    assert_tags("tmp_pack", "");
}

#[test]
fn directory_with_non_utf8_data_still_tags_sources() {
    assert_tags(".", "Café\tsource.rb\t1;\"\tc\nScript\tscript\t2;\"\tc\n");
}

#[test]
fn still_recognizes_extensionless_ruby_scripts() {
    assert_tags("script", "Script\tscript\t2;\"\tc\n");
}
