//! 不支持的平台的空实现

use super::IntegrationStatus;
use anyhow::Result;

/// 获取系统集成状态（空实现）
pub fn get_integration_status() -> IntegrationStatus {
    IntegrationStatus::default()
}

/// 注册右键菜单（空实现）
pub fn register_context_menu() -> Result<()> {
    Err(anyhow::anyhow!(
        "Context menu registration is not supported on this platform"
    ))
}

/// 注销右键菜单（空实现）
pub fn unregister_context_menu() -> Result<()> {
    Err(anyhow::anyhow!(
        "Context menu unregistration is not supported on this platform"
    ))
}

/// 设为默认图片查看器（空实现）
pub fn set_as_default_app() -> Result<()> {
    Err(anyhow::anyhow!(
        "Setting default app is not supported on this platform"
    ))
}
