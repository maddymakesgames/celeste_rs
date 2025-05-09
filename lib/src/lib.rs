#![allow(
    // I always either have checks or comments where I unwrap to ensure safety
    // and tbh I don't think unwrap is unidiomatic
    clippy::unnecessary_unwrap
)]

//! Celeste save reader and writer
pub mod maps;
pub mod mods;
pub mod playbacks;
pub mod saves;
pub mod utils;
