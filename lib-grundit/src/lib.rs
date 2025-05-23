// module declarations
#[cfg(feature = "full")]
pub mod app;
#[cfg(feature = "full")]
pub mod auth;
#[cfg(feature = "full")]
pub mod config;
#[cfg(feature = "full")]
pub mod error;
pub mod types;

#[cfg(feature = "full")]
pub use app::{AuthrState, run};
