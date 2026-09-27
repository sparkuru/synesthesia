# 部署交付

本目录定义本项目的安装包、构建产物、便携运行目录与部署心智模型。产品名称为 `synesthesia`，覆盖 `synesthesia-full`、`synesthesia-server` 和 `synesthesia-client`。

**当前交付的是部署骨架，不是可运行的软件发行版。** 已包含目录约定、配置示例、Windows 启动入口、容器配置示例和安装包结构预览。Rust 程序、安装器主体、校验器主体、容器镜像与实际推理尚未实现。

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
| `payload/config/*.example.json` | 产品配置草案，尚未声明为程序支持的配置格式 |
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

完整产品需求见 [../prd.md](../prd.md)。
