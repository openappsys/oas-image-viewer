//! 平台特定的系统集成实现

use super::IntegrationStatus;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

// 非上述平台的空实现
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod fallback;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use fallback::*;
#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
pub use windows::*;
