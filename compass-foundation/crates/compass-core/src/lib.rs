#[cfg(not(target_os = "linux"))]
compile_error!("Compass Foundation 0.1 supports Linux only; see the filesystem contract.");

pub mod acquisition;
pub mod codec;
pub mod collection;
pub mod compare;
pub mod d0;
pub mod diffusion;
mod error;
pub mod model;
pub mod observe;
pub mod read;
pub mod retrieval;
pub mod search;
pub mod semantic;
pub mod spectral_acquisition;
pub mod storage;
pub mod structure;

pub use error::{Error, Result};

pub const BUILD_ID: &str = env!("COMPASS_BUILD_ID");
