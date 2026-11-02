# Windows 原生角色包与可选 Linux CPU 容器：技术设计

本版沿用已批准的 [父设计](../09-27-synesthesia-mvp/design.md)，范围为 deploy 安装/打包/配置/文档/归档清单及 Linux Docker 文件；不自动发布外部服务。

## 依赖与边界

- [full-client-integration](../09-27-full-client-integration/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

领域状态、事件身份、revision、语言、音频时间和资源归属由共享契约统一；接口改变需同步消费者与聚焦测试。
新建模块按职责依赖，不能让 client 引入推理/模型或 server 引入桌面。

## 执行与验证

Windows 10/11 原生软件包/校验工具、角色载荷、许可与 Linux CPU server 镜像。
本 task 的可观察验收为 AC-12 与 AC-01–03 安装验证/AC-13 已声明 GPU；数值门槛与所有实机门槛通过才正式发布；根据当前源码/工具证据固定技术细节并记录本 task research，不将推测写成既有能力。

## 失败与回滚

失败给出具体范围/错误/实验输入与结果，不自动退到 CPU、不换模型、不删失败样本。
回滚仅处理本 task 已知创建的文件；保留用户配置/模型、其他子任务和初始 PRD/README 改动。
