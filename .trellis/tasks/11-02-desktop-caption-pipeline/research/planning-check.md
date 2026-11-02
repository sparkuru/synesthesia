# 新交付规划独立检查

日期：2026-11-02。范围：父计划与 R1 的六份规划文档、R1 运行研究、上下文清单，以及旧 MVP/七个子任务的 superseded 归档迁移。本次不实施产品代码，也不批准实施。

## Findings (fixed)

无。未发现需要修复的局部规划缺陷；仅新增本检查记录。

## Findings (not fixed)

无阻塞项。Windows 11 x64、麦克风/Fun-ASR/关闭翻译/仅原文、用户显式重新开始及当前窗口控制范围已经明确，没有需要再次询问的产品范围决定。

框架/renderer/MSRV、实际账号端点条件、context/heartbeat 映射、事件时间范围和 Stop/join deadline 均列入获批启动后的早期技术验证门；未验证的具体实现数值或依赖不是新的产品范围授权。任一原型结果若要求改变行为或平台，计划已要求返回规划。最终计划评审仍待用户确认，父任务及 R1 应保持 planning、implementation_approved=false。

## 需求与 PRD 收敛判断

- Requirement convergence：已具备最终评审条件。父任务只协调 R1–R5，唯一详细子任务 R1 的输入、服务、平台、显示、生命周期、隐私及非目标有明确边界；未复活旧三角色、托盘/穿透或固定三句的实施批准。
- PRD convergence：已具备最终评审条件。R1-AC1–AC6 可观察并映射根 PRD，要求真实中文样本识别对照、真实设备/窗口、无本地模型、旧结果隔离、正常有界收尾、错误与资源释放；75 项既有测试和旧原生实验未被冒充新链路验收。
- 设计/实施门：识别独立就绪、Disabled 状态、云发送与旧逐帧 ASR 完成语义分离、空句首/未知结束时间/冲突 final/清屏高水位、采集截止与规范化尾帧、总队列预算和 GUI 非阻塞释放均有对应步骤及验证方式。R2–R5 保留路线，未建立空实现任务或自动串行开发授权。
- 结论：可提交本轮最终规划摘要供用户评审；不能据此 task.py start、运行窗口原型或直接进入产品实现。

## 服务依据复核

官方 [WebSocket 接入指南](https://help.aliyun.com/zh/model-studio/fun-asr-realtime-websocket-api) 支持先等待 task-started、结束上传后继续接收结果直至 task-finished 的设计。[客户端事件](https://help.aliyun.com/zh/model-studio/fun-asr-client-events) 和 [服务端事件](https://help.aliyun.com/zh/model-studio/fun-asr-server-events) 的结构化 context、heartbeat、句子身份、空句首及空结束时间与研究中列出的适配门一致。

可选 workspace 的公共端点来源另由官方 [HarmonyOS Fun-ASR 文档](https://help.aliyun.com/zh/model-studio/harmonyos-sdk-for-fun-asr-real-time-service) 和 [SDK 选型文档](https://help.aliyun.com/en/model-studio/sdk-selection-and-download) 佐证。它们只是地址依据，不证明 Rust 适配或实际账户可用；计划明确固定目标、不自动换域名/地域/模型，并在真实就绪阶段安全报告账户条件错误。复核仅浏览官方文档，未调用识别服务或读取凭据。

## Verification

- 以 `/tmp/synesthesia-replan-baseline.json` 为基准：8 个旧任务均已移至任务归档；completed 仅为关闭，archive_reason=superseded、acceptance_status=not_claimed，原 status/完成日期另存 historical 字段。
- 对比全部旧 task.json：除归档 status/日期及明确新增/替换的历史说明外，原字段、批准、依赖和父子关系可完整保留；原 superseded_by 保留，replacement_plan 指向新父任务。
- 73 个旧文件独立对比结果：39 个字节不变，10 个 Markdown 只修改链接目标，16 个 JSONL 只重映射 file 路径，8 个为上述 task.json 元数据变化。研究脚本及研究正文没有被改写。
- 19 个独立 Trellis 工具更新文件哈希未变；产品源码、配置、CI、部署文件与根历史 PRD 无工作区差异。根历史 PRD 的 5 个原位链接仍为已记录例外，现行链接由主会话做最终全量检查。
- 新父任务只有 R1 一个子任务，双向关系一致；六份 PRD/design/implement 齐全，两任务共四份 JSONL 均有真实目标，均保持 planning 与 pending-final-review。
- 直接核对现行 Session 的强制翻译与 Running-only 结果接收、单帧完成拥有权，CaptureHandle 的 Stop 后拒绝取帧/清队列、Normalizer 的旧 Stop 丢尾及 reducer final 终态，研究描述与源码相符。
- 适用的文本、JSON/JSONL 与 git diff --check 通过。代码 lint/type-check/Cargo/设备/云/窗口检查不适用，本轮未执行；不能从本次规划检查推断运行支持。

迁移的最终链接、快照与元数据验证详见主会话的 transition-validation.md。未改任务生命周期、暂存或提交，未读取用户根私密 archive。
