# Windows 设置、托盘与悬浮字幕

状态：planning。已批准的父任务分解，尚未开始；审批依据为用户在最终规划摘要后明确要求“开始实现”。

## 目标与价值

共用设置/悬浮窗、托盘恢复、三种显示模式、字体/DPI/多屏。本 task 只交付上述可独立验证的职责，不宣称整个 MVP 已验收。

## 已批准需求依据

- [父任务 PRD](../09-27-synesthesia-mvp/prd.md) 与 [产品 PRD](../../../../../docs/history/2026-09-27-prd.md) 为范围和行为权威。
- 本 task 负责 crates/synesthesia-desktop 与窗口资源；不把 React/Tailwind 引入原生应用。
- 遵守 Windows 10/11 x64 原生、英日手选译简中、单路、无内容持久化、模型手动提供及父任务全部 D 决策。
- 跨模块接口按 [父技术设计](../09-27-synesthesia-mvp/design.md) 共享，不能另建不同字幕/会话语义。

## 明确依赖

- [foundation-and-contracts](../09-27-foundation-and-contracts/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。
- [system-audio-capture](../09-27-system-audio-capture/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

## 验收标准

- AC-08/09 与 AC-01/03 显示侧；原生窗口及 UUPM 研究/真实平台证据。
- 提交可复测的针对性验证、工具版本、实际结果与不可自动验证的证据缺口；没有运行的检查不可报通过。
- 不擅自改模型、放宽发布门槛、缩小 Windows 范围或静默更换计算设备。

## 范围外

- 不实施父计划中其他子任务的职责，不更改 Trellis runtime/平台模板、现有 bootstrap task 或用户模型。
- 不自动 commit/push/archive 或对外部署；任务生命周期及最终发布由主会话管理。

## 技术未知项

依赖/API/设备实验属于本 task 的技术工作；已批准行为不能因选型未知而被替换。若失败，留下证据并报告主会话。
