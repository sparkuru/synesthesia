# 基础 workspace、配置与领域/协议契约：技术设计

本版沿用已批准的 [父设计](../09-27-synesthesia-mvp/design.md)，范围为 Cargo.toml/Cargo.lock/rust-toolchain.toml、crates/synesthesia-core、crates/synesthesia-protocol、apps 三入口、docs/contracts.md、Linux hako 与根 .gitignore。

## 依赖与边界

无实施子任务前置；最终父规划已于 2026-09-27 获批准。

领域状态、事件身份、revision、语言、音频时间和资源归属由共享契约统一；接口改变需同步消费者与聚焦测试。
新建模块按职责依赖，不能让 client 引入推理/模型或 server 引入桌面。

## 执行与验证

Cargo workspace、三入口、core/protocol、fake adapters 与确定性测试。
本 task 的可观察验收为 AC-04/05/10/11/14/15；client 无推理依赖、server 无桌面依赖；根据当前源码/工具证据固定技术细节并记录本 task research，不将推测写成既有能力。

## C1 技术边界

首轮只建立 core/protocol 与 apps 三入口，不生成空的 audio/inference/desktop 模块冒充完成。
领域层不依赖 GUI、平台采音或真实引擎；codec 使用领域类型，运行时入口不能复制 reducer 或配置解析。
实际 session 生命周期提供可验证状态和 fake effects，真实推理取消/资源回收属于 C2/C4/C6。
识别原文即时发布与显示模式分离；translation-only 不改变 final-only 翻译时机；eviction 水位避免无限存储旧语句身份。
补齐语句身份、非倒退 revision、数值溢出、重复事件、输入切换与迟到事件拒绝规则；协议校验不反射原始载荷或令牌到日志/错误。
配置 schema 版本、PCM 头具体布局、错误码与技术预算在本 task 固定并写入 docs/contracts.md；实现细节允许在批准行为内自主选择。
Linux 当前无 cargo/rustc，但已有 rust:1.90-bookworm 镜像与 Docker daemon；验证前核实镜像工具版本/组件，按 dev-it-in-docker 创建一次性 hako，不创建无实际服务的 dev.sh，不发布端口。
Windows 原生路线不使用 hako/Docker/WSL；原生构建未运行时明确记录缺口，不用 Linux 编译结果外推 Windows 兼容性。

## 失败与回滚

失败给出具体范围/错误/实验输入与结果，不自动退到 CPU、不换模型、不删失败样本。
回滚仅处理本 task 已知创建的文件；保留用户配置/模型、其他子任务和初始 PRD/README 改动。
