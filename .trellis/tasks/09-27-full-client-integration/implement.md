# full 本机与 client/server 端到端集成：执行清单

## 开始条件

- [inference-compatibility](../09-27-inference-compatibility/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。
- [system-audio-capture](../09-27-system-audio-capture/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。
- [compute-server-and-ws](../09-27-compute-server-and-ws/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。
- [desktop-overlay](../09-27-desktop-overlay/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

- [ ] 前置交付与契约检查通过，主会话激活本 task。
- [ ] 加载本 task PRD/design、implement.jsonl 全部引用及 trellis-before-dev 对应指南。

## 实施与验收

- [ ] 在 apps/full、apps/client 与端到端测试/必要共享集成；跨模块改动先协调 ownership 范围交付：full/client 产品链路、重连/停止/退出、数据策略及各 60 分钟验收。
- [ ] 运行有意义的行为测试和固定工具链的 fmt/check/clippy/test；平台实验另记录真实结果。
- [ ] 对照 AC-01/03–11/14/15；按 D-10 锁定数值复测，client 无模型，记录 validation、剩余实机/质量门槛。
- [ ] 主会话安排独立 trellis-check，处理具体问题后复测。

## 交付约束

Windows 不要求 Docker/WSL；Linux 开发命令可按项目 readiness 采用有界 hako。
实际命令与技术决定在源码建立后固定，不把 scaffold 或 fake adapter 当作真实采音/推理完成。
涉及原生设备/GPU/视觉验证时保留证据缺口；自动检查能解决的先运行。发布按父任务 D-10，不能自行降低门槛。
禁止替主会话 commit/push/archive、删除用户状态或开启外部服务。
