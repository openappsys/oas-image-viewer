//! macOS 平台系统集成实现
//!
//! 通过 duti 工具或 CoreServices API 设置默认程序
//! 右键菜单通过 Info.plist 配置自动支持"打开方式"

use super::SystemIntegration;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// 支持的图片文件 UTI 列表（Uniform Type Identifier）
const IMAGE_UTIS: &[&str] = &[
    "public.png",         // PNG 图片
    "public.jpeg",        // JPEG 图片
    "public.tiff",        // TIFF 图片
    "com.compuserve.gif", // GIF 图片
    "public.bmp",         // BMP 图片
    "public.webp",        // WebP 图片
    "public.image",       // 通用图片类型
];

/// macOS 系统集成实现
pub struct MacOSIntegration;

impl MacOSIntegration {
    /// 创建新的 macOS 集成实例
    pub fn new() -> Self {
        Self
    }

    /// 获取当前可执行文件的 Bundle Identifier
    /// 尝试从 Info.plist 中读取，如果失败则使用默认值
    fn get_bundle_id(&self) -> Result<String> {
        // 首先尝试从当前可执行文件路径获取 bundle ID
        if let Ok(exe_path) = self.get_exe_path() {
            // 如果应用被打包成 .app 格式，尝试读取 Info.plist
            if let Some(bundle_id) = self.read_bundle_id_from_plist(&exe_path) {
                return Ok(bundle_id);
            }
        }

        // 默认使用开发时的 bundle ID
        // 实际打包后应该能从 Info.plist 中读取到正确的值
        Ok("com.openappsys.oas-image-viewer".to_string())
    }

    /// 从可执行文件路径尝试读取 bundle ID
    /// 在 macOS 中，应用通常位于 MyApp.app/Contents/MacOS/executable
    fn read_bundle_id_from_plist(&self, exe_path: &PathBuf) -> Option<String> {
        // 尝试找到 .app 包目录
        let mut current = exe_path.as_path();

        // 向上遍历查找 .app 目录
        while let Some(parent) = current.parent() {
            if let Some(name) = parent.file_name()?.to_str() {
                if name.ends_with(".app") {
                    // 找到 .app 包，读取 Info.plist
                    let plist_path = parent.join("Contents/Info.plist");
                    if plist_path.exists() {
                        return self.parse_bundle_id_from_plist(&plist_path);
                    }
                }
            }
            current = parent;
        }

        None
    }

    /// 解析 plist 文件获取 Bundle Identifier
    fn parse_bundle_id_from_plist(&self, plist_path: &PathBuf) -> Option<String> {
        // 使用 plutil 命令将 plist 转换为 JSON 格式
        let output = Command::new("plutil")
            .args(&["-convert", "json", "-o", "-", plist_path.to_str()?])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_str = String::from_utf8(output.stdout).ok()?;
        let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

        // 获取 CFBundleIdentifier
        json.get("CFBundleIdentifier")?
            .as_str()
            .map(|s| s.to_string())
    }

    /// 获取当前可执行文件路径
    fn get_exe_path(&self) -> Result<PathBuf> {
        std::env::current_exe().context("无法获取可执行文件路径")
    }

    /// 检查 duti 工具是否可用
    fn check_duti_available(&self) -> bool {
        Command::new("which")
            .arg("duti")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 使用 duti 设置默认程序
    fn set_default_with_duti(&self, uti: &str, bundle_id: &str) -> Result<()> {
        let output = Command::new("duti")
            .args(&["-s", bundle_id, uti, "all"])
            .output()
            .context("运行 duti 命令失败")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("duti 命令执行失败: {}", stderr);
        }

        Ok(())
    }

    /// 获取指定 UTI 的默认程序 bundle ID
    fn get_default_handler_for_uti(&self, uti: &str) -> Result<String> {
        // 使用 Launch Services API 的 Python 脚本来获取默认程序
        let python_script = format!(
            r#"
import sys
try:
    from LaunchServices import LSCopyDefaultRoleHandlerForContentType
    result = LSCopyDefaultRoleHandlerForContentType("{}", 0xFFFFFFFF)
    if result:
        print(result)
        sys.exit(0)
except Exception as e:
    pass

# 回退到 mdls 命令
import subprocess
try:
    result = subprocess.run(["mdls", "-name", "kMDItemContentType", "-raw", "-nullMarker", "", "/System/Library/CoreServices/Finder.app"], 
                          capture_output=True, text=True)
except:
    pass

sys.exit(1)
"#,
            uti
        );

        let output = Command::new("python3")
            .args(&["-c", &python_script])
            .output()
            .or_else(|_| {
                Command::new("python")
                    .args(&["-c", &python_script])
                    .output()
            })
            .context("无法运行 Python 查询默认程序")?;

        if output.status.success() {
            let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !bundle_id.is_empty() {
                return Ok(bundle_id);
            }
        }

        // 如果 Python 方法失败，尝试使用 duti 查询
        if self.check_duti_available() {
            let output = Command::new("duti")
                .args(&["-q", uti])
                .output()
                .context("运行 duti 查询命令失败")?;

            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                // duti 输出格式: "handler    bundle_id"
                let parts: Vec<&str> = result.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }

        bail!("无法获取 UTI {} 的默认程序", uti)
    }

    /// 重置指定 UTI 的默认程序为系统默认（预览应用）
    fn reset_default_to_preview(&self, uti: &str) -> Result<()> {
        // macOS 预览应用的 bundle ID
        let preview_bundle_id = "com.apple.Preview";

        if self.check_duti_available() {
            self.set_default_with_duti(uti, preview_bundle_id)
        } else {
            // 如果没有 duti，使用 Python 脚本
            self.set_default_with_python(uti, preview_bundle_id)
        }
    }

    /// 使用 Python 脚本设置默认程序
    fn set_default_with_python(&self, uti: &str, bundle_id: &str) -> Result<()> {
        let python_script = format!(
            r#"
import sys
try:
    from LaunchServices import LSSetDefaultRoleHandlerForContentType
    import objc
    
    result = LSSetDefaultRoleHandlerForContentType(
        "{}",
        0xFFFFFFFF,
        "{}"
    )
    if result == 0:
        sys.exit(0)
    else:
        print(f"设置默认程序失败，错误码: {{result}}")
        sys.exit(1)
except ImportError as e:
    print(f"缺少必要的模块: {{e}}")
    sys.exit(1)
except Exception as e:
    print(f"错误: {{e}}")
    sys.exit(1)
"#,
            uti, bundle_id
        );

        let output = Command::new("python3")
            .args(&["-c", &python_script])
            .output()
            .or_else(|_| {
                Command::new("python")
                    .args(&["-c", &python_script])
                    .output()
            })
            .context("无法运行 Python 设置默认程序")?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("设置默认程序失败: {}", stderr)
        }
    }
}

impl Default for MacOSIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemIntegration for MacOSIntegration {
    /// 设置为默认图片查看器
    ///
    /// 使用 duti 工具或 CoreServices Python API 将本应用设置为
    /// 所有支持的图片格式的默认打开程序
    fn set_as_default(&self) -> Result<()> {
        let bundle_id = self.get_bundle_id()?;

        // 检查是否已安装 duti
        let use_duti = self.check_duti_available();

        // 为每种图片类型设置默认程序
        for uti in IMAGE_UTIS {
            if use_duti {
                self.set_default_with_duti(uti, &bundle_id)
                    .with_context(|| format!("无法设置 {} 的默认程序", uti))?;
            } else {
                self.set_default_with_python(uti, &bundle_id)
                    .with_context(|| format!("无法设置 {} 的默认程序", uti))?;
            }
        }

        // 通知 Finder 刷新（可选）
        let _ = Command::new("killall")
            .arg("-u")
            .arg("$USER")
            .arg("Finder")
            .output();

        Ok(())
    }

    /// 添加到右键菜单
    ///
    /// 在 macOS 上，右键菜单主要通过 Info.plist 中的 CFBundleDocumentTypes
    /// 声明自动支持"打开方式"菜单。此方法返回成功，因为功能已通过
    /// Info.plist 配置实现。
    fn add_context_menu(&self) -> Result<()> {
        // macOS 的"打开方式"菜单是系统级的，通过 Info.plist 配置
        // 只要应用声明了支持的文件类型，系统会自动显示在"打开方式"中
        //
        // 如果需要在服务菜单中添加，需要额外创建 Service Provider
        // 这需要更复杂的实现，当前版本暂不提供

        // 检查 Info.plist 是否正确配置
        if let Ok(exe_path) = self.get_exe_path() {
            if self.read_bundle_id_from_plist(&exe_path).is_none() {
                bail!("未找到 Info.plist，请确保应用已正确打包为 .app 格式\n\
                       通过 Info.plist 声明的文件类型会自动出现在系统的「打开方式」菜单中");
            }
        }

        Ok(())
    }

    /// 从右键菜单移除
    ///
    /// 将默认程序重置为系统预览应用，从而"移除"本应用作为默认
    fn remove_context_menu(&self) -> Result<()> {
        // 将所有图片类型的默认程序重置为 macOS 预览应用
        for uti in IMAGE_UTIS {
            self.reset_default_to_preview(uti)
                .with_context(|| format!("无法重置 {} 的默认程序", uti))?;
        }

        // 通知 Finder 刷新
        let _ = Command::new("killall")
            .arg("-u")
            .arg("$USER")
            .arg("Finder")
            .output();

        Ok(())
    }

    /// 检查是否已是默认查看器
    ///
    /// 检查第一个图片类型 (PNG) 的默认程序是否为本应用
    fn is_default(&self) -> bool {
        // 使用第一个图片类型进行检查
        let first_uti = IMAGE_UTIS[0];

        match self.get_default_handler_for_uti(first_uti) {
            Ok(current_bundle) => {
                if let Ok(our_bundle) = self.get_bundle_id() {
                    current_bundle == our_bundle
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
}
