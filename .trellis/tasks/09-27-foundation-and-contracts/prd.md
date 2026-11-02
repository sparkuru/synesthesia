# 基础 workspace、配置与领域/协议契约

状态：in_progress。基础依赖交付已通过独立复核、Linux 与原生 Windows 11 最终复测（36 项测试）。审批依据为用户明确要求“开始实现”。设备/推理/UI/网络与 Windows 10 证据由后续 task 继续补齐，不宣称整个 MVP 已验收。

## 目标与价值

Cargo workspace、三入口、core/protocol、fake adapters 与确定性测试。本 task 只交付上述可独立验证的职责，不宣称整个 MVP 已验收。

## 已批准需求依据

- [父任务 PRD](../09-27-synesthesia-mvp/prd.md) 与 [产品 PRD](../../../prd.md) 为范围和行为权威。
- 本 task 负责 Cargo.toml/Cargo.lock/rust-toolchain.toml、crates/synesthesia-core、crates/synesthesia-protocol、apps 三入口、docs/contracts.md、Linux hako 与根 .gitignore。
- 遵守 Windows 10/11 x64 原生、英日手选译简中、单路、无内容持久化、模型手动提供及父任务全部 D 决策。
- 跨模块接口按 [父技术设计](../09-27-synesthesia-mvp/design.md) 共享，不能另建不同字幕/会话语义。

## 明确依赖

无实施子任务前置；最终父规划已于 2026-09-27 获批准。

## 验收标准

- AC-04/05/10/11/14/15；client 无推理依赖、server 无桌面依赖。
- 建立可构建的 core/protocol 与三个产品入口；入口提供真实 help/version/config 校验，未接入的采音、推理、GUI/WS 不能报告 ready 或伪装完整运行。
- 配置按角色强类型、版本化解析并校验：en/ja → zh-Hans、非法 auto 源语言拒绝、client 拒绝推理/model 字段；共享令牌不进入 Debug/错误/例子输出，运行根与模型路径独立于调用者 CWD。
- 单一 reducer 以会话/输入代际/语句/revision 管理草稿替换、final 终态、重复 final、源版本匹配、迟到译文与 3 句有界历史；final 翻译效果只触发一次，旧会话/代际不能写入。
- WS v1 文本消息与二进制 PCM 帧具有完整版本/长度/顺序/样本/会话校验与黄金夹具；未知版本、畸形/超大载荷明确拒绝，不用真实网络或模型验证替代 codec 测试。
- 有界音频/翻译/事件预算及会话停止/重开状态提供确定性 fake adapter 行为测试；fake 只用于测试，不出现在正常产品推理路径。
- 诊断基础采用可审计字段白名单与有界日志，应用/所属引擎总预算 20 MiB；正文、音频、令牌不能进入日志接口。日志轮转、超大记录、重启与不可写失败做针对性验证。
- Rust fmt、check、clippy（warnings as errors）、workspace tests 通过；Linux wrapper 验证仅证明通用代码，不替代 Windows 10/11 原生构建/设备验收。
- 提交可复测的针对性验证、工具版本、实际结果与不可自动验证的证据缺口；没有运行的检查不可报通过。
- 不擅自改模型、放宽发布门槛、缩小 Windows 范围或静默更换计算设备。

## 范围外

- 不实施父计划中其他子任务的职责，不更改 Trellis runtime/平台模板、现有 bootstrap task 或用户模型。
- 不自动 commit/push/archive 或对外部署；任务生命周期及最终发布由主会话管理。

## 技术未知项

依赖/API/设备实验属于本 task 的技术工作；已批准行为不能因选型未知而被替换。若失败，留下证据并报告主会话。
