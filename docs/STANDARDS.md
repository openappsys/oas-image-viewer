# Image-Viewer 项目规范

**记录时间**: 2026-02-28  
**项目路径**: ~/projects/image-viewer/  
**状态**: 生效中

---

## 1. 测试覆盖率要求 (强制)

| 指标 | 要求 |
|------|------|
| **测试覆盖率** | 达到业界高标准（≥80%） |
| **提交门槛** | 未达标 **不得提交代码** |

### 检查命令
```bash
# 生成覆盖率报告
cargo tarpaulin --out Html

# 检查覆盖率（提交前必须执行）
cargo tarpaulin --fail-under 80
```

---

## 2. 功能分级 (PRD)

| 阶段 | 功能 ID | 状态 |
|------|---------|------|
| P0 MVP | F-001 ~ F-006 | ✅ 已完成 |
| P1 重要 | F-101 ~ F-108 | 📋 待开发 |
| P2 增强 | F-201 ~ F-206 | 📋 待开发 |

**新增 P1 功能**: F-107 拖放打开图片、F-108 右键打开方式

---

## 3. Git 提交规范

```
<type>: <描述>

type:
  - feat: 新功能
  - fix: Bug 修复
  - docs: 文档
  - ci: CI/CD 配置
  - chore: 杂项
```

---

## 4. 提交前强制检查清单

| 步骤 | 命令 | 通过标准 |
|------|------|----------|
| 1. 代码检查 | `cargo check` | 0 errors |
| 2. 构建测试 | `cargo build --release` | 成功 |
| 3. **测试覆盖率** | `cargo tarpaulin --fail-under 80` | **≥80%** |

**⚠️ 以上全部通过才能提交代码**

---

## 5. CI/CD 构建平台 (6个)

| 平台 | Runner | 状态 |
|------|--------|------|
| Windows x86_64 | windows-latest | ✅ |
| Windows MSI | windows-latest | ✅ |
| Linux x86_64 | ubuntu-latest | ✅ |
| Linux aarch64 | ubuntu-24.04-arm | ✅ |
| macOS x86_64 | macos-latest | ✅ |
| macOS aarch64 | macos-latest | ✅ |

---

## 6. 文档要求

- **所有文档使用中文**
- 文档位置: `docs/`
- 包含: PRD.md, BUILD.md, DEVELOPMENT.md, USAGE.md

---

## 7. 已知问题

| 问题 | 处理方式 |
|------|----------|
| macOS 安全警告 | 暂不处理（需 Apple Developer ID，不影响使用）|

---

## 8. 关键代码配置

**Windows 无命令行窗口** (`src/main.rs`):
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

---

*此文档为项目强制规范，所有开发者必须遵守*
