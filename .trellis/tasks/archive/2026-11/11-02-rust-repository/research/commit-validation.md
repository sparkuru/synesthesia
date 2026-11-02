# 归档与提交前验证

日期：2026-11-02。范围：1102 的 Rust 工程基础、现行文档/规范、配置示例、保留的兼容入口及补充整理成果。依据用户最新“归档、提交”授权复核；未修改产品源码、暂存文件、提交、归档或任务指针。

## 本次重新执行的 Rust 检查

全部通过既有 `rust:1.90-bookworm` hako 临时容器执行，网络为 none，使用缓存依赖，不安装或下载。项目固定 Rust 1.90.0、edition 2024，所有 workspace 成员参加检查。

| 实际命令 | 执行者 | 结果 |
| --- | --- | --- |
| `HAKO_NETWORK=none ./hako cargo fmt --all -- --check` | 主会话 | 通过，exit 0 |
| `HAKO_NETWORK=none ./hako cargo check --workspace --all-targets --locked --offline` | trellis-check | 通过，exit 0 |
| `HAKO_NETWORK=none ./hako cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | trellis-check | 通过，exit 0 |
| `HAKO_NETWORK=none ./hako cargo test --workspace --all-targets --locked --offline` | trellis-check | 75 项通过，0 失败、0 忽略，exit 0 |
| `HAKO_NETWORK=none ./hako cargo build --workspace --locked --offline` | trellis-check | 通过，exit 0 |

测试分布：正式入口 1、audio 18、core 36、desktop 5、inference 8（含 benchmark 示例 1）、protocol 7。以上 75 项是本次提交前新执行的结果，不是引用先前验证记录。

首个普通沙箱 fmt 尝试在 Cargo 启动前因 Docker socket `unix:///var/run/docker.sock` 访问被拒而 exit 126。主会话通过 `exec_command` 的 `require_escalated` 单次审批成功执行 fmt，随后本复核使用同类窄执行审批完成其余四项。未改变策略、socket 权限、命令前缀配置或网络设置；没有剩余执行阻塞。

## 全范围静态复核

- workspace 默认成员为正式 `apps/synesthesia`，四个入口与所有 crate 均仍参加 workspace 检查；旧三角色入口没有增加模型或窗口依赖。
- schema 2 将输入、识别和翻译独立建模；禁用翻译不接受模型字段，云配置仅保存密钥环境变量名。解析与配置检查不读取环境变量值、访问云服务、加载模型或打开设备。
- 检查新入口的安全错误、配置大小与字段约束、配置文件父目录路径基准，以及 desktop 借用投影的单一状态所有权；相关断言由本次 schema、CLI、状态与呈现测试覆盖。
- 新入口明确拒绝未整合的运行命令；规范和状态文档不把配置可解析、历史原生适配器或纯逻辑检查当成当前实时产品验收。
- 既有 Session/ComputeAdapter 双模型耦合、final 终态和 Stop 取消清空仍是明确记录的迁移边界，本轮未伪装为已实现独立运行时或正常收尾。
- 补充整理的元数据、历史快照、链接、清单及 ZIP 完整性已独立复核，详见同目录 `cleanup-review.md`；归档后引用与最终暂存边界由主会话处理。
- `bash -n hako`、`shellcheck hako`、`shfmt -d -i 4 hako` 及 `git diff --check` 均通过。

最终暂存检查额外发现首次入库的 `rust-toolchain.toml` 末尾多一空行，主会话仅将文件结尾规范为单个换行，TOML 内容不变；之后重新执行暂存差异检查。

## 结论与限制

未发现阻塞本任务本地提交的实现或检查问题。Lint、类型检查、测试、构建均通过；没有新增或删除测试，也没有压制警告。

未执行云调用、真实采音、模型推理、原生窗口、C++ helper、安装器或 Compose。Linux 全工程检查不证明 Windows/macOS 设备及窗口支持，也不等于 GitHub CI 远端执行。未读取用户根 `archive/`，未检查或修改独立 Trellis 工具升级源码。
