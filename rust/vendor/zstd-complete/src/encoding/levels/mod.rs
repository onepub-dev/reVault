#[allow(dead_code)]
pub(crate) mod c_port;
mod fastest;
#[cfg(test)]
mod fastest_tests;
pub(crate) use fastest::compress_at_level_without_incompressible_probe;
pub use fastest::{compress_at_level, compress_fastest};
