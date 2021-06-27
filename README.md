[![crates.io](https://img.shields.io/crates/v/linemd)](https://crates.io/crates/linemd)
[![docs.rs](https://docs.rs/linemd/badge.svg)](https://docs.rs/linemd)

# linemd

`linemd` is a simple, no dependencies, markdown parser and renderer.

## Features

- No dependencies.
- Does not depend on `std`, only depends on `alloc` and `core`.
- No allocations while parsing; only allocation is done to store the tokens (unless you provide your own preallocated `Vec`).
- Can render to HTML and SVG; they need `html` and `svg` features enabled respectively.
  - By default, `html` feature is enabled.
- Comes with a CLI utility for rendering to HTML or SVG.

## Install

- Cargo: `cargo install linemd`
- Nix:
  - Flakes: `nix profile install github:yusdacra/linemd`
    - Or run without installing: `nix run install github:yusdacra/linemd`
  - Non-flakes: `nix-env -i -f "https://github.com/yusdacra/linemd/tarball/master"`

## Usage

See the [library documentation](https://docs.rs/linemd) for library usage.

CLI usage:
```
renders a markdown file

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
