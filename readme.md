# Synesthesia

Rust 桌面实时字幕工具：选择麦克风或系统音频，使用本地模型或第三方服务识别为文字，按需翻译，在透明置顶窗口显示。

~~~text
麦克风 / 系统音频 → 语音识别（本地 / Fun-ASR 等服务）→ 原文
                                                       ↓ 可选翻译
                                              原文 / 双语 / 译文字幕
                                                       ↓
                                                 透明置顶窗口
~~~

**当前是可构建的 Rust 工程基础，实时采音、识别和透明窗口尚未整合成完整产品。** 已使用的 Python 原型保留原地作为体验参照，本仓库不依赖 Python 运行，不携带其真实凭据。

## 文档

- [当前 PRD](prd.md)：产品目标、范围与验收。
- [架构](docs/architecture.md)：模块、独立引擎与配置兼容。
- [状态](docs/status.md)：已实现、未实现与历史验证边界。
- [路线](docs/roadmap.md)：逐步形成可用闭环。
- [历史资料导航](docs/history/index.md)：旧需求、部署骨架、原位链接与任务状态说明。
- [配置规范](.trellis/spec/backend/pipeline.md)：schema 2 的实际接口。
- [历史契约](docs/contracts.md)：旧三角色 schema 1 / WS v1。

## 开发

Rust 1.90.0 / edition 2024，使用 Cargo.lock。Linux 可复用现有 Docker 包装器，无需宿主 Rust：

~~~sh
./hako rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy --no-self-update
./hako cargo run --locked -- --help
./hako cargo run --locked -- --check-config config/examples/microphone-fun-asr.json
~~~

首次依赖下载需要网络；缓存齐全后可使用 HAKO_NETWORK=none，并给 Cargo 加 --offline。包装器是 Linux 开发工具，不是 Windows 产品运行依赖。

已有原生 Rust 时，在仓库根目录直接运行：

~~~sh
cargo run --locked -- --help
cargo run --locked -- --check-config config/examples/microphone-fun-asr.json
~~~

默认入口是 apps/synesthesia。目前提供帮助、版本与配置结构检查，不读取密钥、不访问设备/服务/模型；未实现的运行命令明确失败。

全工程检查（原生环境移除 ./hako）：

~~~sh
./hako cargo fmt --all -- --check
./hako cargo check --workspace --all-targets --locked
./hako cargo clippy --workspace --all-targets --locked -- -D warnings
./hako cargo test --workspace --all-targets --locked
./hako cargo build --workspace --locked
~~~

[GitHub 工作流](.github/workflows/rust.yml) 使用相同检查。纯逻辑和 Linux 构建通过不能代替原生设备、窗口或 GPU 验收。

## 配置

| 示例 | 输入 | ASR | 翻译 |
| --- | --- | --- | --- |
| [microphone-fun-asr.json](config/examples/microphone-fun-asr.json) | 麦克风 | Fun-ASR | 关闭 |
| [system-fun-asr.json](config/examples/system-fun-asr.json) | 系统音频 | Fun-ASR | 关闭 |
| [microphone-local.json](config/examples/microphone-local.json) | 麦克风 | 本地 | 关闭 |
| [system-local-translation.json](config/examples/system-local-translation.json) | 系统音频 | 本地 | 本地到中文 |

这是已实现解析器的输入，**不是已经可运行的产品组合**。输入、ASR 和翻译分别选择；更多服务需要真实适配器，不接受任意服务名冒充支持。

云配置仅填写 api_key_env 环境变量名，不填写 API key；结构检查无需密钥存在。运行时接入后才由鉴权边界读取其值。schema 2 相对模型路径从配置文件实际父目录解析，不自动下载模型。

个人配置放在已忽略的 config/local/；模型使用 models/ 或外部绝对路径。示例本地路径相对于示例文件目录，实际使用时将配置放到自己的运行目录并调整路径。真实凭据、录音、字幕正文不得进入示例或诊断。

## 目录

~~~text
apps/
  synesthesia/            正式桌面入口，目前为配置检查
  synesthesia-full/       保留的旧 full 入口
  synesthesia-server/     保留的旧 server 入口
  synesthesia-client/     保留的旧 client 入口
crates/
  synesthesia-core/       配置、身份、字幕状态、预算与诊断
  synesthesia-audio/      既有系统回环及格式规范化
  synesthesia-inference/  既有本地推理实验与原生 helper
  synesthesia-desktop/    纯呈现投影，原生窗口待实现
  synesthesia-protocol/   旧远程会话协议
config/examples/         无机密的 schema 2 示例
docs/                    架构、状态、路线和历史需求
.trellis/                规范、任务与验证证据
deploy/                  历史部署骨架，不是新发行包
~~~

旧 full/server/client 仍使用 schema 1，不接受新配置。直接云识别不要求先运行自建服务器；旧任务与部署 ZIP 不代表当前产品的交付承诺。

许可证见 [license](license)。
