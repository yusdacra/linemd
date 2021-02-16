# linemd
`linemd` is a simple and opinionated markdown crate.

## Features
- No deps
- Can render to HTML (includes CLI utility to render to HTML)

## Install
Nix:
- Flakes: `nix profile install github:yusdacra/linemd`

## Usage
```rust
let md: String;

let tokens = linemd::parse(md);
// use tokens however you want
```

You can also render as HTML:
```rust
let parsed_tokens: Vec<Token>;

let html = linemd::render_as_html(parsed_tokens);
```

CLI usage:
```
renders a markdown file as HTML

usage:
  linemd FILE     Reads from file
  linemd -        Reads from stdin

options:
  -h, --help      Prints this text

exit codes:
  0               Everything was successful
  1               Failed to read the given file
  2               Failed to read from stdin
```

Also see [examples](examples) directory.