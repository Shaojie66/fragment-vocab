pub mod backup;
pub mod config;
pub mod pet;
pub mod review;
mod types;
mod utils;
pub mod wordbook;
pub mod wrong_book;

// Re-export types for lib.rs
pub use types::*;

// Re-export non-command functions used directly in lib.rs
pub(crate) use config::{get_today_stats, load_app_config, pause_scheduler};
pub(crate) use pet::{init_pet_on_startup, show_pet_window};
