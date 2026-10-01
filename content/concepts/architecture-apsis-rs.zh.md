---
title: 工作区架构
tags: 架构, 分层, crate
summary: apsis-rs 的分层结构、每层的 crate 以及依赖规则。
---

# 工作区架构

`apsis-rs` 是一个 Cargo workspace，其成员 crate 按层分组。分组由规则强制，而非仅靠约定：某一层的 crate 可以依赖其左侧的层，不能依赖其右侧的层。对外发布的门面 crate `apsis-rs` 重导出各层，并把每一层变成一个 Cargo feature。

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim
```

## 分层规则

该规则带来两个在实践中重要的性质：

- **修改不会向上传播。** 新增一个物理常数或修正一次单位换算只需改动一层，其上的所有层无需修改即可获得该改动。
- **单独理解一层即可。** 阅读 `orbit` 层需要阅读 `physics` 和 `foundation`，不需要了解射频链路或调度。

每层的 crate 显式声明其依赖，因此规则在每份清单中都可见，违反规则会导致构建失败，而不会变成习惯。

## 层与 crate

| 层 | crate | 职责 |
|----|-------|------|
| foundation | `apsis-core`、`apsis-units`、`apsis-math`、`apsis-macros` | 常量、错误、物理量 newtype、线性代数 |
| physics | `apsis-time`、`apsis-leap`、`apsis-coord`、`apsis-geodesy`、`apsis-frames` | 时间尺度、闰秒、坐标系、椭球 |
| orbit | `apsis-tle`、`apsis-sgp4`、`apsis-kepler`、`apsis-ephemeris`、`apsis-maneuver`、`apsis-orbit` | 元素解析、传播、二体运动、插值、机动 |
| rf | `apsis-rf`、`apsis-antenna`、`apsis-doppler`、`apsis-atmosphere`、`apsis-noise`、`apsis-interference`、`apsis-link` | 射频链路与天线 |
| network | `apsis-constellation`、`apsis-coverage`、`apsis-access`、`apsis-topology`、`apsis-isl`、`apsis-routing`、`apsis-handover`、`apsis-mesh` | 星座与网络 |
| ops | `apsis-resource`、`apsis-task`、`apsis-scheduler`、`apsis-conflict`、`apsis-schedule` | 资源、任务、计划 |
| sim | `apsis-sim-engine`、`apsis-scenario`、`apsis-event`、`apsis-output`、`apsis-sim` | 离散事件仿真 |

## 门面 crate 的 feature

门面 crate 没有默认 feature。每个 feature 启用一层，并连带启用其下各层的 feature。

| feature | 启用 |
|---------|------|
| `foundation` | foundation |
| `physics` | foundation、physics |
| `orbit` | foundation、physics、orbit |
| `rf` | 以上全部，加上 rf |
| `network` | 以上全部，加上 network |
| `ops` | 以上全部，加上 ops |
| `sim` | 全部层，包括 sim |
| `full` | `sim` 的别名 |

本服务基于 `orbit` 构建，这是包含元素、传播、时间与几何内核的最小 feature 集合。接口部分展示的 crate 结构图由接口目录生成，因此描述的是本次构建启用的集合，而不是整个 workspace。

## 错误策略

存在两类错误类型。

`apsis-core` 定义了通用 `Error` 与粗略的 `ErrorKind`。它的存在使那些失败没有有趣结构的函数无需自造类型即可报告错误。它刻意**不是**各层的错误类型。

有特定领域失败的层定义自己的错误枚举。元素解析器有 `ElementErrorKind`，变体如 `Checksum` 与 `TooShort`；传播器有 `PropagationErrorKind`，变体如 `InvalidElements` 与 `Decayed`。调用方对变体进行匹配，因此变体名指明了调用方需要区分的具体条件。每个错误还携带可选的详情文本，并实现 `Display` 与 `std::error::Error`。

## 另见

- 本服务以图谱形式展示该结构：`GET /api/graph/crates`
- 接口目录：`GET /api/api`
