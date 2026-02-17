# 更新日志

[English](./CHANGELOG.md) | 简体中文

本文件记录了项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
项目遵循 [语义化版本](https://semver.org/lang/zh-CN/spec/v2.0.0.html)。

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
- 🌐 **英文为主要 README** - 将 README.md 切换为英文版，中文版移至 README_CN.md

#### 基础设施
- ⚙️ **GitHub Actions 工作流**
  - CI 流水线（`.github/workflows/ci.yml`）
  - GPUI 兼容性测试（`.github/workflows/gpui-compatibility.yml`）
  - 发布自动化（`.github/workflows/release.yml`）
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

### 🔧 功能增强
- 解析器改进以支持更好的语法
- `src/codegen.rs` 中的代码生成优化
- 增强宏错误报告

### 🛠️ 内部改进
- 添加 `trybuild` 以支持编译测试
- 改进项目结构和组织方式
- 更新 Cargo.toml 关键字和分类

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

[0.2.0]: https://github.com/wsafight/gpui-rsx/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/wsafight/gpui-rsx/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wsafight/gpui-rsx/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wsafight/gpui-rsx/releases/tag/v0.1.0
