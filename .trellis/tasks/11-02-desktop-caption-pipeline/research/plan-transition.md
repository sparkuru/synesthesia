# 旧计划关闭与新规划映射

日期：2026-11-02。用户要求处理旧计划整体归档并重新规划，并明确 R1 优先 Windows 11 x64。此次只变更计划/文档/元数据，不实施产品代码。

## 关闭记录

旧任务统一归档到 `.trellis/tasks/archive/2026-11/`，每份 task.json 都包含：

- `status=completed`：沿用 Trellis 归档生命周期，表示关闭，不作产品验收结论。
- `meta.archive_reason=superseded`、`acceptance_status=not_claimed`：关闭原因与非验收语义。
- `meta.historical_status`、`historical_completed_at`：关闭前原状态和原完成日期；原批准及父子关系继续保留。
- `meta.superseded_by=11-02-rust-repository`：原 PRD 变更与 R0 整理起点。
- `meta.replacement_plan=11-02-desktop-caption-pipeline`：新的交付父计划。
- `meta.historical_scope_before_archive`：归档前历史范围原文，新说明解释现在的关闭含义。

按子任务先、父任务后的顺序调用 task.py archive --no-commit --skip-branch-validation。跳过 PR 分支检查是因为旧计划退役，不是声称有 PR 合并；没有修改 Trellis 脚本。归档后按快照恢复了原 parent/children。

## 进度与复用

| 旧任务 | 关闭前状态 | 已有成果与新阶段关系 |
| --- | --- | --- |
| [MVP 父任务](../../archive/2026-11/09-27-synesthesia-mvp/prd.md) | in_progress | 旧三角色计划整体退役，不继承实施批准 |
| [C1 基础契约](../../archive/2026-11/09-27-foundation-and-contracts/research/native-validation.md) | in_progress | 36 项基础测试和 Windows 11 复核；复用身份、reducer、预算、诊断及兼容接口 |
| [C2 本地推理](../../archive/2026-11/09-27-inference-compatibility/research/native-experiments.md) | in_progress | 进程适配、C++ helper、真实 GPU 短样本；R3/R4 拆分复用，不冒充质量/长时完整验收 |
| [C3 系统采音](../../archive/2026-11/09-27-system-audio-capture/research/native-validation.md) | in_progress | WASAPI 回环与 18 项音频测试；规范化可复用，回环在 R2 接线，麦克风需新增证据 |
| [C4 server/WS](../../archive/2026-11/09-27-compute-server-and-ws/implement.md) | planning | 无监听主体，协议组件保留，远程计算为后续扩展 |
| [C5 桌面 UI](../../archive/2026-11/09-27-desktop-overlay/research/native-ui-readiness.md) | in_progress | 只有窗口/字体/DPI 研究；R1 重新验证框架与恢复路径 |
| [C6 集成](../../archive/2026-11/09-27-full-client-integration/implement.md) | planning | 无完整链路，按 R1–R4 形成新集成 |
| [C7 发行](../../archive/2026-11/09-27-packaging-and-docker/implement.md) | planning | 旧部署骨架保留，真正发行归 R5 |

## 新活动计划与验证

仅创建本父任务与 [R1](../../11-02-r1-microphone-fun-asr/prd.md)，均为 planning。R2–R5 继续以根路线表达，到达阶段时才细化任务。没有运行 task.py start、采音、云调用、模型或窗口。

迁移前快照在临时 `/tmp/synesthesia-replan-baseline.json`，用于核对 8 个 task.json 的原批准/依赖/关系可还原，研究/代码不变。Markdown 只调整必要链接，旧产品 PRD 指向逐字保留的历史快照；JSONL 只修复相同语义的路径。Git 历史保留迁移前版本。

最终验证包括归档语义、全部本地链接/上下文、新规划完整性、旧证据保留、产品源码/原历史快照及独立工具更新未变，见 [规划检查](planning-check.md)。
