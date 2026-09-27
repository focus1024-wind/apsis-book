---
title: 标准与来源
summary: 各内核所遵循的公开标准与论文，以及引用方式。
---

# 标准与来源

实现的可靠程度取决于其所遵循的来源。工作区中的每个算法都在模块文档中注明来源，本文档把这些引用汇集在一处。

## 为什么来源属于接口的一部分

同一名称的模型，两份实现可能相差数公里。原因通常不是错误，而是选择：

- **引力模型。** SGP4 是针对 WGS 72 拟合的。Space-Track 发布的元素集是为配合 WGS 72 使用而设计的。改用 WGS 84 会使位置偏移数公里，两个结果都不算错，但它们对应不同的拟合。
- **角度的定义。** 纬度可以是大地纬度或地心纬度。二者在中纬度最大相差约 0.19 度，地面距离约 21 公里。
- **时间尺度。** 某时刻的位置只有与该时刻所处的尺度一起才有意义。

注明来源使这些选择可被核查。

## 按主题列出的来源

| 主题 | 来源 | 使用方 |
|------|------|--------|
| SGP4 与 SDP4 | Vallado、Crawford、Hujsak、Kelso，*Revisiting Spacetrack Report #3*，AIAA 2006-6753，2006 | `apsis-sgp4` |
| 两行元素格式 | CelesTrak，*FAQs: Two-Line Element Set Format*，记录原始 NORAD 列位置与校验和规则 | `apsis-tle` |
| 引力模型常数 | WGS 72 取值同 SGP4 参考实现；WGS 84 取自 NIMA TR8350.2 | `apsis-sgp4` |
| 地球自转角 | *IERS Conventions (2010)*，公式 8.32 | `apsis-frames` |
| 大地坐标转换 | NIMA TR8350.2，*Department of Defense World Geodetic System 1984*，第三版，2000 | `apsis-geodesy` |
| 儒略日与约化儒略日 | IAU 约定，见《天文年历解释性补充》 | `apsis-time` |
| 地球时偏差 | IAU 2000 决议 B1.9，定义 `TT - TAI = 32.184 s` | `apsis-time` |
| 闰秒 | IERS Bulletin C，由 `apsis-leap` 以表形式内嵌 | `apsis-leap` |
| 日历算法 | Howard Hinnant，*chrono-Compatible Low-Level Date Algorithms*，2015 | `apsis-time` |
| 离散事件仿真 | Zeigler、Praehofer、Kim，*Theory of Modeling and Simulation*，第二版，2000 | `apsis-sim-engine` |

## 标准与论文的区别

*标准*由权威机构维护。WGS 84、IERS 约定与 IERS Bulletin C 是标准。*论文*描述算法及其验证。SGP4 论文属于论文：算法由论文确定，随论文分发的参考实现是实际操作的描述。

当二者不一致时，本工作区遵循论文并记录差异，因为元素集是拟合到论文模型上的。

## 复现一个已发布的位置

要复现已编目卫星的已发布位置，需要四项一致：

1. 元素集，包括其历元与校验和。
2. 引力模型。除非来源另有说明，使用 WGS 72。
3. 坐标系。SGP4 返回真赤道平春分点坐标系。
4. 时间。传播时间自元素历元量起。

第四项最常出错：元素集中的历元是一个 UTC 时刻，而“传播一天”指的是一天的均匀经过时间，不是一天 UTC 日历日。当区间跨越闰秒插入时，两者相差一秒。见 [[time-scales]] 与 [[leap-seconds]]。

## 另见

- [[sgp4]]：传播模型
- [[geodesy-wgs84]]：椭球
- [[reference-frames]]：坐标系
