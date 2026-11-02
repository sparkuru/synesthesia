# 当前项目整理评估

日期：2026-11-02。按用户要求接在 1102 下，不新建 task。本轮交付评估与建议，未实施以下整理，也未恢复旧任务或修改产品代码。

后续进展：用户已要求开始整理，以下保留整理前的评估依据；实施与验证结果见 [补充整理验证](cleanup-validation.md)。

## 结论与范围

需要一轮轻量整理，优先消除现行入口、历史方案和任务状态之间的歧义。现有 Rust workspace 的包边界与当前规模匹配，没有发现必须通过搬目录、合并 crate 或预建插件框架解决的问题。

本次采用 repo-audit 的架构与演进模块，检查根需求/入口、Cargo 清单、core 调度与 inference 适配边界、desktop 投影、CI、部署文档、Trellis 规范与任务记录。没有审计完整算法实现、依赖漏洞或外部服务契约。

## 架构现状

- `apps/synesthesia` 是 schema 2 的正式入口，目前仅提供帮助、版本及配置检查；四个应用均只依赖 core。
- core 拥有配置、字幕身份/reducer、调度、预算与诊断；protocol、audio、inference、desktop 各自依赖 core，core 不依赖这些适配层。
- `docs/architecture.md`、`docs/status.md`、`docs/roadmap.md` 已区分目标、已实现能力及历史证据。架构资料存在，问题主要是旁支资料仍复制旧状态。
- schema 1 / WS v1、本地双模型适配和既有 WASAPI 保留复用价值；新管线尚未接通。这是已知迁移边界，不是搬目录可以解决的缺口。

## 待整理项（按优先级）

### 1. medium：验证规范的现行章节仍复制旧状态

证据：`.trellis/spec/trellis-plus/validation.md:13` 的 Current Surface、`:27` 的“no CI workflow”、`:103` 的“No ... CI is established”，以及 `development.md:17` 的“no capture, inference ... or CI”，与 `.github/workflows/rust.yml`、正式入口和既有原生适配器不一致。顶部虽有历史边界说明，但现行章节仍使用当前时态。`frontend/native-ui.md:38` 的 Required Evidence 仍要求完成旧 C5 范围。

影响：后续实现/检查可能忽略已有 CI，或把旧托盘、穿透、双语默认等范围重新当成当前交付要求。

最小修改：现行能力统一引用 `docs/status.md` 与本任务验证记录；历史 C1/C5 内容放入明确的历史小节。现行 UI 验收依据 PRD v2 和对应获批工作范围。保留 `backend/core-boundaries.md` 对旧代码行为的准确描述，不把待迁移行为改写成已经实现。

验证：逐项对照 Cargo、CI、源码和 PRD；区分 CI 文件存在、本地检查通过和远端 CI 实际执行；文档改动只跑链接、内容与空白检查。

### 2. medium：旧任务没有反向标注已被新产品计划取代

证据：8 个 `09-27-*` task.json 仍为 planning/in_progress；父任务和 desktop 子任务保留旧批准及 implementing 信息，均无 `meta.superseded_by`。1102 仅单向记录 `supersedes_product_plan`，`task.py list` 仍将全部旧任务列在活动列表中。

影响：直接读取任务清单或旧 task 上下文的会话可能继续旧三角色方案。根 AGENTS 已禁止这样做，元数据仍应提供同样清晰的指引。

最小修改：为旧任务增加被当前 PRD 取代的反向引用和历史状态说明，保留研究证据、原批准时间及原完成程度。自定义 meta 本身不会让现有列表自动过滤，应明确这一限制；不要用普通 archive 命令把未验收旧任务全部标成 completed，也不要在本轮改造 Trellis 工具。若以后要隐藏历史任务，再单独确定非完成归档的表示方法。

验证：检查 8 个任务的父子引用、指向当前 PRD/1102 的引用及状态含义；确认历史验收未被误标完成，当前任务指针未被接管。

### 3. medium：deploy 入口仍以旧拓扑定义产品

证据：`deploy/README.md:3` 定义 full/server/client 三角色，`:5` 仍称 Rust 程序未实现，`:47` 将读者引向已经替换为 v2 的根 PRD。INSTALL 和 MENTAL-MODEL 也沿用必需本地双模型的旧方案。根 README 已说明其历史属性，但直接进入 deploy 的读者看不到同等明确的版本边界。

影响：可能误判云识别必须运行自建服务器，或把旧安装骨架当作当前发行设计。

最小修改：先在部署入口增加历史版本说明，分别指向 `docs/history/2026-09-27-prd.md` 和当前架构/路线；保留原地文件、脚本和 ZIP，不先搬目录。修改前核对 `deploy/package-files.sha256` 与 ZIP 的收录范围，避免编辑说明后使既有完整性声明失真。若说明文件在收录范围内，需同步处理清单和骨架预览的版本边界。

验证：链接、JSON、清单与 ZIP 一致性；无需启动服务或安装器。旧模板仍应明确标为草案，不能声称它们是解析器接受的 schema 1/2 示例。

### 4. low：历史 PRD 移位后有 5 处相对链接失效

证据：`docs/history/2026-09-27-prd.md:211`、`:316`、`:317`、`:318`、`:324` 使用原位 `deploy/...` 路径；从历史目录解析时目标不存在。

影响：从历史需求追溯部署证据时导航中断。

最小修改：考虑到根 PRD 承诺旧需求逐字保留，优先在独立导航中记录原路径到仓库路径的映射，不批量改写历史快照正文。验证时将这些明确的原位链接作为历史快照例外记录，现行文档仍要求链接有效。

验证：确认快照正文保持不变，替代导航的所有目标存在；现行文档无新增失效链接。

## 建议顺序与边界

1. 同步现行规范，补旧任务和部署入口的历史指引，处理历史链接导航。
2. 后续获准提交时，将产品基础与既有 Trellis 0.6.14 → 0.6.17 工具更新分别审阅。当前 Cargo/apps/crates/docs 等大量内容仍未跟踪，普通 git diff 不覆盖它们；这不表示文件缺失。本轮不 stage、commit 或重置。
3. 然后沿 R1 接通麦克风 → Fun-ASR → 原文透明字幕。先设计独立识别生命周期、关闭翻译状态与正常 Stop 有界收尾，再接旧适配器；后续本地 ASR/翻译迁移按 R3/R4 分步做。

不建议为本轮整理移动源码或 deploy、删除旧三角色/协议、拆出空 provider crate、补虚构运行能力或重跑全部设备实验。已有 `Session::source` 始终排翻译、`Session::stop` 清空取消、`NativeInference::start_input` 要求两模型就绪，分别见 `session.rs:247`、`:314` 和 inference `lib.rs:113`；这些应在实际功能迁移中修改并测试。

## 本轮验证

- 只在根公开文件及 `docs/`、`.trellis/spec/`、`.trellis/tasks/`、`deploy/` 检查；未读取或修改用户根 `archive/`。
- 落盘前检查 81 个 Markdown 文件、169 个本地链接，发现上述 5 个历史快照原位链接失效；其余检查到的目标存在。此检查不解析 Markdown 标题锚点。
- 所有现有任务的 implement/check JSONL 路径检查未发现缺失目标；`task.py validate 11-02-rust-repository` 通过，两份清单各 5 项。
- `git diff --check` 通过；新增评估文件另做 UTF-8、末尾换行、行尾空白和链接检查。
- 未重跑 Cargo、设备/云/模型/窗口验收；75 项测试通过是此前 `validation.md` 的记录，不是本轮执行结果。

评估结束时尚未实施这些建议；后续补充整理的完成情况以以上链接的实施验证为准。R0 原始验收与补充整理分别记录。
