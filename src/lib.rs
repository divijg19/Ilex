pub mod app;
pub mod cli;
pub mod config;
pub mod contracts;
pub mod detectors;
pub mod modules;
pub mod render;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
