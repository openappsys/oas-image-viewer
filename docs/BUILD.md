# Image-Viewer 打包指南

本文档详细说明如何在 Windows、macOS、Linux 三个平台分别打包生成发布包。

## 📋 目录

- [1. 本地打包（手动）](#1-本地打包手动)
  - [Windows (x86_64)](#windows-x86_64)
  - [macOS (x86_64 + Apple Silicon)](#macos-x86_64--apple-silicon)
  - [Linux (x86_64 + aarch64)](#linux-x86_64--aarch64)
- [2. CI/CD 自动打包（GitHub Actions）](#2-cicd-自动打包github-actions)
- [3. 交叉编译打包](#3-交叉编译打包)
- [4. 打包产物说明](#4-打包产物说明)
- [5. 常见问题排查](#5-常见问题排查)

---

## 1. 本地打包（手动）

### Windows (x86_64)

#### 环境要求

- Windows 10/11 64位
- [Rust](https://rustup.rs/) 1.93 或更高版本
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) (包含 C++ 构建工具)

#### 依赖安装

1. **安装 Rust**

   ```powershell
   # 使用 rustup 安装
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # 或下载并运行 rustup-init.exe
   # https://rustup.rs/
   ```

2. **安装 Visual Studio Build Tools**

   下载并安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)，选择以下工作负载：
   - **使用 C++ 的桌面开发**

3. **安装 cargo-wix（用于生成 MSI 安装包）**

   ```powershell
   cargo install cargo-wix
   ```

#### 打包命令

```powershell
# 克隆仓库
git clone https://github.com/yourusername/image-viewer.git
cd image-viewer

# 构建 Release 版本
cargo build --release

# 验证构建结果
ls target\release\image-viewer.exe

# 打包为 ZIP
# 创建发布目录
mkdir -p dist\image-viewer-v0.1.0-windows-x86_64
copy target\release\image-viewer.exe dist\image-viewer-v0.1.0-windows-x86_64\
copy README.md dist\image-viewer-v0.1.0-windows-x86_64\
copy LICENSE dist\image-viewer-v0.1.0-windows-x86_64\

# 压缩为 ZIP
cd dist
7z a ..\image-viewer-v0.1.0-windows-x86_64.zip image-viewer-v0.1.0-windows-x86_64\

# 生成 MSI 安装包（可选）
cargo wix --no-build --output target/wix/image-viewer.msi
```

#### 输出位置

- 可执行文件：`target\release\image-viewer.exe`
- ZIP 包：`image-viewer-v0.1.0-windows-x86_64.zip`
- MSI 安装包：`target\wix\image-viewer.msi`

---

### macOS (x86_64 + Apple Silicon)

#### 环境要求

- macOS 11.0 (Big Sur) 或更高版本
- [Rust](https://rustup.rs/) 1.93 或更高版本
- [Homebrew](https://brew.sh/)

#### 依赖安装

1. **安装 Rust**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **安装 create-dmg（用于生成 DMG 安装包）**

   ```bash
   brew install create-dmg
   ```

3. **安装交叉编译目标（如需构建另一种架构）**

   ```bash
   # 在 Intel Mac 上构建 Apple Silicon 版本
   rustup target add aarch64-apple-darwin
   
   # 在 Apple Silicon Mac 上构建 Intel 版本
   rustup target add x86_64-apple-darwin
   ```

#### 打包命令

**本地架构构建（自动检测）：**

```bash
# 克隆仓库
git clone https://github.com/yourusername/image-viewer.git
cd image-viewer

# 构建 Release 版本
cargo build --release

# 验证构建结果
ls -la target/release/image-viewer
file target/release/image-viewer
```

**特定架构构建：**

```bash
# Intel Mac (x86_64)
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Apple Silicon (M1/M2/M3)
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

**创建发布包：**

```bash
VERSION="0.1.0"
ARCH="$(uname -m)"  # x86_64 或 arm64

# 创建目录结构
mkdir -p dist/image-viewer-v${VERSION}-macos-${ARCH}
cp target/release/image-viewer dist/image-viewer-v${VERSION}-macos-${ARCH}/
cp README.md dist/image-viewer-v${VERSION}-macos-${ARCH}/
cp LICENSE dist/image-viewer-v${VERSION}-macos-${ARCH}/

# 打包为 tar.gz
cd dist
tar czvf ../image-viewer-v${VERSION}-macos-${ARCH}.tar.gz image-viewer-v${VERSION}-macos-${ARCH}/
```

**创建 DMG 安装包：**

```bash
VERSION="0.1.0"
ARCH="$(uname -m)"

mkdir -p dist/dmg
cp target/release/image-viewer dist/dmg/
cp README.md dist/dmg/
cp LICENSE dist/dmg/

create-dmg \
  --volname "Image Viewer ${VERSION}" \
  --window-size 800 400 \
  --icon-size 100 \
  "image-viewer-v${VERSION}-macos-${ARCH}.dmg" \
  dist/dmg/
```

#### 输出位置

- 可执行文件：`target/release/image-viewer`
- tar.gz 包：`image-viewer-v0.1.0-macos-x86_64.tar.gz` 或 `image-viewer-v0.1.0-macos-aarch64.tar.gz`
- DMG 安装包：`image-viewer-v0.1.0-macos-{arch}.dmg`

---

### Linux (x86_64 + aarch64)

#### 环境要求

- Ubuntu 20.04+ / Debian 11+ / Fedora 35+ / Arch Linux
- [Rust](https://rustup.rs/) 1.93 或更高版本

#### 依赖安装

**Ubuntu/Debian：**

```bash
# 系统依赖
sudo apt-get update
sudo apt-get install -y \
  libgtk-3-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libssl-dev \
  pkg-config \
  cmake

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**Fedora：**

```bash
# 系统依赖
sudo dnf install -y \
  gtk3-devel \
  libxcb-devel \
  openssl-devel \
  pkgconfig \
  cmake

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**Arch Linux：**

```bash
# 系统依赖
sudo pacman -S --needed \
  gtk3 \
  libxcb \
  openssl \
  pkgconf \
  cmake

# 安装 Rust (通过 pacman 或 rustup)
sudo pacman -S rust  # 或 curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**ARM64/aarch64 额外步骤：**

```bash
# 添加 ARM64 编译目标
rustup target add aarch64-unknown-linux-gnu

# 安装交叉编译工具链（如在 x86_64 上构建 ARM64）
sudo apt-get install -y gcc-aarch64-linux-gnu
```

#### 打包命令

**x86_64 本地构建：**

```bash
# 克隆仓库
git clone https://github.com/yourusername/image-viewer.git
cd image-viewer

# 构建 Release 版本
cargo build --release

# 验证构建结果
ls -la target/release/image-viewer
file target/release/image-viewer
```

**aarch64 构建：**

```bash
# 添加目标
rustup target add aarch64-unknown-linux-gnu

# 如果在 ARM64 设备上本地构建
cargo build --release --target aarch64-unknown-linux-gnu

# 如果在 x86_64 上交叉编译（需要安装 cross 工具）
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

**创建发布包：**

```bash
VERSION="0.1.0"
ARCH=$(uname -m)  # x86_64 或 aarch64

# 确定目标目录
if [ "$ARCH" = "x86_64" ]; then
    TARGET="x86_64-unknown-linux-gnu"
else
    TARGET="aarch64-unknown-linux-gnu"
fi

# 创建目录结构
mkdir -p dist/image-viewer-v${VERSION}-linux-${ARCH}
cp target/${TARGET}/release/image-viewer dist/image-viewer-v${VERSION}-linux-${ARCH}/
cp README.md dist/image-viewer-v${VERSION}-linux-${ARCH}/
cp LICENSE dist/image-viewer-v${VERSION}-linux-${ARCH}/

# 打包为 tar.gz
cd dist
tar czvf ../image-viewer-v${VERSION}-linux-${ARCH}.tar.gz image-viewer-v${VERSION}-linux-${ARCH}/
```

#### 输出位置

- 可执行文件：`target/x86_64-unknown-linux-gnu/release/image-viewer`
- tar.gz 包：`image-viewer-v0.1.0-linux-x86_64.tar.gz` 或 `image-viewer-v0.1.0-linux-aarch64.tar.gz`

---

## 2. CI/CD 自动打包（GitHub Actions）

项目已配置 `.github/workflows/release.yml`，支持自动多平台打包。

### 触发 Release Workflow

#### 方式一：推送标签（推荐）

```bash
# 创建并推送标签
git tag v0.1.0
git push origin v0.1.0
```

标签格式要求：
- `v[0-9]+.[0-9]+.[0-9]+*` 例如：`v0.1.0`, `v1.2.3-beta`
- 符合语义化版本规范

#### 方式二：手动触发（如配置了 workflow_dispatch）

```bash
# 通过 GitHub CLI 触发
gh workflow run release.yml -f version=v0.1.0

# 或在 GitHub 仓库页面手动触发
# Actions → Release → Run workflow
```

### 需要配置的 Secrets

该 Workflow 使用默认的 `GITHUB_TOKEN`，无需额外配置 secrets。

如需自定义，可配置以下可选 secrets：

| Secret | 用途 | 必需 |
|--------|------|------|
| `GITHUB_TOKEN` | 自动提供，用于创建 Release | 否（自动） |

### 打包流程说明

```
推送标签 v0.1.0
    ↓
触发 Release Workflow
    ↓
┌─────────────────────────────────────────────────────────────┐
│  1. Generate Changelog                                      │
│     - 提取版本号                                            │
│     - 生成变更日志                                          │
└─────────────────────────────────────────────────────────────┘
    ↓
并行执行构建任务
    ↓
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ Linux x86_64    │ │ Linux aarch64   │ │ Windows x86_64  │
│ (ubuntu-latest) │ │ (cross compile) │ │ (windows-latest)│
└─────────────────┘ └─────────────────┘ └─────────────────┘
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ macOS x86_64    │ │ macOS aarch64   │ │ Windows MSI     │
│ (macos-latest)  │ │ (macos-latest)  │ │ (windows-latest)│
└─────────────────┘ └─────────────────┘ └─────────────────┘
    ↓
上传 Artifact
    ↓
创建 GitHub Release
    ↓
发布完成
```

### 产物下载位置

打包完成后，可在以下位置下载：

1. **GitHub Release 页面**
   - 地址：`https://github.com/yourusername/image-viewer/releases`
   - 包含所有平台的预编译包

2. **Artifacts（构建期间）**
   - Actions → 具体 Workflow Run → Artifacts
   - 包含每个平台的独立 artifact

---

## 3. 交叉编译打包

### 使用 cross 工具

[cross](https://github.com/cross-rs/cross) 使用 Docker 容器进行交叉编译，无需手动配置工具链。

#### 安装 cross

```bash
cargo install cross

# 或使用 cargo-binstall
cargo install cargo-binstall
cargo binstall cross
```

#### 在 Linux 上交叉编译 Windows 包

```bash
# 添加 Windows 目标
rustup target add x86_64-pc-windows-gnu

# 使用 cross 构建
cross build --release --target x86_64-pc-windows-gnu

# 注意：Windows MSVC 目标不支持从 Linux 交叉编译
# x86_64-pc-windows-msvc 需要 Windows 环境
```

#### 在 Linux 上交叉编译 macOS 包

⚠️ **注意**：由于 macOS 是闭源系统，cross 不支持直接从 Linux 交叉编译 macOS 目标。

替代方案：
1. 使用 GitHub Actions 自动构建
2. 在 macOS 虚拟机中构建
3. 使用 [osxcross](https://github.com/tpoechtrager/osxcross)（需要 Apple SDK，存在许可限制）

#### 在 x86_64 Linux 上构建 aarch64 Linux 包

```bash
# 添加 ARM64 目标
rustup target add aarch64-unknown-linux-gnu

# 使用 cross 构建
cross build --release --target aarch64-unknown-linux-gnu

# 验证结果
file target/aarch64-unknown-linux-gnu/release/image-viewer
# 输出应显示：ELF 64-bit LSB executable, ARM aarch64
```

#### Cross 配置

创建 `Cross.toml` 自定义构建配置：

```toml
# Cross.toml
[target.aarch64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main"

[target.x86_64-pc-windows-gnu]
image = "ghcr.io/cross-rs/x86_64-pc-windows-gnu:main"

[target.aarch64-unknown-linux-gnu.env]
passthrough = ["RUST_BACKTRACE"]
```

---

## 4. 打包产物说明

### 支持的包格式

| 平台 | 格式 | 用途 | 文件名示例 |
|------|------|------|-----------|
| Linux x86_64 | tar.gz | 便携包 | `image-viewer-v0.1.0-linux-x86_64.tar.gz` |
| Linux aarch64 | tar.gz | 便携包 | `image-viewer-v0.1.0-linux-aarch64.tar.gz` |
| Windows x86_64 | zip | 便携包 | `image-viewer-v0.1.0-windows-x86_64.zip` |
| Windows x86_64 | msi | 安装包 | `image-viewer.msi` |
| macOS x86_64 | tar.gz | 便携包 | `image-viewer-v0.1.0-macos-x86_64.tar.gz` |
| macOS aarch64 | tar.gz | 便携包 | `image-viewer-v0.1.0-macos-aarch64.tar.gz` |
| macOS x86_64 | dmg | 安装包 | `image-viewer-v0.1.0-macos-x86_64.dmg` |
| macOS aarch64 | dmg | 安装包 | `image-viewer-v0.1.0-macos-aarch64.dmg` |

### 如何验证打包结果

#### 1. 验证可执行文件架构

**Linux/macOS：**

```bash
file image-viewer
# x86_64 Linux 输出：
# ELF 64-bit LSB executable, x86-64, version 1 (SYSV)

# aarch64 Linux 输出：
# ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV)

# macOS x86_64 输出：
# Mach-O 64-bit executable x86_64

# macOS ARM64 输出：
# Mach-O 64-bit executable arm64
```

**Windows：**

```powershell
# 使用 dumpbin（Visual Studio 工具）
dumpbin /headers image-viewer.exe | findstr machine

# 或使用 PowerShell 检查文件属性
[System.Reflection.AssemblyName]::GetAssemblyName("image-viewer.exe")
```

#### 2. 验证压缩包完整性

```bash
# 测试 tar.gz
tar tzf image-viewer-v0.1.0-linux-x86_64.tar.gz

# 测试 zip
unzip -t image-viewer-v0.1.0-windows-x86_64.zip
```

#### 3. 运行测试

```bash
# 解压并运行

# Linux/macOS
tar xzf image-viewer-v0.1.0-linux-x86_64.tar.gz
cd image-viewer-v0.1.0-linux-x86_64
./image-viewer --version
./image-viewer --help

# Windows
Expand-Archive -Path image-viewer-v0.1.0-windows-x86_64.zip -DestinationPath .
cd image-viewer-v0.1.0-windows-x86_64
.\image-viewer.exe --version
```

#### 4. 检查文件大小

```bash
ls -lh image-viewer-v*.{tar.gz,zip,dmg,msi}
```

---

## 5. 常见问题排查

### 5.1 构建失败

#### 问题：`linker cc not found`

**原因**：缺少 C 编译器

**解决**：
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora
sudo dnf install gcc gcc-c++

# macOS
xcode-select --install
```

#### 问题：`pkg-config not found`

**解决**：
```bash
# Ubuntu/Debian
sudo apt-get install pkg-config

# Fedora
sudo dnf install pkgconfig

# macOS
brew install pkg-config
```

#### 问题：`cannot find -lgtk-3`

**解决**：
```bash
# Ubuntu/Debian
sudo apt-get install libgtk-3-dev

# Fedora
sudo dnf install gtk3-devel
```

### 5.2 交叉编译问题

#### 问题：`cross: Docker is not installed`

**解决**：安装 Docker
```bash
# Ubuntu
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# 或使用 Docker Desktop for Mac/Windows
```

#### 问题：cross 构建缓慢

**解决**：
- 确保 Docker 正常运行
- 使用本地工具链代替 cross（如果支持）
- 配置 cargo 镜像加速

### 5.3 Windows MSI 打包问题

#### 问题：`cargo wix` 失败

**解决**：
```powershell
# 初始化 WiX 配置
cargo wix init --force

# 确保已安装 WiX Toolset
# 下载地址：https://wixtoolset.org/
```

### 5.4 macOS DMG 打包问题

#### 问题：`create-dmg: command not found`

**解决**：
```bash
brew install create-dmg
```

#### 问题：DMG 创建失败（资源繁忙）

**解决**：
```bash
# 强制卸载已挂载的镜像
hdiutil detach /Volumes/Image\ Viewer -force
```

### 5.5 运行问题

#### 问题：`cannot open shared object file: libgtk-3.so.0`

**解决**：
```bash
# 安装运行时依赖
sudo apt-get install libgtk-3-0
```

#### 问题：macOS 显示 "无法打开，因为无法验证开发者"

**解决**：
```bash
# 临时允许（不推荐长期使用）
sudo xattr -rd com.apple.quarantine /path/to/image-viewer

# 或在系统设置 → 隐私与安全 → 安全性 中点击"仍要打开"
```

#### 问题：Windows SmartScreen 阻止运行

**解决**：
- 点击 "更多信息" → "仍要运行"
- 或使用签名证书对可执行文件签名（需要购买证书）

### 5.6 GitHub Actions 问题

#### 问题：Workflow 未触发

**检查**：
- 标签格式是否正确（`v*` 开头）
- Workflow 文件是否在 `.github/workflows/` 目录
- 是否有语法错误（查看 Actions 页面的错误提示）

#### 问题：Artifact 上传失败

**解决**：
- 检查文件路径是否正确
- 确保文件存在且大小合理

---

## 附录：快速参考

### 一键打包脚本

#### Linux/macOS

```bash
#!/bin/bash
VERSION=${1:-0.1.0}
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

# 构建
cargo build --release

# 打包
mkdir -p dist/image-viewer-v${VERSION}-${OS}-${ARCH}
cp target/release/image-viewer dist/image-viewer-v${VERSION}-${OS}-${ARCH}/
cp README.md LICENSE dist/image-viewer-v${VERSION}-${OS}-${ARCH}/
cd dist
tar czvf ../image-viewer-v${VERSION}-${OS}-${ARCH}.tar.gz image-viewer-v${VERSION}-${OS}-${ARCH}/

echo "打包完成: image-viewer-v${VERSION}-${OS}-${ARCH}.tar.gz"
```

#### Windows (PowerShell)

```powershell
param([string]$Version = "0.1.0")

# 构建
cargo build --release

# 打包
New-Item -ItemType Directory -Force -Path "dist\image-viewer-v${Version}-windows-x86_64"
Copy-Item "target\release\image-viewer.exe" "dist\image-viewer-v${Version}-windows-x86_64\"
Copy-Item "README.md" "dist\image-viewer-v${Version}-windows-x86_64\"
Copy-Item "LICENSE" "dist\image-viewer-v${Version}-windows-x86_64\"

# 压缩
Compress-Archive -Path "dist\image-viewer-v${Version}-windows-x86_64" -DestinationPath "image-viewer-v${Version}-windows-x86_64.zip" -Force

Write-Host "打包完成: image-viewer-v${Version}-windows-x86_64.zip"
```

### 平台支持矩阵

| 功能 | Linux x86_64 | Linux aarch64 | Windows x86_64 | macOS x86_64 | macOS aarch64 |
|------|:------------:|:-------------:|:--------------:|:------------:|:-------------:|
| 本地构建 | ✅ | ✅ | ✅ | ✅ | ✅ |
| CI/CD 自动构建 | ✅ | ✅ | ✅ | ✅ | ✅ |
| ZIP/TAR.GZ 包 | ✅ | ✅ | ✅ | ✅ | ✅ |
| MSI 安装包 | ❌ | ❌ | ✅ | ❌ | ❌ |
| DMG 安装包 | ❌ | ❌ | ❌ | ✅ | ✅ |
| 交叉编译支持 | - | ✅ | ⚠️ | ❌ | ❌ |

> **图例**: ✅ 支持 | ❌ 不支持 | ⚠️ 有限支持 | - 不适用

---

*最后更新: 2026年*
