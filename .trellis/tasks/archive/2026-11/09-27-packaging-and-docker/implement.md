# Windows 原生角色包与可选 Linux CPU 容器：执行清单

## 开始条件

- [full-client-integration](../09-27-full-client-integration/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

- [ ] 前置交付与契约检查通过，主会话激活本 task。
- [ ] 加载本 task PRD/design、implement.jsonl 全部引用及 trellis-before-dev 对应指南。

## 实施与验收

- [ ] 在 deploy 安装/打包/配置/文档/归档清单及 Linux Docker 文件；不自动发布外部服务 范围交付：Windows 10/11 原生软件包/校验工具、角色载荷、许可与 Linux CPU server 镜像。
- [ ] 运行有意义的行为测试和固定工具链的 fmt/check/clippy/test；平台实验另记录真实结果。
- [ ] 对照 AC-12 与 AC-01–03 安装验证/AC-13 已声明 GPU；数值门槛与所有实机门槛通过才正式发布，记录 validation、剩余实机/质量门槛。
- [ ] 主会话安排独立 trellis-check，处理具体问题后复测。

## 交付约束

Windows 不要求 Docker/WSL；Linux 开发命令可按项目 readiness 采用有界 hako。
实际命令与技术决定在源码建立后固定，不把 scaffold 或 fake adapter 当作真实采音/推理完成。
涉及原生设备/GPU/视觉验证时保留证据缺口；自动检查能解决的先运行。发布按父任务 D-10，不能自行降低门槛。
禁止替主会话 commit/push/archive、删除用户状态或开启外部服务。
