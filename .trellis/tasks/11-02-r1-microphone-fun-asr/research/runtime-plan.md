# Research: R1 运行时迁移与验收顺序

- Query: Windows 11 x64 上，以麦克风、Fun-ASR、关闭翻译、原文透明字幕形成首条真实链路，需要哪些已有能力、边界迁移和技术验证？
- Scope: mixed；只读源码与官方服务文档研究，不实现代码、不访问凭据或设备、不发起云请求。
- Date: 2026-11-02

## Findings

### 文件与既有能力

| 文件 | 可复用内容及准确边界 |
| --- | --- |
| `crates/synesthesia-core/src/pipeline.rs:37`、`:52`、`:78`、`:112` | schema 2 已独立表达输入、识别、翻译和显示；`:220` 强制翻译关闭时仅原文。配置不等于运行时适配。 |
| `crates/synesthesia-core/src/domain.rs:73`、`:97`、`:119`、`:208` | 会话/输入代际、句子身份、版本、事件与唯一字幕 reducer；无需另建 UI 字幕历史。 |
| `crates/synesthesia-core/src/session.rs:39`、`:100` | 可借鉴纯状态机、effects、队列和精确完成身份；旧 ComputeAdapter 同时拥有音频和翻译，不应直接充当新云识别接口。 |
| `crates/synesthesia-audio/src/normalize.rs:27`、`:70` | 已有设备格式校验、带限重采样与 16 kHz mono PCM16 输出时间轴。 |
| `crates/synesthesia-audio/src/capture.rs:72`、`:277`；`native.rs:118`、`:134`、`:504`、`:629` | 现有句柄及设备类型只服务 render endpoint / loopback，没有麦克风适配；复用线程/缓冲/错误管理不等于已有麦克风。 |
| `crates/synesthesia-desktop/src/presentation.rs:10`、`:28`、`:30` | 纯借用投影已实现，但没有 Disabled 翻译状态，也没有窗口。 |
| `apps/synesthesia/Cargo.toml`、`crates/synesthesia-desktop/Cargo.toml` | 正式入口目前仅依赖 core；desktop 没有 GUI 依赖或事件循环。 |
| `config/examples/microphone-fun-asr.json` | 已有无本地模型、关闭翻译的结构示例；workspace_id 未填写，运行端点兼容需要明确。 |

### 必须先解决的源码接线缺口

1. **识别独立就绪。** `session.rs:171` 的 loaded 注释仍以双模型真实加载为前提；`:247` 的 source 在每个被接受的定稿后排翻译；`inference/src/lib.rs:113` 又要求两个本地进程都存在。R1 应建立独立识别生命周期及 Disabled 翻译策略，直接证明不会构造翻译队列/任务，不通过空译文适配器伪装就绪。纯 reducer 可以继续给出“定稿已接受”的数据，但是否调度翻译由显式策略决定。
2. **正常停止必须跨越采集与网络边界。** `session.rs:314` 当前撤销 input、清队列并丢 reducer；`:247` 只接受 Running 状态的来源结果。`capture.rs:299` 在 stop 后拒绝取帧，`:325` 的 stop 清未读输出；`normalize.rs:161` 明确 Stop 丢弃尾部。新正常 Stop 必须先截止采音，再有界排空已接收的规范化音频，发送服务结束指令，继续接最后结果，然后结束身份。故障/超时取消可丢剩余内容，但应报告最后结果不完整。旧接口不能原样充当该 drain 操作。
3. **不要把云发送完成冒充 ASR 帧完成。** `session.rs:211` 只允许一个 ASR 在途帧，`:220` 用 `(input, seq)` 释放拥有权。云流式发送与返回字幕没有一帧一结果关系；新适配层需区分“本地写端不再拥有 PCM”和“云任务处理完成”，独立约束传输队列。保留精确身份检查，不虚构服务端逐帧确认，也不等一句字幕才发送下一帧。
4. **服务事件不能直接反序列化成核心事件。** `domain.rs:137`/`:145` 要求非空文本、非零版本及有效采样范围，`:253` 把 final 视为终态，`:267` 固定一句起点且终点不能倒退。适配器应先过滤心跳/空句首标记，再按任务与输入身份关联句子，分配本地版本，校验 ms→sample 的整数边界。缺失 end_time 不能随意填成当前采音位置，导致真正定稿时间倒退；草稿未知时间的表示必须先用 fixture 明确，再接 reducer。
5. **仅原文仍需显式状态。** presentation 的 None 译文会显示 Pending（`:38`），不能让 R1 看起来正在等待一个从未创建的翻译任务。增加明确 Disabled 输入/状态，并保持切换显示不会调度工作。视觉行数是渲染限制，不能直接替代 reducer 历史容量。

### 已核实的 Fun-ASR 官方接口事实

以下是服务契约证据，不是本仓库的实现或云验收结果。

- 官方接入文档列出北京/新加坡 Workspace 专属域名，并建议从原 DashScope 域名迁移；握手用 Bearer 鉴权。schema 2 的 workspace_id 仍可选，建议采用下述确定性端点映射，不在模型或地域间静默回退。见 [WebSocket 接入指南](https://help.aliyun.com/zh/model-studio/fun-asr-realtime-websocket-api)。
- 连接后发启动指令，收到 `task-started` 后才上传单声道二进制音频；正常结束发送 `finish-task`，仍接收识别结果，直至 `task-finished`。应用应另设有界等待和故障取消，而非立即断连。见同一份 [接入指南](https://help.aliyun.com/zh/model-studio/fun-asr-realtime-websocket-api)。
- 当前客户端事件文档提供 PCM / 16000 的示例；schema 的 sentence_silence_ms 应显式映射到 `max_sentence_silence`，不能假定服务默认值等于本地 800。context 有结构化消息格式，不能将 schema 的单字符串原样透传；语言提示应按所选模型校验，R1 不自动扩大 schema 的 zh/en/ja 集合。见 [客户端事件](https://help.aliyun.com/zh/model-studio/fun-asr-client-events)。
- 服务结果包含句子编号、毫秒时间、中间/最终标记；空句首示例有空文本和空结束时间，心跳不是字幕。失败事件的外来错误文本不能直接进入诊断。见 [服务端事件](https://help.aliyun.com/zh/model-studio/fun-asr-server-events)。

### workspace_id 可选时的具体运行策略建议

官方当前 [Fun-ASR HarmonyOS 文档](https://help.aliyun.com/zh/model-studio/harmonyos-sdk-for-fun-asr-real-time-service) 明列公共北京端点；[语音 SDK 选型文档](https://help.aliyun.com/en/model-studio/sdk-selection-and-download) 仍列北京、新加坡公共端点，并将 Fun-ASR 列为相应服务能力。此处只采纳端点证据，不引入移动 SDK，也不将其当作 Rust 端到端验收。

建议把下表写入 R1 design，并用纯函数/离线测试固定；这是本项目建议契约：

| 配置 | 确定性目标 |
| --- | --- |
| workspace_id 缺省，cn-beijing | `wss://dashscope.aliyuncs.com/api-ws/v1/inference` |
| workspace_id 缺省，ap-southeast-1 | `wss://dashscope-intl.aliyuncs.com/api-ws/v1/inference` |
| workspace_id 存在 | 对应地域的 `wss://{WorkspaceId}.REGION.maas.aliyuncs.com/api-ws/v1/inference` |

workspace 作为 DNS 标签前需做适配边界格式验证，不能直接拼接任意标识符为主机名。目标由配置一次确定；握手或任务失败时不自动切域名/地域/模型。若实施时官方支持或真实账户条件变化，运行就绪检查应返回安全、明确的端点/业务空间配置错误，指导补 workspace；不把条件变成结构解析阶段读密钥或联网，也不修改旧无 workspace 示例来伪装已验收。

### 可现在确定的内部约束与待验证决策

| 事项 | 现在可确定 | 实施前/早期验证门槛 |
| --- | --- | --- |
| 运行编排 | 识别就绪后才开麦；云模式不构造本地模型；关闭翻译不产生任务；UI 线程不做阻塞 join/网络操作 | 将新的纯状态/effects 与实际异步适配器契约写入 design，并验证单一资源所有权 |
| 生命周期 | Start → Preparing → Running → Draining → Stopped；失败/超时另走取消，都是建议内部状态名，并非已存在签名 | Stop 尾帧、尾部重采样、服务最终结果、取消 deadline、重复操作和退出释放的假时钟/可控传输测试；数值是工程预算，不宣传为质量或性能指标 |
| 定稿修订 | 当前官方文档将 sentence_end 标为最终结果；本次未找到“同句定稿之后可合法修订”的明确保证 | R1 优先保留 final 终态与重复去重。对相同身份的不同内容第二次 final，先定义可见协议/能力错误；如真实固定样本证明合法修订，再设计 v2 专用语义与译文失效，不悄悄改变 WS v1 |
| 时间与身份 | 每次显式重启新会话，输入切换新代际，清屏保留高水位；适配层拒绝错 task_id 的晚到结果 | 用空句首、无结束时间、中途文字替换、重复/迟到 final 与 clear 后结果的 fixture 固定映射，不能通过改 UI 绕过 core |
| 麦克风 | 独立 capture endpoint 适配，复用 MixFormat/Normalizer/AudioChunk 和安全错误；默认/指定设备均应有真实边界 | Windows 11 真麦克风、权限拒绝、禁用/拔插、设备变化、格式、静音、停止与重启；不能宣称旧 WASAPI 回环结果覆盖麦克风 |
| 窗口 | 透明置顶、移动/缩放/字号/换行、视觉行数、清屏、恢复和可见错误属于当前需求；托盘/穿透不在 R1 必需范围 | 用真实 Windows 11 原型验证 alpha、置顶、恢复控制、字体/DPI/焦点与事件循环；旧 C5 的 eframe/renderer/version 仅是候选研究，不是本轮批准或可直接照抄的依赖锁定 |
| 云配置 | 结构检查继续不读密钥、不联网、不打开设备；运行时能力错误单独报告 | 区域/模型/语言、workspace 端点、热词/context 映射、消息大小与文本边界、超时和静音行为；不做自动重连或断线补播 |

### 建议实施与验收顺序

1. 用离线受控事件/假传输固定识别独立生命周期、Disabled 翻译、输入身份与 Stop drain 契约；覆盖上面的空/迟到/重复/时间边界，不改动旧 WS v1 行为。
2. 建真实 Fun-ASR 适配边界和受控 WebSocket 对端测试，固定启动、失败、结束、大小及预算约束。需要真实实现时再增加相应包；不先建空插件层，不从旧三角色入口复制应用。
3. 接麦克风 adapter，验证规范化和停止尾部拥有权；先有设备级证据，再与云识别串联。云实测仅使用用户本机凭据，不保存录音、字幕或外来错误正文。
4. 在并行的技术原型门槛后接入真实原文窗口；验证源事件到 reducer 到 UI 的唯一状态流，以及清屏、错误、恢复、Stop 最后一行和退出。
5. Windows 11 x64 端到端验收记录实际版本、设备、地域/模型、语言、质量、延迟、资源和清理结果。R1 可用链路与 PRD 的 60 分钟发行支持门槛分开；不得仅凭短样本就宣称正式支持组合或给出未经测量的质量/延迟数值。

### 相关规范

- `.trellis/spec/backend/pipeline.md`：schema 2 与无副作用结构检查。
- `.trellis/spec/backend/core-boundaries.md`：旧 state/wire 不变量及迁移边界。
- `.trellis/spec/backend/quality-guidelines.md`：全 workspace 与原生验证边界。
- `.trellis/spec/frontend/native-ui.md`：当前 UI 需求与历史 C5 范围分离。
- `.trellis/spec/trellis-plus/validation.md`：自动检查、原生残余验证与任务证据。

## Caveats / Not Found

- 本次没有运行代码、真实麦克风、窗口、云请求或任何原生实验；不包含当前依赖选型/MSRV/renderer 可用性的复验。
- 没有读取用户根 archive 或 Python 原型；原型中“定稿修订”的说法不构成当前 Fun-ASR API 的服务保证。
- 可选 workspace 的建议映射已有上表及官方端点依据；context 单字符串、缺失草稿时间与长静音行为仍需固定适配策略和测试，不能按示例解析通过推断真实可用。
- 技术原型和服务 fixture 是实施门槛，不授权改变产品行为、扩展平台或开始实现；本任务仍是 planning。
