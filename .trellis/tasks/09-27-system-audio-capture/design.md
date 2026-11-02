# Windows 系统播放采音：技术设计

本版沿用已批准的 [父设计](../09-27-synesthesia-mvp/design.md)，范围为 crates/synesthesia-audio 与采音聚焦测试/诊断；不实施 GUI 或模型推理。

## 依赖与边界

- [foundation-and-contracts](../09-27-foundation-and-contracts/prd.md)：所需交付验证通过后才能开始本 task 的依赖实现。

领域状态、事件身份、revision、语言、音频时间和资源归属由共享契约统一；接口改变需同步消费者与聚焦测试。
新建模块按职责依赖，不能让 client 引入推理/模型或 server 引入桌面。

## 执行与验证

WASAPI 回环、格式规范化、设备选择/切换与活动诊断。
本 task 的可观察验收为 AC-07/14 与 AC-01/03 输入侧；Windows 10/11 实机采音证据；根据当前源码/工具证据固定技术细节并记录本 task research，不将推测写成既有能力。

## 已实现的适配边界

源码位于 `crates/synesthesia-audio`。`windows=0.62.2` / `windows-core=0.62.2` 仅为 Windows 目标依赖；`rubato=0.16.2` 禁用默认 FFT 功能，使用单声道 `SincFixedIn<f32>`。workspace 核心的 unsafe 禁令不改变，FFI 只在本 crate 的 `native.rs`，其资源归属和指针有效期由逐处 safety 注释说明。

`CaptureHandle::start(CaptureConfig, InputId)` 在专属 STA worker 中选择 active `eRender/eMultimedia`，注册原子通知信号并在打开后复查选择，使用 shared LOOPBACK、100 ms 请求缓冲及 5 ms 有限消息等待轮询；不用 EVENTCALLBACK、IAudioClient3 或麦克风。实测 buffer 必须在 10..200 ms 范围内。初始化/filter 准备完成后才 Start；COM 引用、GetBuffer/ReleaseBuffer、task allocation、Stop 和通知解除留在 owning thread。归一化由第二个 worker 处理；两个 worker 都由 handle 的 stop/drop join。

默认设备身份或 mix format 改变返回 `InputChangeRequired` 并终止旧输入；锁定设备失效明确失败，不改用默认设备。调用方先执行共享 Session 的 StopCapture/Cancel/Clear，确认 `CaptureHandle::stop` 已 join 并清空旧帧，再以新 generation 调用 start。C3 不绕过 C2 的取消屏障自动改模型输入身份；完整恢复调度归 C6。后续 DATA_DISCONTINUITY 终止输入，首包可带初始边界标记；TIMESTAMP_ERROR 只计数，不推导缺失时长。

WAVEFORMATEX/EXTENSIBLE 的纯 decoder 与原生入口共用 `MixFormat::from_wave_descriptor`；验证声明长度/平均字节率/容器位数/有效位/GUID/通道 mask。支持 PCM16、packed24、32-bit container 的有效位左对齐与 float32，8..192 kHz、1..8 通道；非有限浮点明确失败。无 mask 的 mono/stereo 分别直通/等权平均；合法 mask 按位序等权平均非 LFE 通道，multichannel direct-out 或 mask/count 不一致拒绝。此策略不做音频增强，反相声道会相消。

归一化跨 packet 保留滤波/分数状态，chunk256、sinc256、BlackmanHarris2/cutoff0.95、linear interpolation/oversampling256；16 kHz 直通。丢弃初始 output_delay，输出只占用 `floor(actual_native_frames * 16000/native_rate)` 样本时轴。packet absence 达 100 ms 且 raw queue 已排空时，内部滤波 padding 仅取真实输入的有效尾部，并重新建立下个片段的 filter；虚拟 padding 不产生样本时间。新的实际片段继续全局整数样本计数及 seq，保留 fractional-count remainder。Stop/switch 清除全部旧 filter/FIFO/tail，禁止为了停止而补发尾部。800 ms 空闲定稿由 C2 的显式 `finish_utterance(input, observed_end_sample)` 处理，只有排空且已送达推理的真实 cursor 可用于它；实际 SILENT packet 的帧数照常归一化。

## 队列与内存上限

- Endpoint requested100 ms，实际要求10..200 ms；单 borrowed packet 最多100 ms，读取后完整释放。
- 8 个预分配 raw buffer，合计容量最多4,915,200 bytes（192 kHz ×100 ms ×8ch ×4bytes ×8）；在途/队列 raw 总帧数最多200 ms，超限通过独立 atomic failure 通道结束输入，不丢包后继续。
- Normalizer 单个 decoded packet 最多100 ms，已计入 raw 在途计数；partial FIFO 少于256 native frames，filter delay另外计入，不能按每包独立四舍五入。
- Normalized publication 最多3200 samples（200 ms）且64帧；共享 Session 后续另有32000 samples/100帧与单 ASR200 ms。包括 endpoint200 ms、raw200 ms、FIFO32 ms、filter16 ms、C3 output200 ms、core2 s 和 ASR200 ms 的保守叠加上限为2.848 s，非发布时延承诺；C6 调度应持续及时排空，D-10 实测后评估预算。
- 失败保留首个 enum/HRESULT 于一项 atomic；正文/设备名称/opaque ID 不进入 Debug/日志。观测只提供数字/身份/安全类别，未写入额外日志或录音文件。

## 复测入口与当前证据

`cargo test/clippy/check -p synesthesia-audio` 使用锁文件和固定工具链。原生 example `audio-diagnostic` 可指定 `--seconds 1..30`、`--expect-active`、`--locked-default`、`--overload`、`--controlled-playback`；未知/矛盾参数拒绝。

受控模式在 **capture 实际所选输出的相同 opaque ID** 上建立独立 shared renderer，内存生成左右697/941 Hz；pause 实际 Stop+Reset 而非送 silent buffer，下一输入在旧 capture join 后启动再 resume。只累积数字频谱幅度，既不录音也不依赖 ambient 音频来代替受控活动。no-packet 条件不可用时明确输出 unavailable，不能关闭用户媒体或修改系统音量/default device 来冒充通过。执行结果与剩余 Windows10/设备切换/原生 review 见 [validation](research/validation.md)，由主会话的真实 Windows 命令补充。

## 失败与回滚

失败给出具体范围/错误/实验输入与结果，不自动退到 CPU、不换模型、不删失败样本。
回滚仅处理本 task 已知创建的文件；保留用户配置/模型、其他子任务和初始 PRD/README 改动。
