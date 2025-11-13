//! Technical indicators
//!
//! High-performance SIMD-optimized technical analysis indicators

pub mod simd;

pub use simd::{atr_simd, bollinger_bands_simd, ema_simd, macd_simd, rsi_simd, sma_simd};
