# 指定模型与 RX 9070 XT 推理兼容：执行清单

## 开始条件

- [foundation-and-contracts](../09-27-foundation-and-contracts/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

- [x] 前置交付与契约检查通过，主会话激活本 task。
- [x] 加载本 task PRD/design、implement.jsonl 全部引用及 trellis-before-dev 对应指南。

## 实施与验收

- [ ] 在 crates/synesthesia-inference、推理实验辅助文件及本 task research；不修改用户模型 范围交付：固定引擎/模型来源，ASR/翻译真实加载、取消/常驻适配与基准报告。
- [ ] 运行有意义的行为测试和固定工具链的 fmt/check/clippy/test；平台实验另记录真实结果。
- [ ] 对照 AC-02/05/06/13/14；两模型实际设备证据及 D-10 数值审阅，记录 validation、剩余实机/质量门槛。
- [ ] 主会话安排独立 trellis-check，处理具体问题后复测。

实现代码、固定源码 patch、两个原生 helper 与 smoke/600s benchmark 输入已建立。
Linux 聚焦 fmt/check/clippy/test/build 全部通过，8 项行为测试通过；实际原生模型、GPU、取消/常驻与完整基准由主会话正在执行，不能勾为已验收。
现有实际证据、边界与剩余门槛见 [native-adapter.md](research/native-adapter.md)。

## 交付约束

Windows 不要求 Docker/WSL；Linux 开发命令可按项目 readiness 采用有界 hako。
实际命令与技术决定在源码建立后固定，不把 scaffold 或 fake adapter 当作真实采音/推理完成。
涉及原生设备/GPU/视觉验证时保留证据缺口；自动检查能解决的先运行。发布按父任务 D-10，不能自行降低门槛。
禁止替主会话 commit/push/archive、删除用户状态或开启外部服务。
