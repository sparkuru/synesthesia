# Rust 正式仓库整理验证

日期：2026-11-02。执行对象为 /home/wkyuu/cargo/repo/36-synesthesia，Linux 开发宿主，Rust 1.90.0，既有 rust:1.90-bookworm 镜像；构建检查通过 hako 在无网络、无服务端口的临时容器中执行。

## 最终结果

以下 Cargo 命令均使用 HAKO_NETWORK=none ./hako 前缀：

| 命令 | 结果 |
| --- | --- |
| cargo fmt --all -- --check | 通过 |
| cargo check --workspace --all-targets --locked --offline | 通过 |
| cargo clippy --workspace --all-targets --locked --offline -- -D warnings | 通过，无警告抑制 |
| cargo test --workspace --all-targets --locked --offline | 75 项测试通过，0 失败、0 忽略 |
| cargo build --workspace --locked --offline | 通过，包含新入口及保留三角色 |
| cargo run --locked --offline -- --check-config config/examples/microphone-fun-asr.json | 通过；默认入口为 synesthesia |

测试分布：新入口 1、audio 18、core 36（含新管线 7）、desktop 5、inference 8（含 benchmark 示例 1）、protocol 7，共 75。新增 13 项测试，其余原有测试保留。

四个 config/examples 配置均通过编译出的 synesthesia --check-config 检查；--help 和 --version 成功；不存在的配置和未实现 --run 均返回 2，并提供安全错误，不伪装运行就绪。无需真实密钥或模型即可完成结构检查。

文档/工程检查：git diff --check 通过；当前 37 个明确文件的换行/行尾空白检查通过；44 个本地文档链接和 4 个 JSON 示例检查通过。工作流 YAML 解析成功，含预期 7 个步骤；GitHub CI 尚未远端执行。Trellis implement/check 上下文各 5 个真实引用并通过 validate。

## 变更与保留

- 新 PRD、README、架构/状态/路线、schema 2、统一 CLI、纯呈现模块、配置示例与 CI 已直接保存正式目录。
- 旧 PRD、原桌面依赖清单和锁文件保存在 docs/history。旧 native audio/inference、core/protocol、三角色入口、历史任务和 deploy 未被删除或当作已完成发行版。
- desktop 原先缺失的模块声明改为真实纯呈现实现；未使用的 GUI 依赖移除，所有 workspace 成员仍在检查范围。Cargo.lock 已据此更新。
- 初始工作区已有未提交内容和规范文件删除，本轮没有重置、提交或推送，也没有接管其他会话的本地任务指针。
- 原型及其真实配置保持原处；用户私密 archive 不读取、不修改，并确认被 Git 忽略。

## 验证边界

临时 /tmp 工作副本曾随环境重启丢失，以上结果均是在恢复到正式目录后重新执行的最终检查。

本轮没有云请求、麦克风/系统音频采集、模型推理或原生窗口验收，没有运行 Windows/macOS 验证。历史 Windows/GPU 记录仍为历史证据。新程序当前提供配置检查；云适配、麦克风、独立识别/翻译运行时和真实透明窗口按 docs/roadmap.md 后续实施。
