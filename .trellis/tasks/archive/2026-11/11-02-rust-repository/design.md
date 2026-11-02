# 设计

音频输入、ASR 和翻译分别配置。STT 产生共享 SourceEvent，翻译只处理定稿并产生关联版本的 TranslationEvent。直接云识别不要求自建 server/client。

core::pipeline 拥有新的 schema 2 桌面配置；core::config 保留 schema 1，避免旧配置被静默赋予新语义。新 apps/synesthesia 目前只提供帮助、版本和结构检查，不伪装实时运行。

core 继续拥有身份、reducer、队列和安全错误。旧 Session/ComputeAdapter 仍绑定本地 ASR+翻译，需要拆分生命周期后才能接入新配置。desktop 仅补齐借用型显示投影，无第二个 reducer；窗口和字体实现另行交付。

相对模型路径按配置文件实际父目录解析；检查不读环境变量值、不联网、不查找/加载模型。旧 v1 路径基准保持不变。保留原型在原处，旧 PRD 与依赖清单留历史副本，不操作私密 archive。

移除 desktop 当前没有实现使用的 GUI 依赖，所有成员继续参加构建。后续真实窗口任务再引入实际使用的框架。原生适配器和历史 GPU/采音证据保留，但不是新产品的端到端验收。

## 补充整理边界

现行能力以 docs/status.md 为唯一状态入口，规范保留执行规则并将 C1/C5 描述明确放入历史上下文。旧 task.json 只追加 meta.superseded_by、superseded_at、historical_status 和历史范围说明，不改变原 status、批准及父子关系；现有列表不会自动隐藏这些任务。

deploy 原地保留，为各文档和复用提示加上旧方案说明，指向历史 PRD 与当前文档；同步文件哈希、ZIP 内容及 ZIP 哈希。历史 PRD 正文不改，用 docs/history/index.md 提供 5 处原位链接的路径映射，并从根 README 和部署说明提供入口。部署包内只携带 deploy 文件，指向仓库外层文档的链接仅用于源码浏览，包内历史说明须能独立理解。
