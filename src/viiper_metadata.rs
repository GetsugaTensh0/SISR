//! Baked-in VIIPER integration metadata.
//!
//! The actual constants are generated at build time from `Cargo.toml`:
//! `[package.metadata.viiper]`.

include!(concat!(env!("OUT_DIR"), "/viiper_metadata.rs"));
