use crate::core::domain::Language;

/// 跨平台系统集成 trait
pub trait SystemIntegration {
    /// 设置为默认图片查看器
    fn set_as_default(&self, language: Language) -> anyhow::Result<()>;
    /// 添加到右键菜单
    fn add_context_menu(&self, language: Language) -> anyhow::Result<()>;
    /// 从右键菜单移除
    fn remove_context_menu(&self, language: Language) -> anyhow::Result<()>;
    /// 检查是否已是默认查看器
    fn is_default(&self) -> bool;
}

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux::LinuxIntegration as PlatformIntegration;
#[cfg(target_os = "macos")]
pub use macos::MacOSIntegration as PlatformIntegration;
#[cfg(target_os = "windows")]
pub use windows::WindowsIntegration as PlatformIntegration;
