# synesthesia MVP 执行计划草案

状态：approved / implementation-in-progress。用户已在最终规划摘要后明确要求“开始实现”。
本任务是规划与集成的上层任务；以下 7 个子任务已创建，按表中依赖激活。
父子关系不表达依赖，创建后每个 child 必须在自己的 PRD / implement 中写明前置条件。

## 1. 规划收敛与启动门槛

- [x] 读取根 PRD、README、部署契约与 Trellis policy，记录现有工作区修改。
- [x] 建立 planning task，补齐需求、设计与执行计划草案。
- [x] 记录官方资料、兼容实验清单与真实上下文。
- [x] 确认 D-01 首发平台：Windows 首发，Linux/macOS 后续支持。
- [x] 确认 D-01b 首版 Windows 范围：Windows 10/11 x64，不限定 build 26100；分别取得两代系统兼容证据。
- [x] 确认 D-02 首版语言：英语、日语 → 简体中文；更多语种后续扩展。
- [x] 确认 D-02b 源语言手动选择英语/日语，自动检测后续；语言切换在新会话边界应用并隔离旧结果。
- [x] 确认 D-02c 样本范围：英日各固定 10 分钟清晰单人语音，短句/长句/静音；复杂场景补充测试，强噪声/重叠语音不设首版质量硬门槛。
- [x] 确认 D-03a 真实验收机器：用户提供 SSH，已完成 Windows/CPU/内存/GPU/Vulkan 只读检查。
- [x] 确认用户模型目录与两款文件：`E:\tmp\models`，已记录字节大小与本地 SHA256。
- [x] 确认 D-03b 资源策略：RX 9070 XT 优先，先验证 Vulkan；CPU 手动选择，GPU 失败报错、不静默回退。
- [x] 确认 D-04 首版范围：三角色、单路系统音频、本机/受控局域网 WS、双语悬浮；Windows 原生，不依赖 Docker；可选 Linux Docker CPU server。
- [x] 确认 D-05 字幕行为：草稿立即显示原文，仅定稿发起翻译，译文异步返回。
- [x] 确认 D-05b 显示方式：默认双语、原文上译文下、最近 3 句（含当前句），长句换行；可切仅原文/仅译文。
- [x] 确认 D-06 远程会话：手填地址/共享令牌、仅本机/受控局域网普通 WS、断线自动重连新会话并丢弃旧音频/上下文；TLS/公网后续。
- [x] 确认 D-07 生命周期：full 仅本机，对外算力由 server 提供；计算端停止会话保留模型，退出应用彻底释放所属资源。
- [x] 确认 D-08 悬浮交互：默认不穿透，托盘切换/恢复操作、设置、停止和退出；关闭设置只隐藏，快捷键/自启/自动更新后续。
- [x] 确认 D-09 数据保留：不保存音频/字幕内容；诊断日志不含正文/令牌，每个运行目录轮转总量上限 20 MiB。
- [x] 确认 D-10：60 分钟无崩溃/无持续积压；真实基准后与用户锁定质量/延迟/资源数值，达标才能发布。
- [x] 完成 requirement convergence 与 lossless PRD convergence pass，保留需求/决策身份、研究引用及 AC 映射。
- [x] 呈现最新 Goal / Scope / AC / Decisions / Risks 摘要，并获得用户后续明确批准：2026-09-27“开始实现”。
- [x] 最终规划批准后创建 7 个 child，分别写 prd/design/implement、依赖与真实上下文。
- [ ] 按依赖推进具体实施 child 的实现、独立检查及所需设备/发布验收。

当前无阻塞规划的产品问题；数值发布门槛按 D-10 明确延后，不把未测结果当作通过。
以下技术门槛归属实施阶段，不循环要求“先完成 C2 才能开始 C1”：

- C1 固定初始工具链、类型化配置/协议和有界队列，关键依赖检查 Windows 10/11 兼容。
- C2 核实模型来源/revision/许可、复核 hash、固定受控样本及实际推理后端；出具兼容与质量/性能基准，后续数值门槛向用户审阅。
- C3/C5/C7 留下两代 Windows 的实机证据；C6 对 full 与 client/server 各运行 60 分钟并对照发布指标。
- 未批准发布数值、设备证据缺失或未达标阻塞正式发布，允许继续不依赖失败条件的模块工作。

## 2. 建议拆分与次序

| 次序 | 建议 child | 前置 | 具体交付与验收 | 根 AC |
| --- | --- | --- | --- | --- |
| C1 | foundation-and-contracts | 产品决策与技术方向批准 | Rust workspace、三入口/依赖边界、类型化配置、领域 reducer、WS 编解码与 fake adapter | AC-04、AC-05、AC-10、AC-11、AC-14、AC-15 的确定性测试 |
| C2 | inference-compatibility | C1；用户模型与目标计算设备可用 | ASR/翻译兼容实验、固定引擎与模型记录、常驻/取消/就绪适配、实际设备诊断 | AC-02、AC-05、AC-06、AC-13、AC-14 |
| C3 | system-audio-capture | C1；首发 OS 与可用测试设备确认 | 系统输出采音、规范化、活动指示、默认/指定设备切换与停止 | AC-07、AC-14；AC-01/AC-03 输入侧 |
| C4 | compute-server-and-ws | C1、C2 | 无桌面 server、协商/会话控制、字幕事件、限额/过载/取消/隔离 | AC-02、AC-10、AC-11、AC-14、AC-15 |
| C5 | desktop-overlay | C1、C3；UI 与恢复操作批准 | full/client 共用设置/悬浮窗、字体、多屏/DPI、观看/调整与恢复操作 | AC-08、AC-09；AC-01/AC-03 显示侧 |
| C6 | full-client-integration | C2、C3、C4、C5 | full 本机完整链路；client 远端完整链路，无模型客户端，故障与持续运行证据 | AC-01、AC-03–AC-11、AC-14、AC-15 |
| C7 | packaging-and-docker | C6；发行/硬件矩阵与门槛满足 | Windows 原生角色包、安装/校验脚本、固定资产与许可，不要求 Docker/WSL；可选 Linux server Docker CPU 基线 | AC-12；AC-01–AC-03 安装验证；AC-13 按批准设备 |

AC-13 首轮针对已批准的 RX 9070 XT 原生路线；两款模型的实际设备诊断与 GPU 失败行为必须验证，不能标记为后续而跳过。
Linux/macOS、Wayland、多路音频和草稿翻译扩展必须单独获批，不自动创建排期。
C2 与 C3/C5 存在不同实验边界；C3/C5 首轮针对已批准的 Windows 10/11 x64，兼容路径与实机验证需分别覆盖。

## 3. 开发准备与规范

- 现有 backend/frontend 规范为模板，不能用 React/TypeScript 标题推断实际技术栈。
- 实施前读取 `trellis-before-dev`；以批准设计制定最小适用规范，标明 greenfield 决策。
- 有源码后，将经过验证的模式和真实路径补回 bootstrap guidelines；保留原 task 并单独验收。
- Windows 采用原生构建/验证，用户关于 Windows 不用 Docker 的要求优先于通用 Docker wrapper bootstrap 建议；不在验收机安装 Docker/WSL。
- 当前 agent 在非 Windows 宿主需要执行已确定的通用开发命令时，可按 `.trellis/spec/trellis-plus/development.md` 配置临时 wrapper；不能用它替代 Windows 原生构建与设备验收。
- UI 子任务按 validation policy 加载 UUPM、保存任务内研究并确认交互与状态，不创建第二套全局设计权威。
- 按 Codex Trellis dispatch 流程，实施/检查交给相应 Trellis agent，主会话负责决策与验收。每次 dispatch 以 `Active task: <task.py current 的路径>` 开头，优先 native context injection。

## 4. 验证方案

### 当前规划文档

已运行的检查另记于 `research/validation.md`，不把下面的计划当作已通过结果。

```text
python3 .trellis/scripts/task.py validate .trellis/tasks/09-27-synesthesia-mvp
python3 .trellis/scripts/task.py current --source
git diff --check
```

额外检查新文件的尾随空白、本地 Markdown 链接、JSON/JSONL 解析、上下文非 seed-only、
AC-01–AC-15 映射完整，以及 task 状态仍为 planning。普通 git diff 不覆盖未跟踪文件。

### 后续可自动化的行为验证

- reducer：draft→draft→final、重复 final、final 后迟到 draft、过期译文、历史淘汰后的迟到事件。
- 翻译时机：草稿输入后观察原文立即可见且没有翻译请求；定稿仅触发一次翻译，慢译文/失败不阻塞原文或下一句识别。
- 身份隔离：重新开始、设备切换、WS 断线后旧任务结果、不同客户端/语言上下文。
- 协议：未知版本、畸形 JSON/二进制、样本数/序号错误、超大帧、未就绪 start、错误 session_id。
- 鉴权：缺少/错误令牌在接收音频前拒绝，错误明确；令牌不出现在 URL、日志、示例或发行包中。
- 重连：网络断开→有界自动重试→新 session_id；清空旧音频/上下文，拒绝迟到旧结果；手动停止可取消重试，鉴权/版本错误不形成重试循环。
- 调度：慢翻译不阻塞 source；音频/事件/翻译队列各自限额；可取消的负载与停止超时。
- 生命周期：停止后没有旧会话工作，模型仍 ready；重新开始使用新会话身份且复用模型；退出回收模型、引擎与所属辅助进程。分别记录模型加载次数、会话任务状态与退出后进程状态。
- 产品边界：full 不开放局域网 WS 计算服务；私有引擎服务仅绑定回环；远端客户端连接独立 server。
- 悬浮交互：启动默认不穿透；托盘可切换观看/调整、打开设置、停止和退出；关闭设置只隐藏且字幕继续。托盘不可用时禁用穿透，运行中入口失效恢复可操作模式；退出仍完整回收资源。首版不注册全局快捷键、不配置自启或自动更新。
- 字幕呈现（C1 reducer/config、C5 窗口、C6 集成）：默认 3 个语句组含当前句、原文上译文下；长句自动换行，草稿替换/定稿/译文不重复占组；第 4 句淘汰最旧组且迟到译文不将其插回。切换双语/仅原文/仅译文不增加推理请求，仅译文模式隐藏草稿原文并显示等待/失败状态；窗口中英文/日文/中文与多屏缩放实测。
- 数据保留（C1 诊断基础、C2 引擎适配、C6/C7 集成与包装）：使用可识别的测试音频、字幕正文和令牌，检查日志/缓存/临时目录无会话内容或凭据；引擎输出不原样落盘。连续写入、超大单条记录、重启及不可写/轮转失败场景下，应用与所属引擎日志合计仍不超过 20 MiB，并有明确诊断状态；不提供字幕回看/内容导出。
- 资源策略：GPU 初始化失败、运行时 GPU 错误均有明确反馈，不静默改用 CPU/集显；手动选择 CPU 后诊断如实反映设备，ASR/翻译分别验证。
- 包装：client 依赖与载荷无模型/推理；server 无桌面；从非根目录启动；移动便携目录。
- Windows 包装：没有 Docker/WSL 环境也可安装、启动三个角色；启动器与安装器不要求容器工具或服务，日志不把缺少 Docker 当成应用错误。
- Windows 版本兼容（C1/C2/C3/C5/C7）：检查固定依赖与系统 API；覆盖 Windows 10 1703 前回环事件差异及所选兼容路径。分别在 Windows 10/11 x64 验证共用发行程序，Windows 11 build 26100 不是最低版本或 Windows 10 的替代证据。
- 模型实验：指定 GGUF 加载、短句/静音/长句、source revision 对应、实际设备与内存证据。
- 在验收机读取 `E:\tmp\models` 两款文件，先复核模型基准 hash；另验证外部绝对路径、便携相对路径及含空格路径，不把测试盘符写成默认值。
- 语言验收：显式英语、日语分别验证 ASR 与简中翻译；各覆盖短句、连续讲话、长句、静音和延迟译文。未选择语言不能开始，配置/协议传入 auto 或其他语种明确拒绝；语言切换建立新会话、清旧上下文并拒绝迟到旧结果，未改变模型/设备时不重新加载模型。
- 基准样本（C2 固定、C6 复用）：英日各 10 分钟清晰单人语音，固定文件/hash、来源、参考文本与语音边界；主样本包含短句/长句/静音。多人对白/背景音乐单独报告，强噪声/重叠语音不混入首版质量硬门槛。按 research/benchmark-plan.md 记录指标，不持久化应用生成的音频/字幕正文。
- D-10 发布门槛：C2 分别报告各语种质量、草稿/定稿/译文 p50/p95/最大延迟、失败数及共同推理资源/前台开销；向用户审阅锁定数值后由 C6/C7 复测。full 与 client/server 各 60 分钟无崩溃且积压不持续增长；不以降低门槛或剔除失败样本判定通过。
- 可选 Linux Docker：在声明的 Linux 宿主验证真实镜像启动/就绪、外部模型配置、客户端会话与容器停止；CPU 实时能力单独报告。不在 Windows 验收机安装 Docker，也不外推容器 GPU 能力。

确认 Rust workspace 后，候选常规检查为 `cargo fmt --all -- --check`、
`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`。
已由 C1 建立 Cargo manifest/lockfile 与 Rust 1.90.0；Linux 和原生 Windows 11 的最终基础快照已通过 fmt/check/clippy/test/build，36 项测试。实际命令见 `.trellis/spec/backend/quality-guidelines.md`，原生证据见 C1 `research/native-validation.md`。C2/C3 已进入实现，后续设备与完整链路仍需独立验证。
平台相关 feature matrix 与构建命令由 C1 固定；测试不能因为 host 不支持采音而假报平台通过。

### 必须取得的真实平台/硬件证据

- 系统播放采音与设备切换；不同缩放、多屏、字体、透明/置顶、穿透与恢复入口。
- Windows 10 和 Windows 11 各自的三角色原生启动/运行证据，以及 full/client 采音、窗口和连接行为；记录实际 build、依赖/驱动与测试设备。缺少 Windows 10 环境列为实机验收缺口，不缩小产品支持目标或假报通过。
- GPU 明确选择、模型加载与实际推理，不以设备枚举代替实际使用。
- 在已记录的 RX 9070 XT 上验证两款 GGUF；区分集显与独显，记录 Vulkan 图层告警是否影响加载、吞吐或退出。CPU 对照通过显式选择运行，GPU 失败不得自动回退；不因首次枚举成功跳过推理检查。
- 受控语言样本的字幕/翻译质量、持续运行、延迟与内存门槛。
- 停止字幕后无采音/上传或旧会话推理活动，模型驻留符合设计；恢复时无不必要模型重载。退出应用后窗口、模型、所属任务/辅助进程与音频资源完全释放；全屏覆盖范围按实测声明。

记录 OS/显示协议、硬件、模型/引擎版本与哈希、步骤、结果与 scoped diagnostics。
原生悬浮窗分类为 `playwright-not-effective`；不添加无效浏览器测试充数。

## 5. 评审与回滚点

- 每个 child 首先做有意义的聚焦检查，最后合并做全链路验收。
- C2 失败先停止选型收敛，给出证据；不换模型或放宽产品要求掩盖失败。
- 协议/schema 改动同时检查所有消费者与部署示例，固定版本后拒绝不兼容输入。
- C7 在应用与验证未完成前保留 scaffold 标记，不发布正式资产或部署外部服务。
- C7 同步现有 deploy Docker 文档与实际 Linux CPU 镜像边界，再更新对应清单/打包校验；当前不改已有 scaffold 归档或校验和，不把 Docker 草案当 Windows 安装步骤。
- C1/C4 仅接受 en/ja 源语言和 zh-Hans 目标语言；C5 提供显式语言选择，C7 同步部署示例中的 source=auto 旧值及归档/校验清单，保留其他 backend/device auto 字段的独立含义。
- C1/C5/C7 同步部署草案 history_lines 与语句组计数的明确契约；默认 3 句、长句视觉行数与历史数量分离。
- 按项目 human review policy 请求剩余设备/视觉证据，自动检查先运行。
- 回滚只处理该子任务创建的确切路径；不删除用户模型、配置或其他 task。
- 提交与归档遵循正常授权，不因本任务建立而自动 commit/archive 或启用 serial mainline。
