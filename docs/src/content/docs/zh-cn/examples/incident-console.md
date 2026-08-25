---
title: 事故响应控制台
description: 一个包含筛选、状态转换和事件时间线的模块化 GPUI-RSX 多面板应用。
---

事故响应控制台是仓库中的完整应用示例。它没有把语法拆成孤立片段，而是模拟实际运维流程：可以筛选、选择、认领、升级事故，推进响应状态，并通过时间线查看处置过程。

![事故响应控制台，包含事故队列、运维指标和所选事故详情](/gpui-rsx/incident-console.png)

## 运行

Demo 使用仓库锁定的 GPUI revision 和 Rust toolchain：

```bash
cargo run --manifest-path demo/Cargo.toml --bin incident_console --locked
```

初始窗口大小为 `1320 x 820`，由稳定的导航栏、事故队列和详情面板组成。

## 交互模型

| 区域 | 行为 |
| --- | --- |
| 范围 | 在进行中、未分配和全部历史事故之间切换。 |
| 严重级别 | 按 Critical、High 或 Medium 筛选当前队列。 |
| 排序 | 在客户影响和最新信号之间切换。 |
| 自动处置 | 控制模拟信号是否自动认领并开始调查。 |
| 生命周期 | 依次推进 Triggered、Investigating、Monitoring 和 Resolved。 |
| 时间线 | 查看所选事故的告警、操作、恢复和备注事件。 |
| 模拟信号 | 无需外部服务即可确定性地注入生产信号。 |

## 源码结构

该示例使用 Cargo 目录式二进制，让入口文件保持精简：

```text
demo/src/bin/incident_console/
├── main.rs         只负责启动窗口
├── domain.rs       事故类型与生命周期规则
├── model.rs        派生状态与应用状态转换
├── sample_data.rs  初始事故和确定性模拟信号
├── view.rs         顶层组合与指标
├── sidebar.rs      范围、自动处置和历史操作
├── queue.rs        筛选器和带 key 的事故行
├── details.rs      概览操作与事件时间线
└── tests.rs        指标、生命周期、模拟和恢复测试
```

这个边界与[最佳实践](/gpui-rsx/zh-cn/guides/best-practices/#按工作流组织视图)中的建议一致：围绕有意义的界面和工作流概念拆分，而不是拆分每一个小元素。

## 派生视图状态

计数、排序、筛选和选择回退统一在 snapshot 中计算。各面板拿到的是可以直接渲染的数据，不会重复业务规则：

```rust
let snapshot = self.snapshot();
let sidebar = sidebar::render(self, &snapshot.stats, cx);
let queue = queue::render(self, &snapshot.visible_incidents, cx);
let details = details::render(self, snapshot.selected.as_ref(), cx);

rsx! {
    <div class="size-full flex bg-neutral-950">
        {sidebar}
        <main class="flex-1 min-w-0">
            {queue}
            {details}
        </main>
    </div>
}
```

## 数据驱动控件

重复控件使用普通 Rust 数据、RSX 循环语法和 `key`，确保 listener 获得稳定身份：

```rust
{for filter in SeverityFilter::OPTIONS.iter() {
    <button
        key={filter.label()}
        class={if view.severity_filter == *filter {
            filter.selected_class()
        } else {
            "px-10 py-6 rounded-md text-zinc-500 cursor-pointer"
        }}
        onClick={cx.listener({
            let filter = *filter;
            move |view, _, _window, cx| {
                view.severity_filter = filter;
                cx.notify();
            }
        })}
    >
        {filter.label()}
    </button>
}}
```

## 状态转换

Listener 会把非简单修改交给命名方法。每次转换都会更新事故、记录时间线事件、更新最近操作，并且只发送一次通知：

```rust
onClick={cx.listener(|view, _, _window, cx| {
    view.advance_selected();
    cx.notify();
})}
```

模型还处理了清理全部已解决事故后的空队列。再次注入信号即可恢复正常流程，不需要重启应用。

使用 `cargo test --manifest-path demo/Cargo.toml --bin incident_console --locked` 可以运行 4 个状态单元测试。

可以浏览[完整源码](https://github.com/wsafight/gpui-rsx/tree/main/demo/src/bin/incident_console)，也可以把这个持续接受编译检查的示例作为 GPUI 桌面应用起点。
