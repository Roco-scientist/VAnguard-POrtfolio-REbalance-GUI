#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::VaporeApp;
pub mod asset;
pub mod calc;
pub mod holdings;
#[macro_use]
extern crate lazy_static;
