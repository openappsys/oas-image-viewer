# Windows 右键菜单注册说明

## 快速开始（推荐）

### 方法一：使用自动检测脚本（推荐）

1. 将 `register-context-menu.bat` 复制到 Image-Viewer 安装目录
2. 双击运行 `register-context-menu.bat`
3. 脚本会自动检测当前目录并注册右键菜单

### 方法二：使用 PowerShell（Windows 10/11）

```powershell
# 以管理员身份运行 PowerShell
# 导航到 Image-Viewer 目录
cd C:\Path\To\Image-Viewer

# 运行注册脚本
.\register-context-menu.bat
```

## 支持的图片格式

- PNG, JPG, JPEG, GIF, WebP
- TIFF, TIF, BMP, ICO
- HEIC, HEIF, AVIF

## 文件说明

| 文件 | 说明 |
|------|------|
| `register-context-menu.bat` | 自动检测安装路径并注册右键菜单（推荐） |
| `register-context-menu-win10-win11.reg` | 静态注册表文件（需要手动编辑路径） |
| `unregister-context-menu.reg` | 卸载右键菜单 |

## 绿色版/便携版使用说明

对于绿色版（解压即用）：
1. 解压到任意目录
2. 双击 `register-context-menu.bat`
3. 完成！无需固定路径

## MSI 安装版

MSI 安装程序会自动检测安装路径并注册右键菜单，无需手动操作。

## 卸载右键菜单

双击运行 `unregister-context-menu.reg` 文件即可移除所有右键菜单项。

## 故障排除

### 提示 "image-viewer.exe not found"
确保 `register-context-menu.bat` 与 `image-viewer.exe` 在同一文件夹中。

### 注册失败
尝试以管理员身份运行批处理脚本。

### 右键菜单未显示
1. 重启资源管理器：任务管理器 -> 重启 Windows 资源管理器
2. 或注销后重新登录
