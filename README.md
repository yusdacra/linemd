# linemd
`linemd` is a simple and opinionated markdown crate.

## Features
- No deps
- Can render to HTML

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

Also see [examples](examples) directory.