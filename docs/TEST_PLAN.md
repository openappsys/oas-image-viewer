# 图片查看器测试计划

> **项目**：oas-image-viewer  
> **技术栈**：Rust + egui  
> **版本**：0.3.3  
> **目标覆盖率**：整体 85%+，核心业务 90%+

---

## 1. 测试策略

### 1.1 单元测试范围

| 模块 | 测试重点 | 优先级 |
|------|----------|--------|
| `decoder` | 格式检测、图像解码、错误处理 | P0 |
| `config` | 配置序列化/反序列化、文件读写 | P0 |
| `utils/errors` | 错误类型转换、错误消息 | P1 |
| `gallery` | 状态管理、图像列表操作 | P1 |
| `viewer` | 缩放逻辑、偏移计算 | P1 |
| `app` | 视图切换逻辑 | P2 |

### 1.2 集成测试范围

| 测试场景 | 描述 | 工具 |
|----------|------|------|
| 文件操作流 | 打开目录→扫描图片→加载缩略图→显示 | tempfile + 样本图片 |
| 配置持久化 | 启动→修改配置→保存→重启→验证 | tempfile |
| 解码器集成 | 各类格式图片的正确解码 | 样本图片库 |
| 错误恢复 | 损坏图片/权限问题的优雅处理 | mock文件 |

### 1.3 GUI 测试策略

**egui 单元测试难点：**
- egui 依赖 `Context` 和实时渲染循环，传统单元测试难以模拟
- UI 状态与渲染紧密耦合
- 异步纹理加载难以在测试中控制

**应对方案：**

```rust
// 策略1: 将业务逻辑与UI分离
// 将核心逻辑提取到可测试的纯函数

// 策略2: 使用 egui::Ui 的 mock
#[cfg(test)]
mod tests {
    use egui::test::TestContext;
    
    #[test]
    fn test_gallery_selection() {
        let mut gallery = Gallery::new(test_config());
        gallery.add_image(PathBuf::from("test.png"));
        
        // 测试状态逻辑，而非渲染
        assert_eq!(gallery.images.len(), 1);
        assert!(gallery.selected_index.is_none());
        
        gallery.select(0); // 业务方法
        assert_eq!(gallery.selected_index, Some(0));
    }
}

// 策略3: 截图测试（可选）
// 使用 egui_kittest 进行视觉回归测试
```

---

## 2. 测试工具链

### 2.1 cargo-tarpaulin（覆盖率）

```bash
# 安装
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html --out Xml --output-dir coverage/

# 排除UI代码
cargo tarpaulin --exclude-files "src/adapters/egui/app/*" --out Html
```

**配置**（`.tarpaulin.toml`）：
```toml
[engine]
engine = "Llvm"

[output]
out = ["Html", "Xml", "Stdout"]
output-dir = "coverage"

[exclude]
exclude-files = ["src/adapters/egui/app/*", "src/main.rs"]
```

### 2.2 mockall（模拟）

**Cargo.toml 添加：**
```toml
[dev-dependencies]
mockall = "0.12"
tempfile = "3.8"
```

**使用示例：**
```rust
use mockall::automock;

#[automock]
pub trait ImageLoader {
    fn load(&self, path: &Path) -> Result<DynamicImage, DecoderError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decoder_with_mock() {
        let mut mock = MockImageLoader::new();
        mock.expect_load()
            .with(eq(Path::new("test.png")))
            .returning(|_| Ok(create_test_image()));
    }
}
```

### 2.3 tempfile（临时文件）

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_config_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    // 创建测试配置
    let config = Config::default();
    
    // 保存到临时目录
    fs::write(&config_path, toml::to_string(&config).unwrap()).unwrap();
    
    // 验证文件内容
    let loaded = fs::read_to_string(&config_path).unwrap();
    assert!(loaded.contains("thumbnail_size"));
}
```

---

## 3. 测试用例大纲

### 3.1 decoder 模块

| 用例ID | 测试点 | 输入 | 期望结果 | 类型 |
|--------|--------|------|----------|------|
| DEC-001 | 检测 PNG 格式 | test.png | Ok(ImageFormat::Png) | 正例 |
| DEC-002 | 检测 JPEG 格式 | test.jpg, test.jpeg | Ok(ImageFormat::Jpeg) | 正例 |
| DEC-003 | 检测不支持的格式 | test.xyz | Err(UnsupportedFormat) | 反例 |
| DEC-004 | 解码有效 PNG | 有效PNG文件 | Ok(DynamicImage) | 正例 |
| DEC-005 | 解码损坏文件 | 损坏的PNG | Err(DecodeFailed) | 反例 |
| DEC-006 | 解码大文件 | 100MB+ 图片 | 内存安全处理 | 边界 |
| DEC-007 | 内存解码 | 字节数组 | Ok(DynamicImage) | 正例 |
| DEC-008 | 空文件 | 0字节文件 | Err(DecodeFailed) | 边界 |

### 3.2 config 模块

| 用例ID | 测试点 | 输入 | 期望结果 | 类型 |
|--------|--------|------|----------|------|
| CFG-001 | 默认配置 | 无 | 各字段有默认值 | 正例 |
| CFG-002 | TOML 序列化 | Config 对象 | 有效 TOML 字符串 | 正例 |
| CFG-003 | TOML 反序列化 | 有效 TOML | Ok(Config) | 正例 |
| CFG-004 | 无效 TOML | 无效语法 | Err(ParseError) | 反例 |
| CFG-005 | 保存到文件 | Config | 文件创建成功 | 正例 |
| CFG-006 | 从文件加载 | 存在配置文件 | Ok(Config) | 正例 |
| CFG-007 | 配置不存在 | 无文件 | 创建默认配置 | 边界 |
| CFG-008 | 配置路径 | 无 | 返回有效路径 | 正例 |

### 3.3 gallery 模块

| 用例ID | 测试点 | 输入 | 期望结果 | 类型 |
|--------|--------|------|----------|------|
| GAL-001 | 添加图片 | PathBuf | images.len() == 1 | 正例 |
| GAL-002 | 清空图库 | 有图片的gallery | images.is_empty() | 正例 |
| GAL-003 | 选择图片 | index = 0 | selected_index = Some(0) | 正例 |
| GAL-004 | 选择越界 | index = 999 | 不 panic | 边界 |
| GAL-005 | 空目录加载 | [] | images.is_empty() | 边界 |
| GAL-006 | 大量图片 | 1000+ 图片 | 性能可接受 | 压力 |

### 3.4 viewer 模块

| 用例ID | 测试点 | 输入 | 期望结果 | 类型 |
|--------|--------|------|----------|------|
| VIEW-001 | 设置图片 | PathBuf | current_image.is_some() | 正例 |
| VIEW-002 | 清空图片 | 无 | current_image.is_none() | 正例 |
| VIEW-003 | 放大 | zoom_in() | scale 增加 | 正例 |
| VIEW-004 | 缩小 | zoom_out() | scale 减少 | 正例 |
| VIEW-005 | 重置缩放 | reset_zoom() | scale = 1.0 | 正例 |
| VIEW-006 | 缩放上限 | 多次 zoom_in | scale <= 10.0 | 边界 |
| VIEW-007 | 缩放下限 | 多次 zoom_out | scale >= 0.1 | 边界 |
| VIEW-008 | 适应窗口计算 | 图片100x100, 窗口200x200 | 返回100x100 | 正例 |

### 3.5 边界条件测试

| 场景 | 测试内容 | 预期行为 |
|------|----------|----------|
| **空目录** | 打开无图片的目录 | 显示空状态，不 panic |
| **大文件** | 打开 >500MB 图片 | 优雅处理，提示用户 |
| **损坏图片** | 打开损坏的 JPEG/PNG | 显示错误提示，继续运行 |
| **无权限文件** | 打开只读权限图片 | 错误提示，不崩溃 |
| **超长路径** | 路径 >4096 字符 | 正确处理或提示 |
| **特殊字符** | 文件名含 emoji/中文 | 正确显示和处理 |
| **并发加载** | 快速切换图片 | 无竞态条件 |
| **内存不足** | 加载超大图片 | 优雅降级 |

---

## 4. 覆盖率目标

### 4.1 分层目标

```
┌─────────────────────────────────────────────────────┐
│  核心业务逻辑 (decoder, config)     目标: 90%+      │
├─────────────────────────────────────────────────────┤
│  整体项目                           目标: 85%+      │
├─────────────────────────────────────────────────────┤
│  关键路径 (图片加载→显示)            目标: 95%+      │
└─────────────────────────────────────────────────────┘
```

### 4.2 豁免清单

以下代码可不计入覆盖率目标：
- `src/main.rs` - 程序入口，主要是框架代码
- `src/adapters/egui/app/` - UI 渲染与交互代码（难以单元测试）
- `src/adapters/egui/widgets/` - UI 组件渲染逻辑

### 4.3 覆盖率追踪

```bash
# CI 中运行
cargo tarpaulin --out Xml --output-dir coverage/

# 查看行覆盖率
cargo tarpaulin --out Html && open coverage/tarpaulin-report.html
```

---

## 5. CI 集成

### 5.1 GitHub Actions 配置

```yaml
# .github/workflows/test.yml
name: Test & Coverage

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Run tests
        run: cargo test --all-features

      - name: Generate coverage
        run: cargo tarpaulin --out Xml --output-dir coverage/

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
          fail_ci_if_error: true
          verbose: true

  coverage-gate:
    runs-on: ubuntu-latest
    needs: test
    steps:
      - name: Check coverage threshold
        run: |
          # 使用 tarpaulin 的 --fail-under 选项
          cargo tarpaulin --fail-under 85
```

### 5.2 PR 门禁

```yaml
# .github/workflows/pr-gate.yml
name: PR Gate

on:
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run tests
        run: cargo test

      - name: Check formatting
        run: cargo fmt --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Coverage check
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --fail-under 85 --exclude-files "src/adapters/egui/app/*"
```

### 5.3 覆盖率报告集成

**Codecov 徽章：**
```markdown
[![codecov](https://codecov.io/gh/openappsys/oas-image-viewer/branch/main/graph/badge.svg)](https://codecov.io/gh/openappsys/oas-image-viewer)
```

---

## 6. 验收测试清单

### 6.1 功能验收

| 功能 | 测试步骤 | 预期结果 | 状态 |
|------|----------|----------|------|
| 启动应用 | 运行 `cargo run` | 窗口正常显示，无崩溃 | ⬜ |
| 打开图片 | File → Open → 选择 PNG | 图片正确显示 | ⬜ |
| 格式支持 | 打开 JPG/GIF/WEBP/BMP/TIFF | 各格式正确显示 | ⬜ |
| 图库浏览 | 打开含多张图片的目录 | 缩略图正确显示 | ⬜ |
| 图片选择 | 点击缩略图 | 切换到大图视图 | ⬜ |
| 缩放操作 | 滚轮上/下 | 图片放大/缩小 | ⬜ |
| 拖拽平移 | 按住拖拽 | 图片跟随移动 | ⬜ |
| 重置视图 | 双击/按钮 | 恢复到初始状态 | ⬜ |
| 配置保存 | 修改设置→重启 | 设置保持 | ⬜ |
| 错误处理 | 打开损坏图片 | 显示错误提示 | ⬜ |

### 6.2 性能验收

| 场景 | 测试方法 | 目标 | 状态 |
|------|----------|------|------|
| 冷启动 | 从点击到可交互 | < 2s | ⬜ |
| 大图加载 | 打开 10MB 图片 | < 1s | ⬜ |
| 缩略图生成 | 100张图片目录 | < 3s | ⬜ |
| 内存使用 | 打开 50MB 图片 | 峰值 < 200MB | ⬜ |
| 响应速度 | UI 操作延迟 | < 16ms (60fps) | ⬜ |

### 6.3 兼容性验收

| 平台 | 测试步骤 | 状态 |
|------|----------|------|
| Linux | 在 Ubuntu 22.04 运行 | ⬜ |
| macOS | 在 macOS 14 运行 | ⬜ |
| Windows | 在 Windows 11 运行 | ⬜ |

---

## 7. 测试实施计划

### Phase 1: 基础测试（Week 1）
- [ ] 配置 `decoder` 模块单元测试
- [ ] 配置 `config` 模块单元测试
- [ ] 集成 cargo-tarpaulin

### Phase 2: 扩展测试（Week 2）
- [ ] `gallery` 模块测试
- [ ] `viewer` 模块测试
- [ ] 边界条件测试用例

### Phase 3: CI 集成（Week 3）
- [ ] GitHub Actions 配置
- [ ] Codecov 集成
- [ ] PR 门禁配置

### Phase 4: 验收测试（Week 4）
- [ ] 手动功能测试
- [ ] 性能基准测试
- [ ] 覆盖率达标验证

---

## 8. 附录

### 8.1 测试数据准备

```bash
# 创建测试图片目录
mkdir -p tests/data/images

# 生成测试图片（使用 ImageMagick）
convert -size 100x100 xc:red tests/data/images/red.png
convert -size 200x200 xc:blue tests/data/images/blue.jpg
convert -size 50x50 xc:green tests/data/images/green.gif

# 创建损坏的测试文件
echo "not an image" > tests/data/images/corrupted.png
```

### 8.2 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test decoder::

# 运行带输出的测试
cargo test -- --nocapture

# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage/
```

### 8.3 相关链接

- [cargo-tarpaulin 文档](https://github.com/xd009642/tarpaulin)
- [mockall 文档](https://docs.rs/mockall/)
- [egui 测试示例](https://github.com/emilk/egui/tree/master/crates/egui/tests)

---

*文档版本：1.0*  
*最后更新：2026-02-28*  
*维护者：QA Team*
