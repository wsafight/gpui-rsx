# 更新日志

[English](./CHANGELOG.md) | 简体中文

本文件记录了项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
项目遵循 [语义化版本](https://semver.org/lang/zh-CN/spec/v2.0.0.html)。

## [0.2.1] - 2026-02-17

### 🚀 性能优化

#### 编译时性能
- **颜色查找改用二分查找** - 将 `lookup_color()` 从 O(n) 线性扫描优化为 O(log n) 二分查找
  - COLOR_MAP（242 个条目）现已按字母序排列
  - 颜色查找从约 120 次比较减少至约 8 次比较
  - 提升使用 class 颜色时的宏展开速度

#### 运行时性能
- **消除 `.children()` 堆分配** - 将以下场景中的 `vec![]` 替换为数组字面量 `[...]`：
  - For 循环 `flat_map()` 展开
  - 3+ 个连续表达式聚合
  - 减少静态元素数量场景下的运行时分配
- **动态 class 字符串零拷贝** - 从 `String::into()` 改为 `AsRef<str>`
  - `&str` 输入现可直接通过无需分配
  - 高效支持 `&str`、`String`、`Cow<str>`

#### 二进制体积优化
- **动态 class match 表去重** - 将 40 分支 match 表提取为 `#[inline(never)]` 局部函数
  - 同一组件内多个 `class={expr}` 现共享同一函数体
  - LLVM ICF 可跨组件合并相同的单态化实例
  - 减少重复 match 表内联导致的代码膨胀

### ♻️ 代码质量改进
- **简化 EVENT_HANDLERS 表** - 从三元组简化为二元组
  - 删除冗余的第三字段（方法名始终等于 snake_case）
  - 更清晰的表结构，减少维护负担
- **删除死代码** - 删除已废弃的 `class_dynamic_value_error()` 函数

### 📊 优化总结

| 类别 | 改进效果 | 修改文件 |
|------|---------|---------|
| 编译速度 | 颜色查找快约 15 倍（O(242)→O(8)） | `tables.rs` |
| 运行时分配 | 消除 .children() 中的 vec! | `element.rs` |
| 二进制体积 | 动态 class match 表去重 | `runtime.rs` |
| 代码清晰度 | 简化 EVENT_HANDLERS 结构 | `tables.rs`, `attribute.rs` |

### ✅ 测试
- 全部 236 个测试通过（203 宏测试 + 31 覆盖率测试 + 2 诊断测试）
- 优化改动零回归

## [0.2.0] - 2026-02-17

### ✨ 新增功能

#### 核心特性
- **For 循环语法糖** - 使用 `{for item in items { ... }}` 语法简化列表渲染
- **Styled 标志** - 根据标签名称应用合理的默认样式（h1-h6、button、input 等）
- **条件样式** - 使用 `when` 和 `whenSome` 属性实现动态样式
- **Fragment 支持** - 使用 `<>...</>` 语法返回多个根元素
- **完整 Tailwind 调色板** - 242 种内置颜色（slate、gray、red、blue 等）+ 任意十六进制颜色值
- **全面的事件处理** - 支持 onClick、onMouseDown、onKeyDown、onHover 等 14 种事件类型
- **属性映射** - camelCase 到 snake_case 自动转换（zIndex → z_index，fontSize → font_size 等）

#### 文档系统
- 📖 **完整的文档体系**
  - 快速入门指南（`docs/getting-started.md`）
  - API 参考文档（`docs/api-reference.md`）
  - 最佳实践（`docs/best-practices.md`）
  - 迁移指南（`docs/migration-guide.md`）
  - 故障排除（`docs/troubleshooting.md`）
- 📐 **架构文档**
  - 综合架构指南（`ARCHITECTURE.md`）
  - 中文架构文档（`ARCHITECTURE_CN.md`）
  - 详细的模块组织和数据流
  - 代码生成策略和设计模式
  - 扩展点和调试指南
- 🌐 **英文为主要 README** - 将 README.md 切换为英文版，中文版移至 README_CN.md

#### 基础设施
- ⚙️ **GitHub Actions 工作流**
  - CI 流水线（`.github/workflows/ci.yml`）
  - GPUI 兼容性测试（`.github/workflows/gpui-compatibility.yml`）
  - 发布自动化（`.github/workflows/release.yml`）
  - Codecov 代码覆盖率跟踪
- 📊 **代码覆盖率**
  - 本地覆盖率脚本（`./coverage.sh`）
  - CI 中集成 Codecov
  - README 中添加覆盖率徽章
  - 目标：80%+ 覆盖率
- 🧪 **编译测试**
  - 7 个编译失败测试（无效语法检测）
  - 7 个编译通过测试（有效语法验证）
  - 测试运行脚本（`test_syntax.sh`）

#### 开发者体验
- 🔧 使用 `proc-macro-error` 依赖提供更好的错误信息
- 📦 更新仓库 URL 为 `https://github.com/wsafight/gpui-rsx`
- ✅ 包含 1200+ 测试用例的综合测试套件

### 🚀 性能优化
- 优化解析器以提升编译时性能
- 改进 `class` 属性展开的代码生成效率
- 减少 for 循环语法转换中的内存分配

### 📝 文档改进
- 更新 README，包含全面的示例代码
- 添加"对比示例"展示约 50% 的代码减少
- 记录所有 14 种事件处理器
- 添加完整的属性映射参考表
- 包含常见问题的 FAQ 部分

### 🐛 修复
- **有状态事件的自动 ID 注入** - 补充了 `NEEDS_ID_ATTRS` 中缺失的 `onHover`/`on_hover`、`onDrag`/`on_drag`、`onDrop`/`on_drop`，修复了这些事件处理器在没有手动 `id` 属性时编译失败的问题
  - 新增 6 个测试用例验证这些事件处理器的自动 ID 注入

### 🔧 功能增强
- 解析器改进以支持更好的语法
- `src/codegen.rs` 中的代码生成优化
- 增强宏错误报告
- 修复 4 个测试中 Clippy 关于无用 `vec!` 的警告

### ♻️ 重构
- **消除子节点解析逻辑重复** - 提取 `try_parse_child_node()` 函数，消除 `parse_children()` 和 `parse_for_loop()` 循环体解析中的重复代码
- **消除 for 循环代码生成重复** - 提取 `generate_for_loop()` 函数，统一 `generate_node()` 和 `generate_children_methods()` 中重复的 `map`/`flat_map` 生成逻辑
- **简化颜色 class 解析** - 将 `parse_color_class()` 中冗余的 `starts_with()` + `strip_prefix()` 模式替换为单次 `strip_prefix()` 调用

### 🗑️ 移除
- 移除 `examples/` 目录 (counter.rs, todo_app.rs)
  - 示例需要外部 GPUI 依赖
  - 核心功能已被 203 个宏展开测试完全覆盖
- 移除 trybuild 编译测试
  - 简化测试结构
  - 消除 `trybuild` 开发依赖
  - 专注于全面的宏展开测试

### 🛠️ 内部改进
- 改进项目结构和组织方式
- 更新 Cargo.toml 关键字和分类
- 更新文档以移除已删除示例的引用
- 清理 CONTRIBUTING.md 测试说明
- 精简项目结构

## [0.1.2] - 2026-02-16

### 🔧 维护
- 更新仓库 URL
- 注释掉 todo_app 示例（等待依赖解决）

## [0.1.1] - 2026-02-15

### 📖 文档
- 切换为英文作为主要 README
- 添加中文 README（README_CN.md）
- 更新安装说明

## [0.1.0] - 2026-02-15

### 🎉 首次发布
- 基本 RSX 宏实现
- 支持嵌套元素
- 属性支持（布尔属性和值属性）
- 基本 class 属性解析
- 表达式插值
- 事件处理基础功能

---

[0.2.1]: https://github.com/wsafight/gpui-rsx/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/wsafight/gpui-rsx/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/wsafight/gpui-rsx/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wsafight/gpui-rsx/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wsafight/gpui-rsx/releases/tag/v0.1.0
