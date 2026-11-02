# 工程状态

更新：2026-11-02。目标以 [PRD](../prd.md) 为准。当前为 Rust 工程基础，未整合成真实采音、识别与透明字幕的完整应用。

| 能力 | 当前实现 | 验证边界 |
| --- | --- | --- |
| 正式 synesthesia 入口 | 帮助、版本、schema 2 检查 | 不访问密钥、设备、模型或服务 |
| 独立选择输入/ASR/翻译 | 配置类型、校验和示例 | 不证明运行时组合已接通 |
| 字幕身份、状态和预算 | 既有 core/reducer/队列 | 旧调度仍需拆分及正常收尾 |
| 字幕呈现 | 借用型纯投影 | 无原生窗口、字体或托盘 |
| 系统音频 | 既有 Windows WASAPI | 历史独立验证，未接新入口 |
| 本地 ASR/翻译 | Rust 进程适配器和 C++ helper | 两模型仍耦合，历史短样本非完整验收 |
| Fun-ASR | 配置与 Python 原型依据 | Rust 运行时适配待实现 |
| 麦克风 | 配置与原型体验 | Rust 采集待实现 |
| 远程计算 | 旧 WS/PCM 编解码与角色配置 | 无实际网络监听与完整客户端 |
| 打包安装 | 历史 deploy 骨架 | 不是新发行包 |
| Linux/macOS 桌面 | 尚未接入 | Linux 编译不算原生支持 |

工程基础检查见 [原验证记录](../.trellis/tasks/archive/2026-11/11-02-rust-repository/research/validation.md)，本次提交前重新执行的全工程检查见 [提交验证](../.trellis/tasks/archive/2026-11/11-02-rust-repository/research/commit-validation.md)。CI 是否通过以实际 GitHub 执行记录为准，本地检查不等于远端工作流已运行。

## 保留的历史证据

- [基础契约 Windows 11 验证](../.trellis/tasks/archive/2026-11/09-27-foundation-and-contracts/research/native-validation.md)。
- [系统采音 Windows 11 验证](../.trellis/tasks/archive/2026-11/09-27-system-audio-capture/research/native-validation.md)：受控双频、两代输入、过载和停止；未替代 Windows 10 或完整产品验收。
- [本地模型短样本实验](../.trellis/tasks/archive/2026-11/09-27-inference-compatibility/research/native-experiments.md)：真实模型和 GPU，不是全部质量、长时和取消阶段验收。

本轮没有远程复验。旧任务状态和批准保留历史含义，新工作以当前 PRD 和路线为准。

8 个 September 任务已按 superseded 关闭归档，保留原进度、批准和父子关系；归档 status=completed 不表示产品验收通过，关闭原因与原状态另有明确字段。旧需求的链接映射及资料入口见 [历史资料导航](history/index.md)。

当前活动规划为 [PRD v2 交付父计划](../.trellis/tasks/11-02-desktop-caption-pipeline/prd.md) 及 [R1 麦克风/Fun-ASR/原文字幕](../.trellis/tasks/11-02-r1-microphone-fun-asr/prd.md)，均处于 planning。用户已确认 Windows 11 x64 优先；本次尚未开始运行时实现或验收，以上能力表不变。
