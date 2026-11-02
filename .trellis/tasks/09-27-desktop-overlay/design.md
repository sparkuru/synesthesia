# Windows 设置、托盘与悬浮字幕：技术设计

本版沿用已批准的 [父设计](../09-27-synesthesia-mvp/design.md)，范围为 crates/synesthesia-desktop 与窗口资源；不把 React/Tailwind 引入原生应用。

## 依赖与边界

- [foundation-and-contracts](../09-27-foundation-and-contracts/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。
- [system-audio-capture](../09-27-system-audio-capture/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

领域状态、事件身份、revision、语言、音频时间和资源归属由共享契约统一；接口改变需同步消费者与聚焦测试。
新建模块按职责依赖，不能让 client 引入推理/模型或 server 引入桌面。

## 执行与验证

共用设置/悬浮窗、托盘恢复、三种显示模式、字体/DPI/多屏。
本 task 的可观察验收为 AC-08/09 与 AC-01/03 显示侧；原生窗口及 UUPM 研究/真实平台证据；根据当前源码/工具证据固定技术细节并记录本 task research，不将推测写成既有能力。

## 失败与回滚

失败给出具体范围/错误/实验输入与结果，不自动退到 CPU、不换模型、不删失败样本。
回滚仅处理本 task 已知创建的文件；保留用户配置/模型、其他子任务和初始 PRD/README 改动。
