# 更新日志

[English](./CHANGELOG.md) | 简体中文

本文件记录了项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
项目遵循 [语义化版本](https://semver.org/lang/zh-CN/spec/v2.0.0.html)。

## [0.2.1] - 2026-02-18

### 🚀 性能优化

#### 编译时性能
- **基于 match 的 O(1) 颜色查找** - 将 `COLOR_MAP` 常量数组 + `.iter().find()` 线性扫描
  替换为 `lookup_color()` 中的 `match` 语句。编译器为 match 生成高效跳转表/trie，
  最坏情况下的比较次数从 O(242) 降为 O(1)。
- **属性/间距/文本大小查找同样改用 match** - 相同模式应用于 `lookup_attr_method()`、
  `lookup_spacing_method()` 和 `is_valid_text_size()`，替换了四个线性扫描数组。
- **`generate_element` 单次属性扫描** - `user_id`、`has_styled`、`needs_id` 现在在一次
  循环中提取，不再多次遍历。
- **缓存 `Ident::to_string()` 结果** - 标签名和属性名字符串在每次代码生成调用中只计算一次，
  避免重复堆分配。

#### 运行时性能
- **动态 class 字符串零拷贝** - 从 `String::into()` 改为 `AsRef<str>`。`&str` 输入无需
  分配直接通过；同样高效支持 `String` 和 `Cow<str>`。
- **`Vec` 复用避免循环内重复分配** - `consecutive_exprs` 在循环外分配一次，每轮用 `.clear()`
  清空复用，不再循环内反复分配。
- **动态 class 解析改用 `split_ascii_whitespace`** - 生成的 class 迭代现在使用
  `split_ascii_whitespace` 而非 `split_whitespace`。class 名称仅含 ASCII 字符，
  可避免标准迭代器的 Unicode 空白字符扫描开销。
- **动态 class 空字符串快速路径** - 在 fold 之前增加 `is_empty()` 判断，对空字符串
  完全跳过迭代器创建——这在 `class={if cond { "flex" } else { "" }}` 模式中十分常见。

#### 内存/分配改进
- **`parse_class_string` 返回迭代器** - 避免中间 `Vec<TokenStream>` 分配；调用方通过
  `.extend()` 直接消费到输出缓冲区。
- **`generate_attr_methods` 直接 push 到调用方 `Vec`** - 每个属性列表少一次额外 `Vec` 分配。
- **全面使用 `Vec::with_capacity` 预分配** - 属性列表、子节点列表、方法链均以合理容量提示
  预分配。
- **class 字符串转换使用 `Cow<str>`** - 不含连字符时零拷贝借用；仅在需要 `-` → `_` 替换时
  才分配。

#### 二进制体积优化
- **动态 class match 表提取为 `#[inline(never)]` 局部函数** - 同一组件内多个 `class={expr}`
  现共享同一函数体。LLVM ICF 可跨组件合并相同的单态化实例，减少重复内联导致的代码膨胀。
- **常用 class match 分支 thread_local 缓存** - `generate_common_class_matches()` 在整个
  编译进程中只调用一次（通过 `thread_local! + RefCell<Option<Rc<…>>>`），后续调用通过
  `Rc` 共享，成本为 O(1)。

### ♻️ 代码质量改进
- **合并 `EVENT_HANDLERS` 和 `ATTRIBUTE_NAME_MAP`** 为单一 `lookup_attr_method()` match
  函数，移除冗余的第三元组字段（方法名始终等于 snake_case）。
- **通过 `parse_color_with_method(color, method)` 统一颜色解析** - 移除三个近乎相同的实现
  （text_color / bg / border_color），合并为一个函数。
- **`is_stateful_attr()` 改用 `starts_with` + `match`** - 替换了对 `NEEDS_ID_ATTRS` 和
  `EVENT_HANDLERS` 数组的两阶段线性扫描。
- **删除死代码** - 删除已废弃的 `class_dynamic_value_error()` 诊断函数。
- **修复 `generate_for_loop` 类型约束问题**（`element.rs`）- for 循环多子节点情况下原先使用
  定长数组 `[...]` 传入 `flat_map`，要求所有元素类型相同。改为 `vec![...]` 后可正确支持
  同一 for 循环体中出现不同类型元素（如 `div()` 与自定义组件混用）。
- **`parse_arbitrary_hex` 零堆分配 3 位 hex 展开**（`class.rs`）- 将 3 位 hex（`[#rgb]` →
  `[#rrggbb]`）的展开从字符串分配改为位运算。每个半字节通过 `d << 4 | d` 复制，
  不产生任何堆分配。
- **重构 `border_` 条件判断逻辑**（`class.rs`）- 将"空 if 体 + else if"模式替换为显式的
  `is_directional` 绑定和单一 `if !is_directional` 守卫，消除死分支，意图更直接。
- **`parse_condition_tuple` 更惯用的元组提取**（`parser.rs`）- 将"创建 iterator + 两次
  `.next().expect()`"替换为 `Punctuated::pop()`，直接按索引移除元素，无需构造中间迭代器。

### 📖 文档改进
- **动态 class 限制已明确文档化**（`runtime.rs`）- `generate_dynamic_class_code` 文档注释
  和 `_ => el` 通配分支现均明确说明：运行时仅识别约 58 个预编译的常用 class，不在列表中的
  class 会被**静默忽略**。文档中按优先级列出了推荐替代方案（字符串字面量、条件表达式）。
- **`auto_id` 计数器增量编译稳定性说明**（`element.rs`）- `AUTO_ID_COUNTER` 和 `next_auto_id`
  现已文档化已知限制：计数器在单次编译进程中单调递增，若增量编译改变了宏的展开顺序，相同元素的
  自动 ID 可能发生变化。依赖 ID 稳定性做焦点/状态追踪的元素应显式指定 `id` 属性。

### ✅ 测试
- 全部 236 个测试通过（203 宏测试 + 31 覆盖率测试 + 2 诊断测试）
- 优化改动零回归

### 🔒 安全
- **更新依赖** - 将不再维护的 `proc-macro-error` 替换为 `proc-macro-error2`
  - 解决 RUSTSEC-2024-0370 安全建议
  - 消除重复的 `syn 1.x` 依赖树
  - 完全兼容的 API（无缝替换）

---

## [0.2.0] - 2026-02-17

### ✨ 新增功能

#### 核心特性
- **For 循环语法糖** - 使用 `{for item in items { ... }}` 语法简化列表渲染
- **Styled 标志** - 根据标签名称应用合理的默认样式（h1-h6、button、input 等）
- **条件样式** - 使用 `when` 和 `whenSome` 属性实现动态样式
- **Fragment 支持** - 使用 `<>...</>` 语法返回多个根元素
- **完整 Tailwind 调色板** - 242 种内置颜色（slate、gray、red、blue 等）+ 任意十六进制颜色值
- **全面的事件处理** - 支持 onClick、onMouseDown、onKeyDown、onHover 等 15 种事件类型
- **属性映射** - camelCase 到 snake_case 自动转换（zIndex → z_index，fontSize → font_size 等）

#### 文档系统
- 完整文档体系：快速入门、API 参考、最佳实践、迁移指南、故障排除
- 架构文档（`ARCHITECTURE.md` / `ARCHITECTURE_CN.md`）
- 英文为主要 README；中文版移至 `README_CN.md`

#### 基础设施
- GitHub Actions CI、GPUI 兼容性测试和发布自动化工作流
- Codecov 集成和本地覆盖率脚本

#### 开发者体验
- 使用 `proc-macro-error` 依赖提供更好的错误信息
- 更新仓库 URL 为 `https://github.com/wsafight/gpui-rsx`
- 包含综合测试套件

### 🐛 修复
- **有状态事件的自动 ID 注入** - 补充了 `onHover`/`on_hover`、`onDrag`/`on_drag`、
  `onDrop`/`on_drop` 的有状态属性检测，修复了这些事件处理器在没有显式 `id` 属性时
  编译失败的问题。新增 6 个测试用例验证。

### ♻️ 重构
- **消除子节点解析重复** - 提取 `try_parse_child_node()`，消除 `parse_children()` 和
  `parse_for_loop()` 间的重复逻辑。
- **消除 for 循环代码生成重复** - 提取 `generate_for_loop()`，统一 `map`/`flat_map`
  生成逻辑。
- **简化颜色 class 解析** - 将冗余的 `starts_with()` + `strip_prefix()` 替换为单次
  `strip_prefix()` 调用。

### 🗑️ 移除
- `examples/` 目录（需要外部 GPUI 依赖；功能已由测试覆盖）
- `trybuild` 编译失败测试（简化测试结构）

---

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
