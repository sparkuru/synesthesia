# 指定模型与 RX 9070 XT 推理兼容：技术设计

本版沿用已批准的 [父设计](../09-27-synesthesia-mvp/design.md)，范围为 crates/synesthesia-inference、推理实验辅助文件及本 task research；不修改用户模型。

## 依赖与边界

- [foundation-and-contracts](../09-27-foundation-and-contracts/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

领域状态、事件身份、revision、语言、音频时间和资源归属由共享契约统一；接口改变需同步消费者与聚焦测试。
新建模块按职责依赖，不能让 client 引入推理/模型或 server 引入桌面。

## 执行与验证

固定引擎/模型来源，ASR/翻译真实加载、取消/常驻适配与基准报告。
本 task 的可观察验收为 AC-02/05/06/13/14；两模型实际设备证据及 D-10 数值审阅；根据当前源码/工具证据固定技术细节并记录本 task research，不将推测写成既有能力。

## 实现准备所得的约束

[固定版本研究](research/native-engine-readiness.md)、[实际模型元数据](research/model-metadata.md) 与 [原生工具环境](research/build-environment.md) 已建立，仍不是推理通过证据。
候选 CrispASR `v0.8.37` / llama.cpp `v0.5.0` 的源码 commit、模型发布 revision 与 hash 必须随实际选型固定；不得以浮动 latest 代替可复测资产。

CrispASR 的普通 PCM stdin CLI 存在设备自动回退、EOF 退出卸载与缺少会话 reset/cancel 控制的已知限制，不能直接包装后宣称满足 D-03b/D-07。
先以小实验验证受控 helper/FFI：引擎所有权与会话输入/上下文分离，停止完成前取消或结束旧推理并销毁旧上下文，完成后保留已加载模型；重开/切换语言以新身份复用模型。任何不满足的阶段如实失败，不以屏蔽迟到输出替代停止，也不以杀进程重载代替正常停止常驻。
对 GPU 初始化使用明确的设备验证与拒绝路径，实际所选后端/独显不能只依赖上游 flags；候选源码的自动 CPU/其他设备路径必须被有证据地阻止。手动 CPU 仍是独立配置模式。

Hy 文件实际使用 `hunyuan-dense` 与 K-quant/IQ3 混合 tensor；采用已核对的内嵌模板，限简中定稿翻译。
若使用私有 llama-server，只绑定回环、禁止 idle 卸载与内容/slot/prompt 落盘，证明请求取消可到达服务引擎并清除旧会话 context；应用退出回收所属服务。私有 HTTP 与产品 WS 协议分离。

共享领域/音频身份与有界队列以最终经检查的 C1 源码和 docs/contracts.md 为准。
ASR 窗口与翻译 context 本身也需显式预算，不以外层 2 秒音频队列掩盖内部无限缓存。
引擎 stdout/stderr 用有界解析提取白名单诊断，不原样打印/落盘；无模型或设备不匹配不能报告 ready。
[WASAPI 准备研究](../09-27-system-audio-capture/research/wasapi-readiness.md) 指出没有包与实际 SILENT 包不同；空闲定稿控制需与 C3 协调并以可测试的样本时间/控制边界表达，不把墙上时钟缺口伪装成采集音频。

原生工具与构建/实验仅在已建立的 task-owned 临时目录中准备；不修改用户模型、全局 PATH/profile/注册表，不要求 Windows Docker/WSL。
引擎、驱动、后端、模型加载与实际推理各层证据分开。短样本 smoke 不替代英日各 10 分钟基准或 60 分钟链路验收；[样本来源准备](research/benchmark-sources.md) 仍需固定实际文件/hash、参考与边界。

## 失败与回滚

失败给出具体范围/错误/实验输入与结果，不自动退到 CPU、不换模型、不删失败样本。
回滚仅处理本 task 已知创建的文件；保留用户配置/模型、其他子任务和初始 PRD/README 改动。

## 当前实现的技术选择

`crates/synesthesia-inference` 直接实现 C1 `ComputeAdapter`，Rust 不使用 FFI。
两个受管理 C++ helper 分别静态链接固定 CrispASR/ggml 与 llama.cpp，通过有界二进制 stdio 通信，不启动任何 HTTP/WS 服务。
不共享两套 ggml 的符号或 DLL。模型读取前核对已批准的 SHA256/大小，不自动下载或替换。

设备匹配由各引擎自己的 Vulkan registry 解析，补充读取实际 physical-device vendor/device ID，加上名称唯一匹配；初始化指定 backend 失败直接拒绝。
Qwen 与独立 audio tower 均强制同一指定设备，不走自动选择/CPU 模型权重溢出路径；Hy 检查所有 transformer block 矩阵的真实 buffer device。
CPU 是显式单独模式；构建关闭 AVX/AVX2 等可选 ISA，以 x64 基线编译。无 Vulkan loader 的 CPU 主机应使用 `GGML_VULKAN=OFF` 构建的 helper，不能把 Vulkan 版二进制当作通用 CPU 包。

流式识别使用真实 PCM 的能量分段作为初始实验策略：峰值阈值 120/32768，实际静音 800 ms 定稿，最多 10 秒语句窗口，每 1 秒可发草稿。
可变帧跨窗口时整帧留在有界 ingress 队列，不丢弃/拼接缺口；长句按窗口切分。
该阈值、窗口与步长是明确的技术起点，不代表 D-10 已批准的质量/时延门槛。弱语音、短促声音、天然停顿及长句需由固定基准验证/调整。
无包空闲通过 `finish_utterance(input, observed_end_sample)` 控制定稿，不生成静音 PCM；C3 已同意只在原始队列及 DSP tail 排空后传入实际 emitted cursor。

每个 helper 同时最多一个计算任务。取消在控制线程设置原子标志，ASR 已提交的 GPU graph 可能需要完成，但后续 graph/token 不再执行；翻译在 batch/token 边界检查，CPU 另有 abort callback。
取消确认前 join 实际工作线程、清 KV 并销毁旧 scheduler/推理 context，再建空 context；保留同一 model 对象、模型权重和进程。
不以屏蔽迟到结果或杀进程重载代替正常取消。60 秒取消/任务期限是安全技术上限，仍需实测取消延迟；超时明确失败，回收所属 helper。
实验专用状态查询只返回 ASR 阶段、实际 graph 内标志、累计 graph 数和当前任务编号。取消实验分别等待 encoder、prefill、decode 中实际 graph 开始，再执行取消并检查 idle、同一进程、同一模型加载计数和不再增长的 graph 数；仅排队或缓存了 PCM 的试验不算计算中取消证据。
翻译超时先发布该句失败并异步等待取消，ASR 事件继续；翻译 helper 故障后继续发布失败状态，显式重新初始化才能恢复其模型能力。

std::io 边界先检查固定 header 和 payload 长度，不解析任意日志文本。
引擎 stderr 直接进 OS null sink；CrispAudio dump/imatrix hooks 在固定源码构建中禁用，环境变量不能开启内容落盘。
研究样本/参考与 app 生成内容分开；实验只输出匿名计数、耗时、错误统计，生成的正文不落盘。
详细边界、重建步骤和实际验证状态见 [native-adapter.md](research/native-adapter.md)。
