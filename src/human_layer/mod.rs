//! Python interface for rapid strategy development
//!
//! This module exposes Rust's high-performance core to Python developers
//! via PyO3 bindings, allowing strategy development in Python while keeping
//! execution speed in Rust.

#[cfg(feature = "python")]
mod python_bindings;

#[cfg(feature = "python")]
pub use python_bindings::*;
