---
title: 事故响应控制台
description: 一个包含筛选、状态转换和事件时间线的模块化 GPUI-RSX 多面板应用。
---

事故响应控制台模拟运维队列：筛选事故、认领负责人、推进响应状态，再查看事件时间线。各面板共用一份状态模型，使用本地示例数据，无需外部服务。

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

snapshot 保存统计值和排序后的事故 ID，渲染时再从模型查找对应记录。状态转换会同步当前选择与筛选条件，详情面板只展示可见队列中的事故：

```rust
let snapshot = self.snapshot();
let visible_incidents = snapshot.visible_ids.iter()
    .filter_map(|id| self.incidents.iter().find(|incident| incident.id == *id))
    .collect::<Vec<_>>();
let sidebar = sidebar::render(self, &snapshot.stats, cx);
let queue = queue::render(self, &visible_incidents, cx);
let details = details::render(self, self.selected_incident(), cx);

rsx! {
    <div class="size-full flex bg-demo-canvas">
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
            "px-10 py-6 rounded-md text-demo-muted cursor-pointer"
        }}
        onClick={rsx_listener!(cx, notify, {
            let filter = *filter;
            move |view, _, _window, _cx| {
                view.set_severity_filter(filter);
            }
        })}
    >
        {filter.label()}
    </button>
}}
```

## 状态转换

Listener 将事故状态修改交给命名方法。`rsx_listener!(cx, notify, ...)` 在方法返回后发送通知，模型本身不负责通知：

```rust
onClick={rsx_listener!(cx, notify, |view, _, _window, _cx| {
    view.advance_selected();
})}
```

筛选没有结果时，队列提供 **Reset filters**；当前活跃队列为空时，可以用 **Simulate incident** 添加信号。认领或解决事故后，如果记录不再符合当前筛选，选择也会随之更新。

共享配色集中在 `demo/rsx.toml`，各面板通过 `bg-demo-surface`、`text-demo-muted` 等 token 使用。运行 `cargo test --manifest-path demo/Cargo.toml --bin incident_console --locked` 可检查状态行为；较小的 API 示例见[示例索引](/gpui-rsx/zh-cn/examples/demos/)。

可以浏览[完整源码](https://github.com/wsafight/gpui-rsx/tree/main/demo/src/bin/incident_console)，也可以把这个持续接受编译检查的示例作为 GPUI 桌面应用起点。
