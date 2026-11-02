# 目录与构建形态

> 历史方案：以下目录及三角色发行布局来自 2026-09-27 的部署草案，不是当前 Rust 工程目录或新产品发行设计。当前正式入口为 `apps/synesthesia`。版本边界及源码文档入口见 [部署说明](README.md)。

产品使用名称 `synesthesia`。以下区分当前文件与计划产物；列出的程序和主体脚本尚未构建。

## 1. 源码中的部署目录

```text
deploy/
  README.md
  INSTALL.md
  MENTAL-MODEL.md
  PACKAGE-LAYOUT.md
  VALIDATION.md
  assets.example.json
  package-files.sha256
  install.bat
  scripts/
    CONTRACTS.md            主体脚本接口；脚本未实现
  payload/
    start-full.bat
    start-server.bat
    start-client.bat
    start-debug.bat
    config/
      full.example.json
      server.example.json
      client.example.json
  docker/
    README.md
    compose.example.yaml
    server.example.json
  prompts/
    reuse.txt
  packages/
    synesthesia-windows-install-scaffold.zip
    synesthesia-windows-install-scaffold.zip.sha256
```

计划增加 `scripts/install.ps1`、`scripts/verify-install.ps1`、`scripts/build-package.ps1` 和 `scripts/debug-launch.ps1`。具体实现顺序由开发任务确定；当前入口在脚本或程序缺失时失败退出。

## 2. 构建产物与正式安装包

Rust 构建分别产生三个产品入口。发行包按产品、版本、系统和架构命名，例如 `synesthesia-full-VERSION-windows-x86_64.zip`。构建目录位置与 Rust workspace 划分由项目设计确定。

```text
源码 + 构建配置 + 固定依赖与模型兼容信息
  → 三种产品的构建产物
  → 校验过的平台/后端私有依赖
  → 按产品组装软件载荷和配置默认值
  → 生成文件清单、许可材料与校验和
  → 软件安装包 / 便携包
```

正式 Windows 安装器包的目标形态：

```text
synesthesia-PRODUCT-VERSION-windows-x86_64-installer/
  install.bat
  INSTALL.md
  assets.json               已固定版本、URL/来源与 SHA256
  package-files.sha256
  scripts/
    install.ps1
    verify-install.ps1
    debug-launch.ps1
  payload/
    synesthesia-PRODUCT.exe          自主构建的产品程序
    start-PRODUCT.bat
    start-debug.bat
    config/
      settings.json          该角色经过验证的默认配置
    runtime/                仅包含该产品需要的私有依赖
    licenses/               自身与所分发第三方组件的许可
```

软件载荷可以随包携带，也可以由安装器从本项目发行源和依赖发行源获取；两种方式均须固定版本并校验。模型不在当前默认交付范围内。客户端包不包含计算后端和模型。

离线可复用的安装器不等于最终运行目录；已经完整安装的便携目录可以整体复制。

## 3. 全功能版运行目录

```text
synesthesia-full/
  synesthesia-full.exe
  start-full.bat
  start-debug.bat
  INSTALL.md
  .synesthesia-portable.json
  app/                      安装后校验与调试工具
  runtime/                  识别与翻译所需私有库或辅助程序
  licenses/
  models/                   用户提供的模型，安装后初始为空
  config/
    settings.json
    assets.json
  cache/
    downloads/              已验证的软件安装源，可供离线安装
    inference/              推理后端按需使用的缓存
  logs/
  temp/
```

## 4. 服务端运行目录

```text
synesthesia-server/
  synesthesia-server.exe
  start-server.bat
  start-debug.bat
  INSTALL.md
  .synesthesia-portable.json
  app/
  runtime/
  licenses/
  models/
  config/
    settings.json
    assets.json
  cache/
    downloads/
    inference/
  logs/
  temp/
```

服务端不交付桌面悬浮窗依赖。根据实际链接方式决定哪些库需要 app-local 分发，不能因为目录存在就复制不需要的 DLL。

## 5. 客户端运行目录

```text
synesthesia-client/
  synesthesia-client.exe
  start-client.bat
  start-debug.bat
  INSTALL.md
  .synesthesia-portable.json
  app/
  runtime/                  仅在音频/窗口等依赖需要时出现
  licenses/
  config/
    settings.json
    assets.json
  cache/
    downloads/
  logs/
  temp/
```

客户端不要求 `models/` 或推理缓存目录。模板配置按角色分开，正式包中选择对应模板生成 `config/settings.json`；不能复制模板后就宣称程序已支持该格式。

## 6. 平台差异与可变状态

Windows 使用 `.exe` 和 `.bat`；Linux/macOS 的可执行文件、启动入口及安装包格式待平台实现确定，沿用相同的角色边界和用户状态约定，不预先宣称已有原生包。

程序与私有依赖属于可校验的软件文件；配置、模型、日志、缓存和临时文件属于可变状态。程序完整性清单不能把允许用户修改的配置当作损坏文件。

镜像中的程序与宿主挂载的模型、配置、缓存、日志同样分离。镜像构建缓存不作为运行数据交付。
