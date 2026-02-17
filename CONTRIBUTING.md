# Contributing to GPUI-RSX

English | [简体中文](#贡献指南)

Thank you for your interest in contributing to GPUI-RSX! We welcome contributions from the community.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo
- Git

### Setting Up Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/gpui-rsx.git
   cd gpui-rsx
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/wsafight/gpui-rsx.git
   ```
4. Build the project:
   ```bash
   cargo build
   ```
5. Run tests to ensure everything works:
   ```bash
   cargo test
   ```

## Development Workflow

### Creating a Branch

Create a feature branch for your changes:

```bash
git checkout -b feature/amazing-feature
```

Branch naming conventions:
- `feature/description` - for new features
- `fix/description` - for bug fixes
- `docs/description` - for documentation changes
- `refactor/description` - for code refactoring
- `test/description` - for test improvements

### Making Changes

1. Make your changes in your feature branch
2. Add tests for any new functionality
3. Ensure all tests pass: `cargo test`
4. Format your code: `cargo fmt`
5. Run linter: `cargo clippy -- -D warnings`
6. Update documentation if needed

### Testing Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Generate code coverage report
./coverage.sh          # HTML report (opens in browser on macOS)
./coverage.sh --text   # Terminal output only

# View macro expansions (for debugging)
cargo install cargo-expand
cargo expand --lib

# Run test script
bash test_syntax.sh
```

## Pull Request Process

### Before Submitting

1. Update CHANGELOG.md with your changes
2. Ensure all tests pass
3. Update documentation if applicable
4. Add yourself to the contributors list if it's your first contribution

### Submitting a PR

1. Push your changes to your fork:
   ```bash
   git push origin feature/amazing-feature
   ```

2. Go to the [repository](https://github.com/wsafight/gpui-rsx) and create a Pull Request

3. Fill in the PR template with:
   - Description of changes
   - Related issue number (if applicable)
   - Type of change (feature, bugfix, docs, etc.)
   - Testing done
   - Screenshots (if UI changes)

4. Wait for review and address any feedback

### PR Requirements

- ✅ All tests must pass
- ✅ Code must be formatted with `cargo fmt`
- ✅ No warnings from `cargo clippy`
- ✅ Documentation updated (if applicable)
- ✅ CHANGELOG.md updated
- ✅ Descriptive commit messages

## Coding Standards

### Rust Style

- Follow the [Rust Style Guide](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` with default settings
- Address all `cargo clippy` warnings

### Code Organization

```
src/
├── lib.rs          # Main macro entry point
├── parser.rs       # Syntax parsing logic
└── codegen.rs      # Code generation logic
```

### Naming Conventions

- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Use `SCREAMING_SNAKE_CASE` for constants
- Prefix private items with underscore when unused

### Comments

- Write doc comments (`///`) for public APIs
- Use regular comments (`//`) for implementation details
- Keep comments concise and clear
- Update comments when code changes

### Example

```rust
/// Parses an RSX element from the token stream.
///
/// # Arguments
///
/// * `input` - The token stream to parse
///
/// # Returns
///
/// Returns a `Result` containing the parsed element or an error.
///
/// # Examples
///
/// ```rust
/// let element = parse_element(tokens)?;
/// ```
fn parse_element(input: ParseStream) -> Result<Element> {
    // Implementation details...
}
```

## Testing Guidelines

### Test Types

1. **Unit Tests** - Test individual functions
   ```rust
   #[test]
   fn test_parse_basic_element() {
       // Test implementation
   }
   ```

2. **Integration Tests** - Test macro expansion
   - Located in `tests/macro_tests.rs`
   - Cover various RSX syntax patterns

### Writing Tests

- Test both success and failure cases
- Use descriptive test names
- Add comments explaining complex test scenarios
- Keep tests focused and independent

### Test Coverage

We aim for high test coverage (target: 80%+):
- New features must include tests
- Bug fixes should include regression tests
- Edge cases should be tested
- Run `./coverage.sh` to generate a local coverage report
- Coverage is automatically tracked in CI via Codecov

## Documentation

### Code Documentation

- Document all public APIs with `///` doc comments
- Include examples in documentation
- Explain complex algorithms
- Document panics and errors

### User Documentation

Update relevant documentation in:
- `README.md` - Project overview and quick start
- `docs/getting-started.md` - Tutorial
- `docs/api-reference.md` - API documentation
- `docs/best-practices.md` - Best practices guide
- `CHANGELOG.md` - Version history

### Examples

Add examples to demonstrate new features:
- Keep examples simple and focused
- Add comments explaining key concepts
- Test examples to ensure they work

## Community

### Getting Help

- Open an [Issue](https://github.com/wsafight/gpui-rsx/issues) for bugs or feature requests
- Check existing issues before creating new ones
- Use clear, descriptive titles

### Issue Labels

- `bug` - Something isn't working
- `enhancement` - New feature or request
- `documentation` - Documentation improvements
- `good first issue` - Good for newcomers
- `help wanted` - Extra attention needed

### Communication

- Be respectful and constructive
- Provide context and details
- Follow up on your issues and PRs
- Help others when possible

---

## 贡献指南

[English](#contributing-to-gpui-rsx) | 简体中文

感谢你对 GPUI-RSX 的关注！我们欢迎社区贡献。

## 目录

- [行为准则](#行为准则)
- [快速开始](#快速开始)
- [开发流程](#开发流程)
- [Pull Request 流程](#pull-request-流程)
- [代码规范](#代码规范)
- [测试指南](#测试指南)
- [文档](#文档)
- [社区](#社区)

## 行为准则

本项目遵循[行为准则](CODE_OF_CONDUCT.md)。参与项目即表示你同意遵守此准则。

## 快速开始

### 前置要求

- Rust 1.75 或更高版本
- Cargo
- Git

### 搭建开发环境

1. 在 GitHub 上 Fork 仓库
2. 克隆你的 Fork 到本地：
   ```bash
   git clone https://github.com/YOUR_USERNAME/gpui-rsx.git
   cd gpui-rsx
   ```
3. 添加上游远程仓库：
   ```bash
   git remote add upstream https://github.com/wsafight/gpui-rsx.git
   ```
4. 构建项目：
   ```bash
   cargo build
   ```
5. 运行测试确保一切正常：
   ```bash
   cargo test
   ```

## 开发流程

### 创建分支

为你的改动创建功能分支：

```bash
git checkout -b feature/amazing-feature
```

分支命名规范：
- `feature/描述` - 新功能
- `fix/描述` - Bug 修复
- `docs/描述` - 文档改进
- `refactor/描述` - 代码重构
- `test/描述` - 测试改进

### 进行改动

1. 在功能分支上进行修改
2. 为新功能添加测试
3. 确保所有测试通过：`cargo test`
4. 格式化代码：`cargo fmt`
5. 运行代码检查：`cargo clippy -- -D warnings`
6. 根据需要更新文档

### 测试改动

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 生成代码覆盖率报告
./coverage.sh          # HTML 报告（macOS 自动打开浏览器）
./coverage.sh --text   # 仅终端输出

# 查看宏展开（用于调试）
cargo install cargo-expand
cargo expand --lib

# 运行测试脚本
bash test_syntax.sh
```

## Pull Request 流程

### 提交前检查

1. 更新 CHANGELOG.md
2. 确保所有测试通过
3. 更新相关文档
4. 如果是首次贡献，将自己添加到贡献者列表

### 提交 PR

1. 推送改动到你的 Fork：
   ```bash
   git push origin feature/amazing-feature
   ```

2. 访问[仓库](https://github.com/wsafight/gpui-rsx)并创建 Pull Request

3. 填写 PR 模板：
   - 改动描述
   - 相关 Issue 编号（如果有）
   - 改动类型（功能、修复、文档等）
   - 测试情况
   - 截图（如果有 UI 改动）

4. 等待 Review 并处理反馈

### PR 要求

- ✅ 所有测试必须通过
- ✅ 代码必须使用 `cargo fmt` 格式化
- ✅ `cargo clippy` 无警告
- ✅ 文档已更新（如适用）
- ✅ CHANGELOG.md 已更新
- ✅ 提交信息描述清晰

## 代码规范

### Rust 风格

- 遵循 [Rust 风格指南](https://rust-lang.github.io/api-guidelines/)
- 使用默认设置的 `cargo fmt`
- 处理所有 `cargo clippy` 警告

### 代码组织

```
src/
├── lib.rs          # 主宏入口
├── parser.rs       # 语法解析逻辑
└── codegen.rs      # 代码生成逻辑
```

### 命名规范

- 函数和变量使用 `snake_case`
- 类型和 trait 使用 `PascalCase`
- 常量使用 `SCREAMING_SNAKE_CASE`
- 未使用的私有项添加下划线前缀

### 注释

- 公共 API 使用文档注释（`///`）
- 实现细节使用普通注释（`//`）
- 保持注释简洁清晰
- 代码改动时同步更新注释

### 示例

```rust
/// 从 token 流中解析 RSX 元素。
///
/// # 参数
///
/// * `input` - 要解析的 token 流
///
/// # 返回
///
/// 返回包含解析元素或错误的 `Result`。
///
/// # 示例
///
/// ```rust
/// let element = parse_element(tokens)?;
/// ```
fn parse_element(input: ParseStream) -> Result<Element> {
    // 实现细节...
}
```

## 测试指南

### 测试类型

1. **单元测试** - 测试单个函数
   ```rust
   #[test]
   fn test_parse_basic_element() {
       // 测试实现
   }
   ```

2. **集成测试** - 测试宏展开
   - 位于 `tests/macro_tests.rs`
   - 覆盖各种 RSX 语法模式

### 编写测试

- 同时测试成功和失败情况
- 使用描述性测试名称
- 为复杂测试场景添加注释
- 保持测试专注和独立

### 测试覆盖率

我们追求高测试覆盖率（目标：80%+）：
- 新功能必须包含测试
- Bug 修复应包含回归测试
- 边界情况应被测试
- 运行 `./coverage.sh` 生成本地覆盖率报告
- CI 自动通过 Codecov 跟踪覆盖率

## 文档

### 代码文档

- 使用 `///` 文档注释记录所有公共 API
- 在文档中包含示例
- 解释复杂算法
- 记录 panic 和错误情况

### 用户文档

更新相关文档：
- `README.md` - 项目概览和快速开始
- `docs/getting-started.md` - 教程
- `docs/api-reference.md` - API 文档
- `docs/best-practices.md` - 最佳实践指南
- `CHANGELOG.md` - 版本历史

### 示例

添加示例演示新功能：
- 保持示例简单专注
- 添加注释解释关键概念
- 测试示例确保可用

## 社区

### 获取帮助

- 为 bug 或功能请求开启 [Issue](https://github.com/wsafight/gpui-rsx/issues)
- 创建新 Issue 前先检查已存在的
- 使用清晰描述性的标题

### Issue 标签

- `bug` - 功能不正常
- `enhancement` - 新功能或请求
- `documentation` - 文档改进
- `good first issue` - 适合新手
- `help wanted` - 需要额外关注

### 沟通

- 保持尊重和建设性
- 提供上下文和细节
- 跟进你的 Issue 和 PR
- 尽可能帮助他人

---

Thank you for contributing to GPUI-RSX! 🎉

感谢你为 GPUI-RSX 做出贡献！🎉
