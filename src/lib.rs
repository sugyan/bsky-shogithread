mod bot;
mod error;
mod shogi;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use crate::error::{Error, Result};
pub use bot::Bot;
