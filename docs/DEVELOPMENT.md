# 开发文档

本文档面向希望为 Image Viewer 项目贡献代码的开发者。

## 目录

- [环境搭建](#环境搭建)
- [项目结构](#项目结构)
- [代码规范](#代码规范)
- [调试技巧](#调试技巧)
- [提交规范](#提交规范)

---

## 环境搭建

### 1. 安装 Rust

确保安装 Rust 1.93 或更高版本：

```bash
# 使用 rustup 安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 更新到最新稳定版
rustup update stable

# 验证版本
rustc --version  # 应显示 1.93.0 或更高
```

### 2. 安装系统依赖

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

#### Linux (Fedora)

```bash
sudo dnf install gtk3-devel libxcb-devel libxkbcommon-devel openssl-devel
```

#### Linux (Arch)

```bash
sudo pacman -S gtk3 libxcb libxkbcommon openssl
```

#### macOS

无需额外依赖，Xcode Command Line Tools 已足够：

```bash
xcode-select --install
```

#### Windows
n
无需额外依赖。

### 3. 克隆项目

```bash
git clone https://github.com/yourusername/image-viewer.git
cd image-viewer
```

### 4. 验证环境

```bash
# 检查格式
cargo fmt -- --check

# 运行 clippy
cargo clippy -- -D warnings

# 运行测试
cargo test

# 构建项目
cargo build
```

---

## 项目结构

```
image-viewer/
├── src/
│   ├── main.rs          # 应用程序入口点
│   ├── config.rs        # 配置管理（加载、保存、默认值）
│   ├── app/             # 主应用程序逻辑
│   │   └── mod.rs       # App 状态、事件处理、UI 布局
│   ├── viewer/          # 图片查看器组件
│   │   └── mod.rs       # 图片显示、缩放、平移
│   ├── gallery/         # 画廊网格组件
│   │   └── mod.rs       # 缩略图网格、选择
│   ├── decoder/         # 图片解码
│   │   └── mod.rs       # 图片加载、格式支持
│   └── utils/           # 工具模块
│       ├── mod.rs       # 模块导出
│       ├── errors.rs    # 错误类型定义
│       └── threading.rs # 线程工具
├── assets/              # 静态资源（图标等）
├── .github/workflows/   # CI/CD 配置
├── docs/                # 文档
├── Cargo.toml          # 项目配置和依赖
├── rustfmt.toml        # 格式化配置
├── .clippy.toml        # Clippy 配置
└── .editorconfig       # 编辑器配置
```

### 模块职责

| 模块 | 职责 |
|------|------|
| `main.rs` | 程序入口，初始化日志、加载配置、启动 eframe |
| `config.rs` | 配置结构定义、TOML 序列化/反序列化、文件 I/O |
| `app` | 主应用状态机，协调 viewer 和 gallery，处理用户输入 |
| `viewer` | 单图片查看：缩放、平移、适应窗口 |
| `gallery` | 缩略图网格：加载、缓存、选择 |
| `decoder` | 图片解码抽象，支持多种格式 |
| `utils` | 共享错误类型、线程池封装 |

---

## 代码规范

### Rust 代码风格

我们使用以下工具确保代码质量：

#### rustfmt

配置见 `rustfmt.toml`：

```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Default"
```

运行格式化：

```bash
cargo fmt
```

#### Clippy

配置见 `.clippy.toml` 和 CI 中的严格模式：

```bash
# 基本检查
cargo clippy

# 严格模式（CI 使用）
cargo clippy -- -D warnings -W clippy::all
```

#### 代码审查清单

- [ ] 所有公共 API 都有文档注释
- [ ] 错误处理使用 `anyhow` 或 `thiserror`
- [ ] 异步操作使用适当的线程池
- [ ] 避免 `unwrap()` 和 `expect()`，使用 `?` 或显式错误处理
- [ ] 复杂逻辑添加行内注释

### 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 结构体 | PascalCase | `ImageViewer`, `GalleryState` |
| 函数/方法 | snake_case | `load_image()`, `zoom_in()` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_ZOOM_LEVEL` |
| 枚举 | PascalCase | `ViewMode::Gallery` |
| 模块 | snake_case | `decoder`, `utils` |

---

## 调试技巧

### 日志级别

我们在代码中使用 `tracing` crate 进行日志记录：

```rust
use tracing::{info, debug, warn, error};

info!("Application started");
debug!("Loading image: {:?}", path);
warn!("Failed to load config, using defaults");
error!("Critical error: {}", e);
```

运行时使用环境变量控制日志级别：

```bash
# 显示所有日志
RUST_LOG=debug cargo run

# 仅显示 info 及以上
RUST_LOG=info cargo run

# 特定模块
RUST_LOG=image_viewer=debug,eframe=warn cargo run
```

### 调试构建

开发时使用调试构建，编译更快且包含调试符号：

```bash
cargo run  # 默认是 debug 构建
```

### IDE 调试

#### VS Code

使用 `CodeLLDB` 或 `rust-analyzer` 扩展：

```json
// .vscode/launch.json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Image Viewer",
            "cargo": {
                "args": ["build"],
                "filter": {
                    "name": "image-viewer",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### 常见调试场景

#### 图片加载失败

```rust
// 在 decoder/mod.rs 中添加日志
tracing::debug!("Attempting to load: {:?}", path);
match image::open(&path) {
    Ok(img) => tracing::debug!("Loaded {}x{}", img.width(), img.height()),
    Err(e) => tracing::error!("Failed to load {:?}: {}", path, e),
}
```

#### UI 响应问题

```rust
// 使用 tracing 记录帧时间
use std::time::Instant;

let start = Instant::now();
// ... 渲染代码 ...
tracing::debug!("Frame render time: {:?}", start.elapsed());
```

---

## 提交规范

### 提交信息格式

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

### 类型 (Type)

| 类型 | 含义 |
|------|------|
| `feat` | 新功能 |
| `fix` | 修复 Bug |
| `docs` | 仅文档更改 |
| `style` | 不影响代码含义的更改（格式化、分号等） |
| `refactor` | 既不修复 Bug 也不添加功能的代码重构 |
| `perf` | 性能优化 |
| `test` | 添加或修正测试 |
| `chore` | 构建过程或辅助工具的变动 |

### 示例

```bash
# 新功能
feat(gallery): 添加缩略图缓存

# Bug 修复
fix(viewer): 修复缩放时的闪烁问题

# 文档
docs: 更新 README 中的安装说明

# 重构
refactor(decoder): 简化错误处理

# 性能优化
perf(gallery): 使用 Rayon 并行加载缩略图
```

### 分支命名

```
feature/description    # 新功能
fix/description        # Bug 修复
docs/description       # 文档更新
refactor/description   # 重构
```

### Pull Request 流程

1. 从 `main` 创建功能分支
2. 进行更改并提交（遵循提交规范）
3. 确保 CI 通过：
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   ```
4. 推送到远程并创建 PR
5. 等待代码审查
6. 合并后删除分支

---

## 有用的资源

- [Rust 官方文档](https://doc.rust-lang.org/)
- [egui 文档](https://docs.rs/egui/)
- [eframe 文档](https://docs.rs/eframe/)
- [image crate 文档](https://docs.rs/image/)
- [Rust 设计模式](https://rust-unofficial.github.io/patterns/)
