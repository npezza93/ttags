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
```bash
git clone https://github.com/npezza93/ttags
cd ttags
cargo build --release
./target/release/ttags $(git ls-files)
```

### Usage

Give a list of file paths and/or directories to ttags to parse and generate.

```bash
ttags $(git ls-files)
```

#### Options

- `-a` or `--apend` - Will keep your tag file in tact and only update the tags
  for the files passed.

- `-r` or `--tag-relative` - Make paths outputed in the tags file be relative to the
  current working directory (i.e. where you called `ttags`)? By default, the tag
  path is relative to the `tag-file` location.

- `-f` or `--tag-file=file` - Path to the file where tags should be written. If
  `-` is passed, tags are outputted to stdout.
