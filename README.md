# ttags

ttags generates ctags using [Tree-sitter](https://github.com/tree-sitter/tree-sitter).

### Installation

##### macOS and Linux

```bash
cargo install ttags
```

Binaries are also available on the releases page of the repo.
Download the tar file, and place the executable in your path.

##### From source

Requires Rust 1.90 or newer (Tree-sitter's minimum supported version).

```bash
git clone https://github.com/npezza93/ttags
cd ttags
cargo build --release
git ls-files -z | ./target/release/ttags --null --files-from -
```

### Usage

Give a list of file paths and/or directories to ttags to parse and generate.

```bash
git ls-files -z | ttags --null --files-from -
```

The file list is read in a single invocation, avoiding the shell's argument-size
limit. NUL separators preserve spaces, quotes, and newlines in input paths.
File paths must be UTF-8. Tabs and newlines in generated tag paths are not
escaped by the tags writer.
Unsupported file extensions are skipped without reading their contents;
extensionless files are still checked for a Ruby shebang.

#### Options

- `-L` or `--files-from=FILE` - Read one path per line from a file, or from stdin
  when `FILE` is `-`. May be combined with positional file paths and directories.

- `-0` or `--null` - Use NUL-separated paths with `--files-from`, as produced by
  `git ls-files -z`.


- `-a` or `--apend` - Will keep your tag file in tact and only update the tags
  for the files passed.

- `-r` or `--tag-relative` - Make paths outputed in the tags file be relative to the
  current working directory (i.e. where you called `ttags`)? By default, the tag
  path is relative to the `tag-file` location.

- `-f` or `--tag-file=file` - Path to the file where tags should be written. If
  `-` is passed, tags are outputted to stdout.

- `lsp` - Starts an LSP server that updates the tags for a file when the file is
  saved

### Supported languages

- [x] C
- [x] C++
- [x] D (`.d` and `.di`, including alias declarations)
- [x] Haskell
- [x] JavaScript
- [x] Nix
- [x] Ruby
- [x] Rust
- [x] Swift
