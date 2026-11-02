# Rust 架构与迁移边界

需求依据：[PRD v2](../prd.md)。当前目标数据流：

~~~mermaid
flowchart LR
    Mic[麦克风] --> Capture[采集与规范化]
    System[系统音频] --> Capture
    Capture --> ASR[识别适配器]
    Local[本地模型] --- ASR
    Cloud[Fun-ASR 等服务] --- ASR
    ASR --> State[共享字幕状态]
    State --> Translate[可选翻译]
    Translate --> State
    State --> Display[透明置顶字幕]
~~~

此图是目标，不表示当前所有环节已接通。

## 模块与所有权

| 位置 | 责任 | 当前边界 |
| --- | --- | --- |
| apps/synesthesia | 正式桌面入口 | 帮助、版本、schema 2 检查 |
| core | 配置、身份、字幕 reducer、预算、诊断 | 平台无关；不访问设备/服务 |
| audio | 采音与规范化 | 既有 Windows 系统回环；麦克风待补 |
| inference | 本地推理进程管理 | 当前仍耦合 ASR 和翻译 |
| desktop | 字幕呈现 | 借用式纯投影；没有原生窗口 |
| protocol | 旧远程 WS/PCM | 不是 Fun-ASR 协议 |
| apps/synesthesia-full/server/client | 历史三角色 | schema 1 检查，运行时未接通 |

crates 中包名均为 synesthesia-*。后续在真实 provider 实现出现时新增相应包，不预建空插件目录。厂商 JSON 在适配边界归一化，界面只消费 core 类型。

## 识别和翻译

新运行时分别管理 ASR 和翻译的准备、失败、停止与退出。云 ASR 不构造本地识别模型；关闭翻译不构造或加载译文引擎。组合不应通过复制整套应用实现。

现有 Session / ComputeAdapter 来自旧方案，仍把本地 ASR 与翻译绑定，不能直接满足新 PRD。后续先提取独立生命周期及翻译关闭状态，再接线；不以空译文适配器伪装就绪。

## 状态与资源

- 延续 InputId、UtteranceId、SourceEvent、TranslationEvent 的身份与采样时间轴。
- 保留一个 core reducer；desktop 的 project(&Subtitle, DisplayMode) 只借用内容，切换显示不触发任务。
- 当前 reducer 将 final 视为终态；Python 允许 final 修订。云适配前须明确修订语义和译文版本，并通过行为测试，不能直接在 UI 修正状态。
- 当前清屏通过高水位拒绝旧句复活。正常 Stop 收尾与故障 Cancel 需要区分；旧调度器的停止即清空语义仍待迁移。
- 沿用 16 kHz、mono、PCM16 和有界队列；新 provider 不能叠加无界缓存。
- 旧 WS v1 的语言限制不覆盖 schema 2；配置可选择某语言不证明模型或服务实际支持。

## 配置

| 版本 | 入口 | 范围 | 相对模型路径基准 |
| --- | --- | --- | --- |
| 2 | synesthesia --check-config FILE | 新桌面管线 | 配置文件实际父目录 |
| 1 | synesthesia-full / synesthesia-server / synesthesia-client --check-config FILE | 历史三角色 | 可执行文件目录或显式 runtime root |

两版本不自动转换。新 schema 不直接送入旧 NativeInference。实际约束见 [配置规范](../.trellis/spec/backend/pipeline.md)。

schema 2 当前表达 zh/en/ja 源语言、本地/Fun-ASR 识别、关闭或本地翻译到 zh-Hans。更多服务与语言要增加真实适配和能力检查，不用任意 URL 假装通用服务支持。

配置加载不会查找、下载或加载模型，不读取密钥值。相对模型路径跟随配置实际目录；链接配置以其实际目标目录为准。

## 依赖与平台

Rust 2024 / 1.90.0；应用和平台无关包禁止 unsafe，FFI 保持在窄原生适配层。成熟 C++ 推理 helper 由 Rust 管理，无 Python 产品依赖。

desktop 目前只有纯呈现。旧清单声明的 egui/eframe 等没有对应窗口实现，已移除未使用依赖，历史清单另存 docs/history；真实窗口任务再引入实际使用的框架。

Linux 复用 hako 开发，不等于 Linux 桌面支持，也不是 Windows 用户的运行前置条件。窗口、采音、驱动和 GPU 需要原生验证。

## 保留与迁移

旧 PRD、任务和 deploy 内容保留历史含义；它们的旧批准和未完成任务不自动成为当前实施计划。Python 原型保留原地作为参照和回退，不复制真实配置或机密。用户私密 archive 不读取、不修改，并从版本控制忽略。
