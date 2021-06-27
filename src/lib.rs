#![cfg_attr(tests, no_std)]
//! `linemd` is a simple and opinionated markdown parsing library.

extern crate alloc;
#[cfg(any(feature = "html", feature = "svg"))]
use alloc::string::String;

/// Parser types used to parse markdown.
pub mod parser;
#[cfg(test)]
mod tests;

/// HTML rendering of tokens.
#[cfg(feature = "html")]
pub mod html;
/// SVG rendering of tokens.
#[cfg(feature = "svg")]
pub mod svg;

#[doc(inline)]
pub use parser::Parser;

#[cfg(feature = "svg")]
#[doc(inline)]
pub use svg::{render_as_svg, Config as SvgConfig, ViewportDimensions as SvgViewportDimensions};

#[cfg(feature = "html")]
#[doc(inline)]
pub use html::render_as_html;
