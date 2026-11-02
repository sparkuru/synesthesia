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

8 个 `09-27-*` 任务保留在 `.trellis/tasks/`，以便追溯父子关系、原批准与原生实验。它们的 `meta.superseded_by` 指向 `11-02-rust-repository`，`meta.historical_status` 记录被新计划取代时的状态。

现有 `task.py list` 不会根据这些自定义字段隐藏任务，仍显示原 planning/in_progress；这不表示当前工作应继续旧方案，也不等于这些任务已验收。`00-bootstrap-guidelines` 是另行按用户要求归档的初始化任务，不能与这些未完成的历史任务混同。
