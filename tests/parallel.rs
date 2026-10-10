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
            "ttags-parallel-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        // macOS temp_dir() may use /var, while the child cwd resolves /private/var.
        Self(path.canonicalize().unwrap())
    }

    fn run(&self, files: &[String], threads: usize, args: &[&str]) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ttags"))
            .current_dir(&self.0)
            .env("RAYON_NUM_THREADS", threads.to_string())
            .args(["--null", "--files-from", "-"])
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let input = files.join("\0");
        let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
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

fn success(output: Output) -> String {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn mixed_language_batches_match_golden_tags() {
    let project = Project::new();
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/languages");
    fs::create_dir(project.0.join("naïve")).unwrap();
    let sources = [
        "sample.c",
        "sample.cpp",
        "sample.d",
        "aliases.d",
        "interface.di",
        "sample.hs",
        "sample.js",
        "sample.nix",
        "sample.rb",
        "metadata.rb",
        "sample.rs",
        "sample.swift",
    ];
    for source in sources {
        fs::copy(
            fixtures.join("naïve").join(source),
            project.0.join("naïve").join(source),
        )
        .unwrap();
    }
    fs::write(
        project.0.join("script"),
        "#!/usr/bin/env ruby\ndef script_symbol; end\n",
    )
    .unwrap();
    fs::write(project.0.join("binary"), b"PACK\xff").unwrap();
    let mut files = Vec::new();
    let mut expected = Vec::new();
    // Repeated inputs exercise equal-name stability across chunk boundaries.
    for _ in 0..12 {
        for source in sources {
            files.push(format!("naïve/{source}"));
            expected.extend(
                fs::read_to_string(fixtures.join(format!("{source}.tags")))
                    .unwrap()
                    .lines()
                    .map(str::to_owned),
            );
        }
        files.extend(["script".into(), "binary".into(), "missing.png".into()]);
        expected.push("script_symbol\tscript\t2;\"\tf".into());
    }
    expected.sort_by(|a, b| a.split('\t').next().cmp(&b.split('\t').next()));
    let expected = expected.join("\n") + "\n";
    for threads in [1, 2, 8] {
        assert_eq!(
            success(project.run(&files, threads, &["-f", "-"])),
            expected,
            "{threads} threads"
        );
    }
}

#[test]
fn equal_names_keep_input_and_per_file_order() {
    let project = Project::new();
    let mut files = Vec::new();
    let mut expected = String::new();
    for index in (0..513).rev() {
        let file = format!("source-{index}.rs");
        fs::write(project.0.join(&file), "fn same() {}\n\nfn same() {}\n").unwrap();
        expected.push_str(&format!("same\t{file}\t1;\"\tf\nsame\t{file}\t3;\"\tf\n"));
        files.push(file);
    }
    for threads in [1, 2, 8, 16] {
        assert_eq!(
            success(project.run(&files, threads, &["-f", "-"])),
            expected
        );
    }
}

#[test]
fn reports_first_missing_input_without_replacing_output() {
    let project = Project::new();
    fs::write(project.0.join("source.rs"), "fn valid() {}\n").unwrap();
    let mut files = vec!["source.rs".into(); 257];
    files[70] = "first-missing.rs".into();
    files[130] = "second-missing.rb".into();
    for threads in [1, 2, 8] {
        fs::write(project.0.join("tags"), "preserve existing output\n").unwrap();
        let output = project.run(&files, threads, &[]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!(
                "{} not found\n",
                project.0.join("first-missing.rs").display()
            )
        );
        assert_eq!(
            fs::read_to_string(project.0.join("tags")).unwrap(),
            "preserve existing output\n"
        );
    }
    // Append is deliberately disabled for stdout, preserving the existing CLI behavior.
    assert_eq!(
        project
            .run(&files, 8, &["--append", "-f", "-"])
            .status
            .code(),
        Some(1)
    );
}

#[test]
fn parallel_append_updates_removes_and_preserves_files() {
    let project = Project::new();
    let mut files = Vec::new();
    for index in 0..130 {
        let file = format!("source-{index}.rs");
        fs::write(project.0.join(&file), format!("fn old_{index}() {{}}\n")).unwrap();
        files.push(file);
    }
    fs::write(project.0.join("kept.rs"), "fn kept() {}\n").unwrap();
    let mut original = files.clone();
    original.push("kept.rs".into());
    success(project.run(&original, 8, &[]));
    let mut expected = vec!["kept\tkept.rs\t1;\"\tf".into()];
    for (index, file) in files.iter().enumerate() {
        if index == 73 {
            fs::remove_file(project.0.join(file)).unwrap();
        } else {
            fs::write(project.0.join(file), format!("fn new_{index}() {{}}\n")).unwrap();
            expected.push(format!("new_{index}\t{file}\t1;\"\tf"));
        }
    }
    success(project.run(&files, 8, &["--append"]));
    expected.sort_by(|a, b| a.split('\t').next().cmp(&b.split('\t').next()));
    let actual = fs::read_to_string(project.0.join("tags")).unwrap();
    assert_eq!(
        actual
            .lines()
            .filter(|line| !line.starts_with("!_"))
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn public_tagger_keeps_caller_supplied_configuration() {
    use npezza93_tree_sitter_tags::TagsConfiguration;
    use ttags::{config::Config, tagger::Tagger};

    let project = Project::new();
    let source = project.0.join("source.rs").to_str().unwrap().to_owned();
    fs::write(&source, "fn symbol() {}\n").unwrap();
    let config = Config {
        files: vec![source.clone(); 130],
        tag_path: project.0.join("tags").to_str().unwrap().into(),
        relative_path: project.0.to_str().unwrap().into(),
        append: false,
        lsp: false,
    };
    let mut tagger = Tagger::new(&config);
    assert_eq!(tagger.read_and_parse(&source).len(), 1);
    tagger.rust_config = TagsConfiguration::new(tree_sitter_rust::LANGUAGE.into(), "", "").unwrap();
    assert!(tagger.read_and_parse(&source).is_empty());
    tagger.run(&config.files);
    assert!(
        fs::read_to_string(&config.tag_path)
            .unwrap()
            .lines()
            .all(|line| line.starts_with("!_"))
    );
}
