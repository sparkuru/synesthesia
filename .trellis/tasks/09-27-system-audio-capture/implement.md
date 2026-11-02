# Windows 系统播放采音：执行清单

## 开始条件

- [foundation-and-contracts](../09-27-foundation-and-contracts/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

- [x] 前置交付与契约检查通过，主会话激活本 task；C1 checked delivery 36 tests passed Linux/Windows11。
- [x] 加载本 task PRD/design、implement.jsonl 全部引用及 trellis-before-dev 对应指南。

## 实施与验收

- [x] 在 crates/synesthesia-audio 范围交付真实 WASAPI、格式规范化、设备选择/变化失败边界与数字活动诊断；不实施 GUI 或模型推理。
- [x] 固定工具链 Linux focused fmt/check/clippy/test/build 通过，16 项音频测试；主会话 Windows11 早期 v4 原生 check/clippy/test/examples 与受控双代采音通过，最后增加的频谱断言仍需主会话复测。
- [x] [validation](research/validation.md) 记录 AC 输入侧证据及 Windows10/设备切换/最终原生断言剩余门槛；不把未测试场景当作通过。
- [ ] 主会话安排独立 trellis-check，处理具体问题后复测。

## 交付约束

Windows 不要求 Docker/WSL；Linux 开发命令可按项目 readiness 采用有界 hako。
实际命令与技术决定在源码建立后固定，不把 scaffold 或 fake adapter 当作真实采音/推理完成。
涉及原生设备/GPU/视觉验证时保留证据缺口；自动检查能解决的先运行。发布按父任务 D-10，不能自行降低门槛。
禁止替主会话 commit/push/archive、删除用户状态或开启外部服务。
