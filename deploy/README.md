# 历史部署骨架

> 版本边界（2026-11-02）：本目录保留 2026-09-27 的 full/server/client 三角色方案，不是当前产品发行设计，也不授权继续旧部署任务。当前正式入口是 `apps/synesthesia`，输入、本地/云识别与可选翻译分别选择；Fun-ASR 不要求自建服务器或本地模型。

以下记录旧方案的安装包、构建产物、便携运行目录与部署模型，覆盖 `synesthesia-full`、`synesthesia-server` 和 `synesthesia-client`。

**本目录交付的是历史部署骨架，不是可运行的软件发行版。** 包含目录约定、草案配置、Windows 启动入口、容器示例和安装包结构预览；不携带程序、模型或已实现的安装器。仓库已有 Rust 配置检查入口及部分原生适配器，不能将本目录的未实现状态推广到整个仓库，也不能将它们视为完整实时产品。

仅在源码仓库中使用以下导航：[旧 PRD](../docs/history/2026-09-27-prd.md)、[历史资料与原位链接](../docs/history/index.md)、[当前架构](../docs/architecture.md)、[状态](../docs/status.md)、[路线](../docs/roadmap.md)。独立骨架 ZIP 只包含本目录文件，不包含这些外层文档；包内说明和脚本仍仅用于旧结构预览。

| 产品 | 用户设备上的职责 | 模型与推理依赖 |
| --- | --- | --- |
| `synesthesia-full` | 采音、识别、翻译、悬浮字幕，全流程使用本机硬件 | 需要 |
| `synesthesia-server` | 检测与启用计算资源，等待客户端连接并提供推理 | 需要 |
| `synesthesia-client` | 本机采音、连接服务端、悬浮字幕 | 不需要 |

## 从哪里读起

- [MENTAL-MODEL.md](MENTAL-MODEL.md)：产品、安装包、模型、资源与进程的关系。
- [PACKAGE-LAYOUT.md](PACKAGE-LAYOUT.md)：源码、安装包和安装后的三种目录形态。
- [INSTALL.md](INSTALL.md)：安装、离线复用、移动、升级和卸载约定。
- [VALIDATION.md](VALIDATION.md)：当前状态与未来发行验收边界。
- [scripts/CONTRACTS.md](scripts/CONTRACTS.md)：待实现脚本的接口和行为。
- [docker/README.md](docker/README.md)：容器服务端的部署边界。
- [prompts/reuse.txt](prompts/reuse.txt)：继续实现本目录的任务上下文。

## 当前文件的真实含义

| 文件 | 含义 |
| --- | --- |
| `assets.example.json` | 未就绪的资产清单示例；空 URL、版本或哈希不能进入正式安装 |
| `payload/config/*.example.json` | 旧产品配置草案，不是已验证的 schema 1/2 输入 |
| `install.bat` | Windows 安装入口；找不到尚未实现的安装器时明确退出 |
| `payload/start-*.bat` | 启动对应项目二进制；不存在时明确报错，不模拟字幕运行 |
| `docker/compose.example.yaml` | 服务编排草案；需要真实镜像与经过确认的运行契约 |
| `package-files.sha256` | 本目录骨架文件的完整性清单，不是程序发行签名 |
| `packages/synesthesia-windows-install-scaffold.zip` | 安装包结构预览，明确不含可运行程序或模型 |

`.example` 文件需要在实现与发行阶段确认并生成正式配置。不存在的主体脚本仅在契约文档中列出；不使用空壳脚本报告安装或验证成功。

## 便携交付原则

1. 应用程序、私有依赖、配置、日志、缓存与临时文件尽量集中在运行目录。
2. 安装目录由用户指定，路径解析以程序或启动入口所在目录为准。
3. 默认不要求管理员权限，不修改全局 PATH，不安装系统级 Python、包管理器或 GPU 开发工具链。
4. 模型与软件分开交付：当前约定不下载、复制或打包模型，由用户手动提供。
5. 首次安装先在临时构建目录完成校验，再发布到目标目录；失败不接管用户已有目录。
6. GPU 驱动、系统音频权限与操作系统组件属于外部运行条件，不承诺跨设备复制后必定可用。
7. 自主实现的程序与第三方推理依赖分别管理；随包分发组件时保留对应许可。

正文中的角色与部署约定属于旧 PRD。当前产品以仓库根 PRD v2 为准，发行工作按当前路线另行确认。本次只同步历史说明、清单和骨架预览，不新增安装或运行能力。
