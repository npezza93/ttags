use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Project(PathBuf);

impl Project {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ttags-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, filename: &str, contents: &str) {
        fs::write(self.0.join(filename), contents).unwrap();
    }

    fn run(&self, args: &[&str], input: &[u8]) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ttags"))
            .current_dir(&self.0)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        // Write concurrently so a large input cannot deadlock with output.
        let input = input.to_vec();
        let writer = std::thread::spawn(move || stdin.write_all(&input));
        let output = child.wait_with_output().unwrap();
        let _ = writer.join().unwrap();
        output
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn large_nul_list_skips_unsupported_files_before_reading() {
    let project = Project::new();
    project.write("source.rs", "fn large_project() {}\n");
    let mut input = Vec::new();
    for index in 0..50_000 {
        write!(input, "assets/{index:06}-{}.png\0", "x".repeat(80)).unwrap();
    }
    input.extend_from_slice(b"source.rs\0");
    assert!(input.len() > 4_000_000);
    let output = stdout(project.run(&["-f", "-", "--null", "--files-from", "-"], &input));
    assert!(output.starts_with("large_project\tsource.rs\t"), "{output}");
    assert_eq!(output.lines().count(), 1);
}

#[test]
fn nul_list_preserves_special_characters_and_unterminated_last_path() {
    let project = Project::new();
    let filenames = [
        " space name .rs",
        "quote'\".rs",
        "-option.rs",
        "line\nbreak.rs",
    ];
    for (index, filename) in filenames.iter().enumerate() {
        project.write(filename, &format!("fn symbol_{index}() {{}}\n"));
    }
    let output = stdout(project.run(
        &["-f", "-", "-0", "-L", "-"],
        filenames.join("\0").as_bytes(),
    ));
    for index in 0..filenames.len() {
        assert!(output.contains(&format!("symbol_{index}\t")), "{output}");
    }
}

#[test]
fn newline_file_list_combines_with_positional_paths_and_directories() {
    let project = Project::new();
    project.write(" space .rs", "fn listed() {}\n");
    project.write("positional.rs", "fn positional() {}\n");
    fs::create_dir(project.0.join("directory")).unwrap();
    project.write("directory/nested.rs", "fn nested() {}\n");
    project.write("list", "\r\n space .rs\r\ndirectory");
    let output = stdout(project.run(&["-f", "-", "-L", "list", "positional.rs"], b""));
    assert_eq!(output.lines().count(), 3);
    for symbol in ["listed", "positional", "nested"] {
        assert!(output.contains(&format!("{symbol}\t")), "{output}");
    }
}

#[test]
fn empty_list_is_valid_and_null_requires_a_list() {
    let project = Project::new();
    assert!(stdout(project.run(&["-f", "-", "-L", "-"], b"")).is_empty());
    assert!(
        !project
            .run(&["--null", "anything.rs"], b"")
            .status
            .success()
    );
    assert!(!project.run(&[], b"").status.success());
}

#[test]
fn invalid_lists_report_errors() {
    let project = Project::new();
    for args in [
        vec!["-L", "missing-list"],
        vec!["-0", "-L", "-"],
        vec!["-L", "-"],
    ] {
        let output = project.run(&args, b"\xff\0");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Failed reading file list:"));
    }
}

#[test]
fn unsupported_inputs_are_skipped_but_missing_supported_files_still_fail() {
    let project = Project::new();
    assert!(stdout(project.run(&["-f", "-", "missing.png"], b"")).is_empty());
    assert!(
        !project
            .run(&["-f", "-", "missing.rs"], b"")
            .status
            .success()
    );
}

#[test]
fn supported_languages_and_extensionless_ruby_still_generate_tags() {
    let project = Project::new();
    let sources = [
        ("sample.rb", "def ruby_symbol; end\n", "ruby_symbol"),
        (
            "sample.js",
            "function javascript_symbol() {}\n",
            "javascript_symbol",
        ),
        ("sample.rs", "fn rust_symbol() {}\n", "rust_symbol"),
        ("sample.hs", "haskell_symbol x = x\n", "haskell_symbol"),
        ("sample.nix", "{ nix_symbol = x: x; }\n", "nix_symbol"),
        ("sample.swift", "func swift_symbol() {}\n", "swift_symbol"),
        ("sample.c", "void c_symbol(void) {}\n", "c_symbol"),
        ("sample.cpp", "void cpp_symbol() {}\n", "cpp_symbol"),
        (
            "ruby-script",
            "#!/usr/bin/env ruby\ndef script_symbol; end\n",
            "script_symbol",
        ),
    ];
    for (filename, contents, _) in &sources {
        project.write(filename, contents);
    }
    let filenames: Vec<_> = sources.iter().map(|(filename, _, _)| *filename).collect();
    let output = stdout(project.run(
        &["-f", "-", "-0", "-L", "-"],
        filenames.join("\0").as_bytes(),
    ));
    for (_, _, symbol) in sources {
        assert!(
            output.contains(&format!("{symbol}\t")),
            "missing {symbol}: {output}"
        );
    }
}

#[test]
fn file_lists_work_with_append() {
    let project = Project::new();
    project.write("updated.rs", "fn old_symbol() {}\n");
    project.write("kept.rs", "fn kept_symbol() {}\n");
    stdout(project.run(&["updated.rs", "kept.rs"], b""));
    project.write("updated.rs", "fn new_symbol() {}\n");
    stdout(project.run(&["--append", "-L", "-"], b"updated.rs\n"));
    let tags = fs::read_to_string(project.0.join("tags")).unwrap();
    assert!(tags.contains("new_symbol\t"));
    assert!(tags.contains("kept_symbol\t"));
    assert!(!tags.contains("old_symbol\t"));
}
