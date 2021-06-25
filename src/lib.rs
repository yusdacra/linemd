#![cfg_attr(tests, no_std)]
//! `linemd` is a simple and opinionated markdown parsing library.

extern crate alloc;
#[cfg(any(feature = "html", feature = "svg"))]
use alloc::{boxed::Box, string::String, vec::Vec};

mod parser;
#[cfg(test)]
mod tests;

#[cfg(feature = "html")]
mod html;
#[cfg(feature = "svg")]
mod svg;

#[doc(inline)]
pub use parser::{Parser, Token};

#[cfg(feature = "svg")]
#[doc(inline)]
pub use svg::{render_as_svg, Config as SvgConfig, ViewportDimensions as SvgViewportDimensions};

#[cfg(feature = "html")]
#[doc(inline)]
pub use html::render_as_html;
