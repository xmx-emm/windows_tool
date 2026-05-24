//! 各游戏在 Steam / EA Desktop 上的便捷 API。
//!
//! - **`steam`**：需 `feature = "steam"`，随 `game` 模块一同编译（见 crate 根 `lib.rs` 条件）。
//! - **`apex` / `ea` / `pubg`**：需 `feature = "game"`。

pub mod steam;

#[cfg(feature = "game")]
pub mod apex;

#[cfg(feature = "game")]
pub mod ea;

#[cfg(feature = "game")]
pub mod pubg;
