# GPUI / gpui-component 主线升级指南

本文面向 `gpui-rsx` 维护者，记录从当前锁定组合升级到 2026-08-25 上游主线的范围、顺序、验收标准和回退方式。

这不是应用用户的版本迁移指南。用户侧用法仍以 `README.md` 和 `docs/src/content/docs/guides/migration.md` 为准。

## 1. 升级基线

### 当前项目锁定版本

| 依赖 | Revision | 日期 |
| --- | --- | --- |
| GPUI / Zed | `f164afda46188939f76c24aba4099d04423bc356` | 2026-06-09 |
| gpui-component | `8752104289424b7f35045b68a2d394018da48e7e` | 2026-06-09 |

### 本次目标版本

| 依赖 | Revision | 日期 |
| --- | --- | --- |
| GPUI / Zed | `e973593455af18719be22b0455c3f928c6ccc24d` | 2026-08-25 13:57（Asia/Shanghai） |
| gpui-component | `7885c41663c7a6cc68ad0c99b1ba33550f807ff0` | 2026-08-25 |

截至该日期，crates.io 最新发布版仍为：

- `gpui = "0.2.2"`
- `gpui-component = "0.5.1"`

上游 git 主线已经超出这两个发布包的 API。其中 gpui-component 主线清单版本为尚未发布的 `0.5.2`，并新增了 `gpui-base`。

升级判断必须使用 git revision 和 `Cargo.lock` 中的 source，不能只比较 crate version。

## 2. 当前验证结果

升级前的当前锁定组合已经通过：

- 根 crate 单元测试、macro tests、coverage tests 和 trybuild tests；
- 根 crate 与 demo 的 Clippy；
- 6 个升级前真实 GPUI demo bin；
- doctest harness 和 benchmark 编译检查；当前 28 个 doctest 均为 `ignored`，不能视为有效示例覆盖。

目标主线组合已于 2026-08-25 13:57（Asia/Shanghai）重新核对远端 `main` 并验证。临时目录不是可持久复核的证据；可复现信息由本节记录的 revision、Rust 版本、命令和提交后的 CI run 共同提供。

在解除 gpui-component 的 manifest `rev`、按第 5.2 节固定锁文件后执行：

```bash
cargo +1.95.0 check --locked --bins
cargo tree --locked -i gpui
```

结果：Rust 1.95 可以编译全部 6 个升级前 demo，依赖树中只有一份 GPUI。升级实施后新增
`api_contract` bin，当前锁定组合会在 CI 中检查全部 7 个 bin。

这个结果只能证明现有 demo 用法没有被破坏，不能证明新增 GPUI API 已被宏正确分类和覆盖。

## 3. 升级目标

本次升级完成后应满足：

1. demo 锁文件解析到目标 GPUI 和 gpui-component revision，且只有一份 GPUI。
2. 最新 `StatefulInteractiveElement` 方法能够正确触发自动 ID。
3. 新增 camelCase 属性能够映射到对应 snake_case GPUI 方法。
4. `scrollbar_width` 不再被误判为 stateful。
5. 真实 GPUI API 合约测试覆盖自动 ID、参数类型和组件扩展 trait。
6. latest compatibility workflow 同时更新 GPUI 和 gpui-component。
7. 文档中的 revision、ID 规则和性能表述与实现一致。

以下事项不要求在本次升级中完成：

- 为 gpui-component 的每个组件增加专用 RSX 语法；
- 完整实现 Tailwind CSS；
- 将 `gpui-base` 设为 gpui-rsx 的直接运行时依赖；
- 改变现有 `rsx!`、`rsx_strict!` 或 `rsx_permissive!` 的公开返回形状。

## 4. 第一阶段：建立可审查工作状态

当前仓库已有可用 `HEAD`。文档编写时 `main` 指向 `2f232d2`，并存在独立 staged 文档改动。升级前应先审查并保留这些改动，不能用所谓“基线提交”把未知 staged 内容一起提交。

建议步骤：

```bash
git status --short
git diff --cached --stat
git diff --cached --check
# 可选：在确认当前 index 内容后创建升级分支
git switch -c upgrade/latest-gpui
```

如果不创建分支，也必须在每个阶段使用 `git diff` 和 `git diff --cached` 区分既有改动与升级改动。不要把 `target/`、临时参考副本或本机配置加入仓库。

## 5. 第二阶段：更新依赖

### 5.1 清单策略

`demo/Cargo.lock` 是兼容性组合的权威记录。建议让三个 git 依赖使用可更新的 source，并由锁文件固定实际 commit：

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit", "runtime_shaders", "wayland", "x11"] }
gpui-rsx = { path = ".." }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
```

不要将直接 `gpui` 依赖设置为带 `rev` 的 source，同时让 gpui-component 继续依赖 bare Zed git source。Cargo 可能将它们解析成两份不同 source 的 GPUI，最终产生不兼容的类型。

### 5.2 更新锁文件

在 `demo/` 中执行：

```bash
cd demo
cargo update -p gpui -p gpui_platform -p gpui-component
cargo check --locked --bins
cargo tree --locked -i gpui
```

如需精确复现本指南记录的组合，可使用 git package 的 `--precise` 更新能力。必须先固定 gpui-component，再固定 GPUI；固定 gpui-component 时 Cargo 会重新解析它的 bare Zed source，如果把该命令放在最后，会覆盖刚固定的 GPUI revision：

```bash
cargo update -p gpui-component --precise 7885c41663c7a6cc68ad0c99b1ba33550f807ff0
cargo update -p gpui --precise e973593455af18719be22b0455c3f928c6ccc24d
```

验收时必须确认：

- `gpui`、`gpui_platform` 及相关 Zed crates 都来自 `e9735934...`；
- `gpui-component`、`gpui-base`、assets 和 macros 来自 `7885c416...`；
- `cargo tree -i gpui` 只显示一个 `gpui` package ID。

## 6. 第三阶段：补齐 GPUI API 元数据

属性映射、自动 ID 和多参数展开集中在 `src/codegen/tables.rs`。最新 GPUI 有 42 个
`StatefulInteractiveElement` 方法；升级前项目按旧版 32 个方法维护。

### 6.1 新增 stateful 方法

将下列方法加入 stateful 元数据：

| GPUI 方法 | 建议 RSX alias | 属性形状 |
| --- | --- | --- |
| `accessibility_id` | `accessibilityId` | value |
| `aria_description` | `ariaDescription` | value |
| `aria_keyshortcuts` | `ariaKeyShortcuts` | value |
| `aria_active_descendant` | `ariaActiveDescendant` | flag |
| `a11y_synthetic_children` | `a11ySyntheticChildren` | value/closure |
| `aria_numeric_value_step` | `ariaNumericValueStep` | value |
| `aria_value` | `ariaValue` | value |
| `aria_placeholder` | `ariaPlaceholder` | value |
| `restrict_scroll_to_axis` | `restrictScrollToAxis` | flag |
| `external_drag_payload` | `externalDragPayload` | value/closure |

注意事项：

- 前 9 个方法独立使用时都必须让宏先生成 `.id(...)`。
- `external_drag_payload` 按 GPUI 约束必须跟在 `on_drag` 后，并使用相同拖拽值类型；`on_drag` 通常已经触发 ID，但该方法仍应有完整元数据和 alias。
- `aria_active_descendant` 和 `a11y_synthetic_children` 只有在元素同时具有 ID 和 `role`、从而生成 accessibility node 时才有实际效果。宏负责 ID，调用方仍需提供 `role`；文档和真实 contract 必须使用语义有效的组合。
- `accessibility_id` 是暴露给辅助技术的平台标识，不等同于 GPUI `.id(...)`；公开文档应明确两者区别。
- `ariaKeyShortcuts` 的公开拼写确定后要同时更新中英文文档和测试，避免再引入第二种不一致的大小写形式。

### 6.2 修正 false positive

从 stateful 方法集合中移除：

```text
scrollbar_width
```

它是 `Styled` 方法，不需要 `.id()`。修复后应验证：

- 单独使用 `scrollbarWidth` 不生成自动 ID；
- 循环内只使用 `scrollbarWidth` 不要求 `key`；
- `overflowScroll`、`overflowXScroll` 和 `overflowYScroll` 仍然要求 ID。

### 6.3 补充事件 alias

最新 GPUI 新增了 `on_mouse_exit`。现有目标中还有几组事件只支持原始 snake_case，建议一并补齐：

- `onMouseExit` -> `on_mouse_exit`
- `onMousePressure` -> `on_mouse_pressure`
- `captureMousePressure` -> `capture_mouse_pressure`
- `onPinch` -> `on_pinch`
- `capturePinch` -> `capture_pinch`

这些方法属于 `InteractiveElement`，不应触发自动 ID。

## 7. 第四阶段：更新 Styled 和 class 覆盖

最新主线需要重新评估以下 helper：

- `text_ellipsis_start` 已在当前 git GPUI 中存在，项目不应继续按“方法已移除”拒绝 `text-ellipsis-start`；
- 新增 `text_ellipsis_middle`，应决定静态、strict 和动态 class 是否一致支持；
- 新增 `grid_rows_min_content` 和 `grid_rows_max_content`；
- 现有 `grid_cols_min_content` 和 `grid_cols_max_content` 也应补充 camelCase alias；
- 新旧 shadow helper 应用真实 `Vec<BoxShadow>` 参数验证，不能只依赖泛型 mock。

建议 alias：

| RSX 属性 | GPUI 方法 |
| --- | --- |
| `gridColsMinContent` | `grid_cols_min_content` |
| `gridColsMaxContent` | `grid_cols_max_content` |
| `gridRowsMinContent` | `grid_rows_min_content` |
| `gridRowsMaxContent` | `grid_rows_max_content` |
| `textEllipsisStart` | `text_ellipsis_start` |
| `textEllipsisMiddle` | `text_ellipsis_middle` |

如果某个 helper 只支持直接属性、不支持 class，必须在 API 文档中明确说明，不能让 permissive、strict 和动态路径无意间表现不同。

## 8. 第五阶段：单元测试与真实 API 合约

测试必须分层。根 crate 单元测试负责快速验证解析、映射和展开，demo contract 负责验证真实 GPUI 类型。两层缺一不可：只使用真实 GPUI 会让反馈过慢，只使用 mock 又会漏掉 `Div`/`Stateful<Div>` 和参数类型错误。

### 8.1 测试层级与职责

| 层级 | 位置 | 主要职责 | 是否依赖真实 GPUI |
| --- | --- | --- | --- |
| 表级单元测试 | `src/codegen/tables.rs` | alias、stateful、multi-arg 元数据 | 否 |
| 宏展开测试 | `tests/macro_tests.rs` | 方法名、自动 ID、调用顺序、参数展开 | 否 |
| 行为测试 | `tests/coverage_tests.rs` | 条件、循环、key、动态 class 行为 | 否 |
| 诊断测试 | `tests/ui`、`tests/pass` | compile-fail/pass 与错误信息 | 否 |
| 真实 API contract | `demo/src/bin` | GPUI 类型、签名、extension traits、组件构造器 | 是 |
| latest workflow | GitHub Actions | 上游 source 漂移和跨平台构建 | 是 |

不要用“测试总数增加”作为验收标准。每个风险点必须有能在错误实现下失败的断言。

### 8.2 拆分 stateful mock 类型

当前 `MockElement` 同时模拟 `Div` 和 `Stateful<Div>`，而且 `.id()` 仍返回 `Self`。这会让漏掉自动 ID 的代码继续通过编译。为避免一次性改写所有既有 mock 测试，优先在独立 type-contract 测试模块中引入严格的两类型 mock；只有确认共享测试可以平滑迁移时，才替换全局 `tests/common` mock。

建议重构为：

```rust
struct MockElement;
struct MockStatefulElement;

impl MockElement {
    fn id(self, id: impl Into<MockElementId>) -> MockStatefulElement {
        capture_id(id);
        MockStatefulElement
    }
}
```

方法实现边界：

- `Styled` 和 `InteractiveElement` 对应方法在两个 mock 类型上都可用；
- `StatefulInteractiveElement` 对应方法只在 `MockStatefulElement` 上实现；
- `MockElement::id` 是进入 stateful 方法集合的唯一入口；
- ID capture 继续记录自动 ID，但不要通过返回同一类型弱化类型检查。

这样 `<div ariaDescription={...} />` 只有在展开结果包含 `.id(...)` 时才能通过 macro test 编译。

对参数类型敏感的方法不要全部使用 `fn method<T>(...)` 泛型占位。至少为下列方法建立接近真实 API 的测试签名：

- `shadow(Vec<MockBoxShadow>)`；
- `role(MockRole)`；
- `aria_numeric_value(f64)` 与 `aria_numeric_value_step(f64)`；
- `aria_level/row_index/column_index(usize)`；
- `on_mouse_down(MockMouseButton, handler)`；
- `on_a11y_action(MockAccessibleAction, handler)`；
- `on_drag(value, constructor)`。

mock 可记录调用序列，例如 `Vec<&'static str>`，用于断言 `.id()` 出现在第一个 stateful 方法之前。

### 8.3 表级单元测试

在 `src/codegen/tables.rs` 的 `#[cfg(test)]` 模块增加数据驱动测试。建议每个 case 至少包含：

```text
(RSX 属性名, GPUI 方法名, needs_id, multi_arg)
```

必须覆盖：

1. 10 个新增 stateful snake_case 方法全部返回 `needs_id = true`。
2. 10 个新增 camelCase alias 映射到预期 GPUI 方法。
3. `external_drag_payload` 为 stateful，但不是 multi-arg tuple 方法。
4. `onMouseExit`、mouse pressure、pinch 及 capture 事件不需要 ID。
5. `scrollbar_width` 和 `scrollbarWidth` 均不需要 ID。
6. `group_active`、`on_a11y_action`、`on_drag` 继续同时标记为 stateful 和 multi-arg。
7. `group_hover`、mouse down/up 和 boxed action 继续只标记为 multi-arg。
8. 未知属性保持透传，不被误判为 stateful。

建议增加一份带目标 revision 的 fixture，例如：

```text
tests/fixtures/gpui_stateful_methods_e973593.txt
```

fixture 应列出最新 42 个 stateful 方法。单元测试逐项调用 `is_stateful_method`，使上游 API snapshot 在代码审查中可见。不要让测试依赖本机 Cargo checkout 路径或运行时网络。

固定 fixture 只能保护已知目标 revision，不能发现未来上游新增方法。latest workflow 应额外运行一个本地检查工具：从 Cargo 实际解析到的 GPUI source 使用 Rust parser 提取 trait 方法，与 fixture 做集合比较。新增或删除方法都应产生可审查的失败，而不是等待现有 demo 偶然编译失败。

### 8.4 宏展开单元测试

在 `tests/macro_tests.rs` 使用 `rsx_expand!` 做字符串级断言。每个新增 stateful 属性必须单独放在一个元素上，不能把 10 个属性合并到同一元素，否则第一个正确属性生成的 ID 会掩盖其他错误。

每个新增属性至少有以下 case：

- snake_case 属性生成 `.id(...).snake_case_method(...)`；
- camelCase 属性生成相同 snake_case 方法；
- 显式 `id` 时只生成一次 `.id(...)`；
- flag 属性生成零参数调用；
- value 属性只求值一次；
- 自动 ID 位于 stateful 方法之前。

重点回归断言：

- `accessibilityId` -> `.accessibility_id(...)` 且包含 `.id(...)`；
- `ariaActiveDescendant` -> `.aria_active_descendant()`；
- `a11ySyntheticChildren` -> `.a11y_synthetic_children(closure)`；
- `restrictScrollToAxis` -> `.restrict_scroll_to_axis()`；
- `scrollbarWidth` -> `.scrollbar_width(...)` 且不包含 `.id(...)`；
- `onMouseExit` -> `.on_mouse_exit(...)` 且不包含 `.id(...)`；
- `externalDragPayload` 在合法的 `onDrag` 属性之后保持相同调用顺序。

调用顺序测试只验证宏保留合法的用户属性顺序，不应暗示宏会自动把写在 `onDrag` 前面的 `externalDragPayload` 重排到后面。该上游约束需要在文档中明确。

### 8.5 行为和 class 单元测试

在 `tests/coverage_tests.rs` 增加以下回归：

- 循环内新 stateful 属性配合动态 `key` 生成不同 ID；
- 字面量 `key` 继续使用静态 `concat!` 路径；
- 显式 `id` 优先于 `key` 和自动 ID；
- 非 stateful 的 `scrollbarWidth` 不消费 `key`；
- `text-ellipsis-start`、`text-ellipsis-middle` 在选定的 class mode 中行为一致；
- grid min/max content alias 使用正确参数；
- 动态 class 未知值在 permissive/strict 模式保持原有策略。

动态 class 测试应使用带可观察字段的非零大小 mock，断言最终样式状态，而不只是“代码没有 panic”。

### 8.6 trybuild 诊断测试

在 `tests/ui` 和 `tests/pass` 增加：

- 循环内使用 `ariaDescription` 但缺少 `id`/`key`：compile-fail；
- 循环内使用 `restrictScrollToAxis` 但缺少 `id`/`key`：compile-fail；
- 为上述元素添加 `key`：compile-pass；
- 循环内只使用 `scrollbarWidth`：compile-pass；
- 新 camelCase alias 的最小调用：compile-pass；
- `externalDragPayload` 的受支持写法：compile-pass。

`.stderr` 应断言稳定、可操作的错误主体，避免绑定不必要的编译器噪音和绝对路径。

如果决定由宏诊断 `externalDragPayload` 缺少或早于 `onDrag`，需要单独的 compile-fail case；如果不做宏级语义检查，则必须依靠文档和真实 contract 覆盖合法顺序。

### 8.7 真实 GPUI 合约 bin

扩展 `demo/src/bin/api_surface.rs`，或者新建只用于编译的 `demo/src/bin/api_contract.rs`。至少覆盖：

- 新增 10 个 stateful 方法的 snake_case 与 camelCase 路径；
- 自动 ID 和显式 ID 两种形式；
- `onMouseExit`、mouse pressure 和 pinch 事件；
- `scrollbarWidth` 不需要 ID；
- `text_ellipsis_start/middle`；
- grid min/max content helpers；
- `Vec<BoxShadow>` 的真实参数类型。

对不适合放在同一条方法链上的 API 使用独立函数，避免一个已有 stateful 属性掩盖另一个属性漏掉自动 ID 的问题。

真实 contract 应参与普通 CI 的 `cargo check --manifest-path demo/Cargo.toml --bins --locked`，也必须参与 latest compatibility workflow 更新锁文件后的非 `--locked` 构建。

### 8.8 分层运行命令

开发时按反馈速度运行：

```bash
# 表级单元测试
cargo test --lib codegen::tables::tests

# 宏展开和行为测试
cargo test --test macro_tests
cargo test --test coverage_tests

# trybuild compile-fail/pass
cargo test --test diagnostic_tests

# 根 crate 全量测试
cargo test --all-features

# 真实 GPUI / component 类型检查
cargo check --manifest-path demo/Cargo.toml --bins --locked
```

单元测试阶段的完成标准：

- [x] 新增或修改的每条 API 元数据至少有一个直接表级断言。
- [x] 每个新增 stateful 方法有独立自动 ID 展开测试。
- [x] 每个新增 alias 同时覆盖方法名和 stateful 分类。
- [x] 至少一个测试能在漏掉 `.id()` 时产生类型错误，而非只检查字符串。
- [x] `scrollbarWidth` false positive 有循环和非循环回归测试。
- [x] 高风险参数不再只由无约束泛型 mock 验证。
- [x] trybuild 错误快照与真实 API contract 均通过。

## 9. 第六阶段：扩展 gpui-component 合约

gpui-rsx 对组件的支持是通用 builder 组合，不需要为 gpui-component 的 57 个公开模块分别增加宏代码。但当前 demo 只覆盖 `Label` 和 `Button`，不足以保护不同构造器形状。

建议在 component demo 中覆盖以下代表性模式：

1. `Button::new(id)`：需要 ID 的构造器。
2. `Label::new(text)`：普通值构造器。
3. Input、Textarea 或 Editor：依赖 state/entity 的组件。
4. Popover、Dialog 或 HoverCard：带 closure content builder 的组件。
5. Tabs、Table 或列表组件：带 children/iterator 的组件。
6. `Sizable`、variant trait、`StyledExt`：extension trait 方法。
7. `gpui-base` 中的一个基础组件或 re-export：验证最新拆分后的导入路径。

验收重点不是运行所有组件，而是确保 `base={...}`、路径 tag、flag/value 属性、children 和扩展 trait 这几种方法链形状都能通过真实类型检查。

## 10. 第七阶段：修复自动更新工作流

### 10.1 latest compatibility workflow

`.github/workflows/gpui-compatibility.yml` 应调整为：

1. 同时更新 GPUI、gpui_platform 和 gpui-component 的可变 git source。
2. 分别记录 GPUI 与 gpui-component 的旧、新 source。
3. 用 commit 短 hash 标识 issue；两者的 crate version 在较长时间内可能不变。
4. 编译所有真实 API contract bins。
5. 至少运行 `cargo check` 和 `cargo clippy`；根 crate tests 只能作为补充。
6. 明确声明创建/关闭 issue 所需的 `issues: write` 权限。
7. 将“只有一个 GPUI package ID”实现为无条件失败门禁；`cargo tree -d` 和 `cargo tree -i gpui` 作为诊断输出，不能只靠人工查看。
8. 从实际解析的 GPUI source 提取 `StatefulInteractiveElement` 方法并与 fixture 比较，发现不破坏编译的新增 API。
9. matrix job 只负责构建并上传结果，由单一汇总 job 创建或关闭 issue，避免一个平台成功就关闭另一个平台的失败 issue。

工作流不能继续依赖固定 `rev` 的 gpui-component，然后把 `cargo update gpui-component` 当作“最新 component”检查。

### 10.2 Dependabot

在 `.github/dependabot.yml` 增加 Cargo `/demo` 目录配置。根 crate 没有 GPUI 依赖，只扫描 `/` 无法维护真实兼容性组合。

Dependabot 更新后仍需由 demo 的 `--locked` 构建和单 GPUI source 检查把关。

## 11. Rust 版本策略

根 proc-macro crate 没有声明 `rust-version`，CI 也没有独立 MSRV lane。升级时需要明确区分：

- gpui-rsx 宏本身的 MSRV；
- demo/目标 GPUI 组合的最低已测试 Rust；
- 上游 Zed 仓库当前默认 toolchain。

本次目标组合已经在 Rust 1.95 编译通过，但目标 Zed revision 的仓库 toolchain 为 1.97.1。这不等同于上游正式承诺 Rust 1.95 MSRV。

建议：

- 在根 `Cargo.toml` 声明经过验证的宏 crate `rust-version`；
- CI 增加该版本的根 crate lane；
- demo 同时测试 `1.95.0` 和 stable，或明确提升 demo toolchain；
- `scripts/check.sh` 显式调用目标 toolchain，不要依赖从根目录执行时自动采用 `demo/rust-toolchain.toml`。

## 12. 文档同步

代码合并前同步更新：

- `README.md` / `README_CN.md`；
- `docs/src/content/docs/compatibility.md` 及中文版；
- IDs、API reference、syntax 和 gpui-component 指南；
- `CHANGELOG.md` / `CHANGELOG_CN.md`。

必须修正的内容：

- 更新 GPUI 和 gpui-component revision；
- 从 stateful ID 列表移除 `scrollbarWidth`；
- 加入新增 accessibility、scroll 和 drag 属性；
- 说明 `gpui-base` 拆分和 facade re-export；
- 将 “Zero Runtime Overhead” 收窄为静态 RSX 路径，明确动态 class matcher 和动态 key 分配；
- 清理仍声称 `text_ellipsis_start` 不存在的说明。

## 13. Benchmark 调整

当前 benchmark 使用零大小 mock，builder 调用没有可观察状态，结果也没有参与后续计算。动态输入已部分使用 `black_box`，但最终 builder 结果未被消费，优化器仍可能删除待测逻辑。

升级时至少完成：

- 使用包含可观察字段的非零大小 mock；
- 对输入和最终结果使用 `black_box`；
- 单独测量静态 class 构造与动态 matcher；
- 不把运行时 Criterion benchmark 当作过程宏展开性能数据；
- 增加一个独立 sample crate，通过 `cargo build --timings` 观察冷编译和增量编译成本。

在修正 benchmark 前，不应根据现有数字宣传具体的零成本或 O(1) 性能结论。

## 14. 完整验收命令

在仓库根目录执行：

```bash
cargo fmt --all -- --check
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings

cargo check --manifest-path demo/Cargo.toml --bins --locked
cargo clippy --manifest-path demo/Cargo.toml --bins --locked -- -D warnings
cargo tree --manifest-path demo/Cargo.toml --locked -i gpui
cargo tree --manifest-path demo/Cargo.toml --locked -d

cargo bench --bench class_performance --no-run
cargo report future-incompatibilities
(cd demo && cargo report future-incompatibilities)
```

发布前再执行：

```bash
scripts/check.sh --release
cargo package --list --allow-dirty
```

`scripts/check.sh --release` 已包含 `cargo publish --dry-run --allow-dirty`，无需重复执行。

最终验收清单：

- [x] 目标 revision 与本指南一致，或已在变更说明中记录更新后的 revision。
- [x] demo 依赖树中只有一份 GPUI。
- [x] 42 个最新 stateful 方法已逐项分类。
- [x] 新增 API 同时有 metadata test、expansion test 和真实 GPUI compile contract。
- [x] `scrollbarWidth` 不注入 ID，不在循环中误报缺少 key。
- [x] latest workflow 真正更新两套上游。
- [x] gpui-component 的代表性构造器和 extension traits 已覆盖。
- [x] Rust 版本策略、README、双语文档和 changelog 已同步。
- [x] benchmark 不再依赖可被完全优化掉的零大小 mock。

## 15. 回退方案

升级应拆成可独立回退的提交，建议顺序：

1. 审查并保留既有 staged 文档改动；
2. 更新 API 元数据和测试；
3. 更新 demo 依赖与锁文件；
4. 扩展 gpui-component contract；
5. 修改 CI / Dependabot；
6. 更新文档和 benchmark。

若上游升级导致无法在计划周期内解决的问题：

1. 回退 `demo/Cargo.toml` 和 `demo/Cargo.lock` 到上一组合；
2. 保留向后兼容的 alias 和测试基础设施改进；
3. 在 compatibility 文档中记录阻塞的上游 revision 和错误；
4. 不要通过引入第二份 GPUI source 临时绕过类型错误；
5. 在独立分支或保留的参考副本中继续跟踪主线。

依赖回退后重新运行完整验收命令，确保锁文件、文档和实际构建重新一致。
