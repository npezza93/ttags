# Language regression fixtures

`tests/languages.rs` checks exact CLI stdout, including names, UTF-8 paths,
1-based lines, tag kinds, sort order and duplicate names. Each of the eight
supported grammars has its own small fixture. C, C++, Haskell, JavaScript, Ruby, Rust
and Swift also exercise Unicode identifiers. The extra Ruby fixture covers
`@metadata` processing for every supported macro, multiple symbol arguments,
association-generated names, and delegates with no prefix, `prefix: true`,
and `prefix: false`.

The expected output was captured and inspected using the unmodified
`a3df6ae` dependency lockfile, with one deliberate correction: its Nix query
had no `@name` capture, so the Nix fixture produced no output. The Nix golden
file requires the intended function and application-binding tags instead of
preserving that bug. An ordinary value binding must still produce no tag.

Other existing behavior is preserved rather than silently normalized:
Haskell signatures and definitions each get a tag; JavaScript constructors
are omitted; Ruby `initialize` is emitted as `new`; Swift functions retain
the `function` kind and its initializer is not emitted. C macros, enum
members and the union declaration are not emitted by the current C tags
query. These fixtures are behavioral checks, not claims of exhaustive
language coverage.
