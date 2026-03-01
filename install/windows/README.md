# Windows 安装与右键菜单注册

## 快速开始

### 方法一：完整安装（推荐）

运行 install.bat：
- 自动检测绿色版或标准版
- 注册右键菜单（无需管理员权限）

### 方法二：仅注册右键菜单（绿色版/便携版）

运行 register-context-menu.bat

脚本会自动检测 image-viewer.exe 的位置。

## 支持的 Windows 版本

- Windows 7/8/8.1/10/11 全部支持
- 使用 HKEY_CURRENT_USER，不需要管理员权限

## 支持的图片格式

PNG, JPG, JPEG, GIF, WebP, TIFF, TIF, BMP, ICO, HEIC, HEIF, AVIF

## 文件说明

- install.bat - 完整安装程序
- register-context-menu.bat - 仅注册右键菜单
- unregister-context-menu.reg - 卸载右键菜单

## 卸载右键菜单

双击运行 unregister-context-menu.reg 即可。
