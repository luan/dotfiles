#![allow(dead_code)]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod chooser;
pub mod color;
pub mod filter;
pub mod group;
pub mod logging;
pub mod order;
pub mod palette;
pub mod picker;
pub mod process;
pub mod project;
pub mod sidebar;
pub mod status;
pub mod tmux;
pub mod usage_bars;
