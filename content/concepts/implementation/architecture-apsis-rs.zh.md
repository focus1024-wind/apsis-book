---
title: crate 架构
tags: 架构, 分层, 模块
summary: apsis crate 的分层结构、每层的模块，以及维持该次序的依赖规则。
---

# crate 架构

`apsis` 是一个 crate，除标准库外没有任何依赖。它的公开表面按七个层组织，每层是 `src` 下的一个目录，并以 crate 模块的形式重导出：

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim
```

模块经由 crate 根到达，因此 `apsis::apsis_sgp4` 是传播器，`apsis::apsis_time` 是时间模型。这些名字保留了各层还是独立 crate 时所承载的主题，因此论文、缺陷报告或本文档早前版本中引用的路径仍然有效。

## 分层规则

规则是：一层可以使用其左侧的各层，不能使用其右侧的层。它带来两个在实践中重要的性质：

- **修改不会向上传播。** 新增一个物理常数或修正一次单位换算只需改动一层，其上的所有层无需修改即可获得该改动。
- **单独理解一层即可。** 阅读 `orbit` 层需要阅读 `physics` 和 `foundation`，不需要了解射频链路或调度。

该规则由库所在仓库的 `scripts/check-layers.sh` 检查，因此引用了右侧层的模块会让检查失败，而不会变成习惯。`apsis_core` 与 `apsis_math` 是规则允许的例外：两者都在 `foundation` 层，任何层都可以使用它们。

## 层与模块

| 层 | 模块 | 职责 |
|----|------|------|
| foundation | `apsis_core`、`apsis_units`、`apsis_math`、`apsis_macros` | 常量、错误、物理量 newtype、线性代数 |
| physics | `apsis_time`、`apsis_leap`、`apsis_coord`、`apsis_geodesy`、`apsis_frames` | 时间尺度、闰秒、坐标系、椭球 |
| orbit | `apsis_tle`、`apsis_sgp4`、`apsis_kepler`、`apsis_ephemeris`、`apsis_maneuver`、`apsis_orbit` | 元素解析、传播、二体运动、插值、机动 |
| rf | `apsis_rf`、`apsis_antenna`、`apsis_doppler`、`apsis_atmosphere`、`apsis_noise`、`apsis_interference`、`apsis_link` | 射频链路与天线 |
| network | `apsis_constellation`、`apsis_coverage`、`apsis_access`、`apsis_topology`、`apsis_isl`、`apsis_routing`、`apsis_handover`、`apsis_mesh` | 星座与网络 |
| ops | `apsis_resource`、`apsis_task`、`apsis_scheduler`、`apsis_conflict`、`apsis_schedule` | 资源、任务、计划 |
| sim | `apsis_sim_engine`、`apsis_scenario`、`apsis_event`、`apsis_output`、`apsis_sim` | 离散事件仿真 |

与所在层同名的模块，例如 `apsis_orbit`，是该层的入口：它重导出该层其他模块所定义的内容，并补上把二者串起来的类型。

## feature

crate 没有默认 feature。每个 feature 启用一层，并连带启用其下各层的 feature。

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

本服务启用 `full`，因为它为每一层编写文档，其接口目录也从全部模块提取。接口部分展示的模块结构图由该目录生成，因此描述的就是本次构建。

## 错误策略

存在两类错误类型。

`apsis_core` 定义了通用 `Error` 与粗略的 `ErrorKind`。它的存在使那些失败没有有趣结构的函数无需自造类型即可报告错误。它刻意**不是**各层的错误类型。

有特定领域失败的层定义自己的错误枚举。元素解析器有 `ElementErrorKind`，变体如 `Checksum` 与 `TooShort`；传播器有 `PropagationErrorKind`，变体如 `InvalidElements` 与 `Decayed`。调用方对变体进行匹配，因此变体名指明了调用方需要区分的具体条件。每个错误还携带可选的详情文本，并实现 `Display` 与 `std::error::Error`。

## 另见

- 本服务以图谱形式展示该结构：`GET /api/graph/modules`
- 接口目录：`GET /api/api`
