# 历史资料导航

当前产品依据 [PRD v2](../../prd.md)，正式入口为 `apps/synesthesia`；能力与后续顺序见 [状态](../status.md) 和 [路线](../roadmap.md)。本目录及 `deploy/` 保存旧方案，不构成当前发行承诺或继续实施旧任务的授权。

| 资料 | 用途 |
| --- | --- |
| [2026-09-27 PRD](2026-09-27-prd.md) | 从根目录逐字保留的旧三角色产品需求 |
| [旧桌面依赖清单](2026-09-27-desktop.Cargo.toml) | 移除未使用 GUI 依赖前的清单，不能作为现行框架选型依据 |
| [旧 Cargo.lock](2026-09-27-Cargo.lock) | 当时的依赖解析快照，当前构建使用仓库根 Cargo.lock |
| [旧部署骨架](../../deploy/README.md) | 三角色安装与容器草案，不是新产品发行包 |

## 旧 PRD 的原位链接

旧 PRD 原先位于仓库根目录。为逐字保留快照，正文中的相对链接不做改写；从历史目录直接点击以下 5 处链接会失效，请使用本表。现行文档的本地链接仍须有效，只有此表明确列出的快照原位链接属于保留例外。

| 快照行号 | 原相对目标 | 当前可用目标 |
| --- | --- | --- |
| 211、316 | `deploy/README.md` | [部署骨架说明](../../deploy/README.md) |
| 317 | `deploy/PACKAGE-LAYOUT.md` | [旧目录布局](../../deploy/PACKAGE-LAYOUT.md) |
| 318 | `deploy/MENTAL-MODEL.md` | [旧部署模型](../../deploy/MENTAL-MODEL.md) |
| 324 | `deploy/VALIDATION.md` | [骨架验证边界](../../deploy/VALIDATION.md) |

## 旧任务状态

8 个 `09-27-*` 任务已按“计划被取代”移入 `.trellis/tasks/archive/2026-11/`，原父子关系、批准与研究继续保留。`meta.superseded_by` 对应 [1102 工程整理](../../.trellis/tasks/archive/2026-11/11-02-rust-repository/prd.md) 的需求变更起点，`meta.replacement_plan` 指向 [新版交付计划](../../.trellis/tasks/11-02-desktop-caption-pipeline/prd.md)。

归档后的 status=completed 只表示关闭：`meta.archive_reason=superseded`、`acceptance_status=not_claimed` 明确没有宣称验收通过，原 planning/in_progress 和原完成日期保存在 historical 字段中。旧任务已从活动列表移除；已有成果及复用映射见 [计划迁移记录](../../.trellis/tasks/11-02-desktop-caption-pipeline/research/plan-transition.md)。`00-bootstrap-guidelines` 是另行归档的初始化任务，不能与旧产品计划的关闭语义混同。
