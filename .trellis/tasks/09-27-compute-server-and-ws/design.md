# 无桌面算力 server 与 WS 会话：技术设计

本版沿用已批准的 [父设计](../09-27-synesthesia-mvp/design.md)，范围为 crates/synesthesia-session、crates/synesthesia-server、apps/server；协议调整需同步其他消费者。

## 依赖与边界

- [foundation-and-contracts](../09-27-foundation-and-contracts/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。
- [inference-compatibility](../09-27-inference-compatibility/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

领域状态、事件身份、revision、语言、音频时间和资源归属由共享契约统一；接口改变需同步消费者与聚焦测试。
新建模块按职责依赖，不能让 client 引入推理/模型或 server 引入桌面。

## 执行与验证

共享令牌握手、单路有界调度、会话控制/隔离、真实推理事件。
本 task 的可观察验收为 AC-02/10/11/14/15；鉴权成功前不收音频、状态/错误可观察；根据当前源码/工具证据固定技术细节并记录本 task research，不将推测写成既有能力。

## 失败与回滚

失败给出具体范围/错误/实验输入与结果，不自动退到 CPU、不换模型、不删失败样本。
回滚仅处理本 task 已知创建的文件；保留用户配置/模型、其他子任务和初始 PRD/README 改动。
