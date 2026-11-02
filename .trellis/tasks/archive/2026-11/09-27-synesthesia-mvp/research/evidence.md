# 规划证据与兼容验证边界

日期：2026-09-27。以下为只读证据，不代表本项目已运行模型或实现相应 API。

## 仓库事实

| 证据 | 结论 |
| --- | --- |
| `prd.md` §4/5/10 | 三角色、系统采音、草稿/定稿、翻译隔离、生命周期与 AC-01–AC-15 |
| `prd.md` §6.3；`deploy/scripts/CONTRACTS.md` | 模型用户提供；软件版本/hash 固定；便携目录与调用者 CWD 无关；实际安装器未实现 |
| `readme.md` | 首轮识别 Qwen3-ASR 1.7B Q4_K；翻译 Hy-MT2 1.8B UD Q3_K_XL |
| `deploy/payload/config/*.example.json` | schema draft；单路/auto/显式失败/关闭草稿翻译和 transcript 是草案值；source=auto 已与 D-02b 手动源语言选择不一致，交付时需同步，不混同 backend/device auto |
| `deploy/docker/server.example.json` | CPU、容器内 0.0.0.0、外部模型路径是草案，不证明镜像存在 |
| `.trellis/spec/backend/index.md`、`frontend/index.md` | 指南待填；不存在可据以提炼的应用模式 |
| `.trellis/spec/trellis-plus/validation.md` | 原生 UI，Playwright 无效；设备验收不能用容器测试代替 |
| `.trellis/spec/trellis-plus/development.md` | toolchain 未确认，dev wrapper 暂缓 |

## 外部一手资料

只记录与设计有关的能力边界，未复制上游实现；发布选型仍需固定 revision。

- [CrispASR v0.8.37 streaming](https://github.com/CrispStrobe/CrispASR/blob/v0.8.37/docs/streaming.md)：说明 PCM 管道与 JSONL 草稿/定稿，可作为 ASR 适配器原型依据；指定模型的吞吐、GPU 和退出行为仍待实测。
- [llama.cpp](https://github.com/ggml-org/llama.cpp)：作为翻译运行时候选；模型后缀不是兼容性保证，固定版本后测试常驻、取消与资源回收。
- [Tencent Hy-MT2-1.8B-GGUF](https://huggingface.co/tencent/Hy-MT2-1.8B-GGUF)：官方模型家族资料；README 指定的 UD 量化资产来源未确定，不据此推断该文件的 hash 或兼容版本。
- [Microsoft loopback recording](https://learn.microsoft.com/en-us/windows/win32/coreaudio/loopback-recording)：WASAPI 支持从播放端点回环采集；Windows 10 1703 前事件驱动回环需要额外兼容处理，1703 及以后不再需要该 workaround。Windows 10/11 x64 范围已批准，版本差异需在适配与测试中处理；设备切换与完整产品能力未验证。
- [Rust platform support](https://doc.rust-lang.org/rustc/platform-support.html)：`x86_64-pc-windows-msvc` 为 Tier 1 with Host Tools，列明 Windows 10+；这支持共用 Windows x64 编译目标的候选设计，不证明 GUI、推理引擎、GPU 驱动或本产品已兼容。
- [egui ViewportBuilder](https://docs.rs/egui/latest/egui/viewport/struct.ViewportBuilder.html)：窗口能力接口候选；真实透明/穿透/置顶组合仍须测试。
- [winit WindowLevel](https://docs.rs/winit/latest/winit/window/enum.WindowLevel.html)：存在 Wayland 窗口层级限制；不能以 Rust GUI 跨平台编译能力保证 Linux 各显示协议可用。
- [RFC 6455](https://www.rfc-editor.org/rfc/rfc6455)：区分未加密 `ws` 与加密 `wss`；握手允许鉴权头。共享令牌不提供音频/字幕的传输保密性，不能把鉴权与 TLS 混为一谈。D-06 的局域网普通 WS 边界已获用户批准，程序鉴权尚未实现。

## 技术实验清单

| 实验 | 需要输入 | 应留下的证据 | 失败处理 |
| --- | --- | --- | --- |
| ASR 流式模型 | 用户 GGUF、明确语言、CPU/GPU、受控 PCM | 模型 hash、引擎 revision、事件格式、草稿/定稿、静音/长句、时延/内存 | 记录不兼容与限制，再评估适配/后端，不换模型冒充通过 |
| 翻译模型 | 用户 UD GGUF、源/目标语言、上下文样本 | 加载、模板/停止条件、质量、迟到任务可取消、内存与设备 | 核查量化与引擎版本/许可 |
| VAD/定稿 | 静音、连续讲话、短句与长句样本 | 是否需额外资产、定稿条件、长句上限、时间轴 | 把附加模型纳入明确资产清单 |
| 私有进程生命周期 | 实际 ASR/翻译程序 | 模型加载就绪、子进程归属、stdio/HTTP 上限、退出/超时 | 评估 FFI 或可控 API |
| 首发采音/窗口 | 确认 OS、真实设备与显示器 | 播放采集、切换、字体、透明置顶、穿透恢复、多屏/DPI | 明确限制，不能改用麦克风代替 |

没有下载模型、安装依赖、拉取容器、执行推理或对外部署。
