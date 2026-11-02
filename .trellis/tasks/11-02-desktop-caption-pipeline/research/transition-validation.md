# 计划迁移验证

日期：2026-11-02。本轮只关闭旧计划并建立新版规划，用户明确选择 Windows 11 x64 优先。

## 保留断言

以迁移前快照与 Git 历史逐项对照，以下检查已执行并通过：

- 8 个旧任务全部移入 archive/2026-11；均有 `archive_reason=superseded`、`acceptance_status=not_claimed` 及新父计划指向。
- 用 historical 字段还原原 status/completedAt/historical_scope，去除本轮新增关闭字段后，8 份原 task.json 完全一致：原批准、依赖、父子关系和完成程度可完整还原。
- 46 个旧 Markdown 正文一致，只修改搬迁必需的链接；16 份 JSONL 除对应 file 路径外的数据一致；其余 3 个研究脚本/证据文件字节一致。
- 根历史 PRD、旧锁文件/桌面清单、产品源码、配置、构建入口、CI、deploy 均与上一提交一致。没有访问用户根私密 archive。
- 原有 19 个 Trellis 工具更新文件的哈希与本轮开始时一致，没有修改工具或权限策略。

## 规划与静态检查

新父/子任务均保持 planning、implementation_approved=false，没有 task.py start 或运行时实施。父任务只映射阶段与跨阶段验收，唯一详细子任务 R1 具有 PRD、design、implement 和源码/官方协议研究。

已运行 `task.py validate`：父任务 implement/check 各 6 项真实引用通过，R1 各 9 项通过。归档清单、所有本地链接、JSON/文本格式与最终规划收敛检查在收尾再次核对；结果见 [独立规划检查](planning-check.md)。

收尾全量检查通过：97 个 Markdown 文件、255 个本地目标，仅 5 处已映射的根历史 PRD 原位链接例外；22 份 JSONL 共 256 个目标全部存在。新父/子完整规划、双向关系、planning/未批准状态与文本格式通过，无 seed 占位。活动列表只有这两个新计划。

没有重跑 Cargo、Windows 原生设备/窗口、模型、云请求或 Compose：产品代码未变，纯文档/元数据检查不代表新增运行能力。旧官方研究引用属于服务依据，本轮没有实际调用云识别。

人审分类：迁移机械部分为 human-not-needed；R1 最终范围与实施启动仍需在呈现最终计划后由用户确认。没有把规划创建或平台选择记录为实施批准。
