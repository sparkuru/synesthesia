# 基础 workspace、配置与领域/协议契约：执行清单

## 开始条件

无实施子任务前置；最终父规划已于 2026-09-27 获批准。

- [x] 前置交付与契约检查通过，主会话激活本 task。
- [x] 加载本 task PRD/design、implement.jsonl 全部引用及 trellis-before-dev 对应指南。

## 实施与验收

- [x] 在 Cargo.toml/Cargo.lock/rust-toolchain.toml、crates/synesthesia-core、crates/synesthesia-protocol、apps 三入口、docs/contracts.md、Linux hako 与根 .gitignore 范围交付：Cargo workspace、三入口、core/protocol、fake adapters 与确定性测试。
- [x] 运行有意义的行为测试和固定工具链的 fmt/check/clippy/test；平台实验另记录真实结果。Linux 全部通过，33 项行为测试；原生 Windows 由主会话另行验证，未以 Linux 代替。
- [x] 对照 AC-04/05/10/11/14/15；client 无推理依赖、server 无桌面依赖，记录 validation、剩余实机/质量门槛。见 [验证证据](research/validation.md) 与 [实际契约](../../../docs/contracts.md)，不代表产品对应 AC 全部已通过。
- [x] 主会话安排独立 trellis-check，四项边界问题已修正；Linux 与原生 Windows 11 最终快照均通过，36 项行为测试。见 [独立复核](research/check.md) 与 [原生复测](research/native-validation.md)。

依赖交付已可用于 C2/C3。整个产品的设备、Windows 10、推理、UI、网络与 D-10 门槛仍由后续 task 验证，本 task 不宣称 MVP 完成。

## 交付约束

Windows 不要求 Docker/WSL；Linux 开发命令可按项目 readiness 采用有界 hako。
实际命令与技术决定在源码建立后固定，不把 scaffold 或 fake adapter 当作真实采音/推理完成。
涉及原生设备/GPU/视觉验证时保留证据缺口；自动检查能解决的先运行。发布按父任务 D-10，不能自行降低门槛。
禁止替主会话 commit/push/archive、删除用户状态或开启外部服务。
