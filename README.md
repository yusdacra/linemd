[![crates.io](https://img.shields.io/crates/v/linemd)](https://crates.io/crates/linemd)
[![docs.rs](https://docs.rs/linemd/badge.svg)](https://docs.rs/linemd)

# linemd
`linemd` is a simple, no deps, markdown parser and renderer.
## Features
- No deps
- Can render to HTML and SVG; they need `html` and `svg` features enabled respectively
  - By default, `html` feature is enabled
- Comes with a CLI utility for rendering to HTML or SVG

## Install

- Cargo: `cargo install linemd`
- Nix:
  - Flakes: `nix profile install github:yusdacra/linemd`
    - Or run without installing: `nix run install github:yusdacra/linemd`
  - Non-flakes: `nix-env -i -f "https://github.com/yusdacra/linemd/tarball/master"`

## Usage
```rust
let md: String;

let tokens = linemd::md.parse_md();
// use tokens however you want
```

You can also render as HTML (needs `html` feature enabled):
```rust
let parsed_tokens: Vec<Token>;

let html = linemd::render_as_html(parsed_tokens);
```

Or SVG (needs `svg` feature enabled):
```rust
let parsed_tokens: Vec<Token>;

let svg = linemd::render_as_svg(parsed_tokens);
```

CLI usage:
```
renders a markdown file as HTML

usage:
  linemd FILE     Reads from file
  linemd -        Reads from stdin

options:
  -h, --help      Prints this text
  -S, --svg       Renders to SVG instead of HTML

exit codes:
  0               Everything was successful
  1               Failed to read the given file
  2               Failed to read from stdin
```

Also see [examples](examples) directory.
