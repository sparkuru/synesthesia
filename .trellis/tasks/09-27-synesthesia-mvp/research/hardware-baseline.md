# 首轮硬件验收基准

检查日期：2026-09-27。用户在询问算力基准后提供 `ssh wkyuu@192.168.9.2`，
本轮仅执行系统与硬件只读查询；未安装软件、启动模型、修改注册表或驱动配置。

## 系统与资源

| 项目 | 实测结果 | 证据 |
| --- | --- | --- |
| OS | Microsoft Windows 11 IoT 企业版 LTSC，64 位 | Win32_OperatingSystem |
| OS version / build | 10.0.26100 / 26100 | Win32_OperatingSystem |
| CPU | AMD Ryzen 9 7950X，16 核 / 32 逻辑处理器 | Win32_Processor |
| 安装内存 | 64 GiB | Win32_PhysicalMemory.Capacity 总和 |
| 系统可见物理内存 | 63.11 GiB | Win32_ComputerSystem.TotalPhysicalMemory；不是空闲内存 |
| 独显 | AMD Radeon RX 9070 XT | Win32_VideoController；Vulkan discrete device |
| 独显驱动报告显存 | 17,095,983,104 bytes = 15.921875 GiB | 显示驱动注册表 HardwareInformation.qwMemorySize |
| 独显 WMI 驱动版本 | 32.0.31041.1004 | Win32_VideoController.DriverVersion |
| 集显 | AMD Radeon(TM) Graphics；驱动报告 512 MiB | CIM / 显示驱动注册表 |
| 集显 WMI 驱动版本 | 32.0.21045.5002 | Win32_VideoController.DriverVersion |

Win32_VideoController.AdapterRAM 对独显报告 4,293,918,720 bytes，不能据此记录为 4 GiB 显卡。
显存记录采用 64 位驱动字段；这是驱动报告值，不是本项目模型可用显存或峰值显存测量。

## Vulkan 只读检查

已有系统 Vulkan loader 和 `vulkaninfo`。执行 `vulkaninfo --summary`，退出码 0。

| 项目 | 结果 |
| --- | --- |
| Vulkan instance version | 1.4.341 |
| 独显 API version | 1.4.349 |
| 独显 driverName / driverInfo | AMD proprietary driver / 26.8.1 (LLPC) |
| 独显 vendorID / deviceID | 0x1002 / 0x7550 |
| 集显 API version | 1.4.315 |
| 本次枚举顺序 | GPU0 集显；GPU1 RX 9070 XT |

枚举序号不是稳定配置身份；后续设备选择必须区分集显与独显，不假设第一个 GPU 是目标设备。
Vulkan 成功枚举只证明驱动工具能发现设备，不证明 CrispASR、llama.cpp 或指定 GGUF 已能在该设备推理。

工具报告两条现有图层配置告警：Epic EOS overlay 图层 JSON 缺失、AMD switchable graphics 图层重复。
告警未阻止本次设备枚举，但可能影响后续模型实验的环境解释；本轮没有修复或禁用图层。

## 连接与执行记录

- 系统级 SSH include 存在权限错误，改用可读的用户配置 `-F /home/wkyuu/.ssh/config`。
- 普通 sandbox 禁止 socket；获批的网络执行成功连接指定目标，使用现有 key/agent、BatchMode 与严格 host key 检查。
- 最小 `true` 探测返回 PowerShell 未识别命令，说明已连接且远端默认 shell 为 PowerShell；不是认证失败。
- 之后 CIM 查询、显示驱动注册表查询与 `vulkaninfo --summary` 均退出码 0。
- 未使用 sudo/提权、密码模板、远端临时文件、文件上传、端口转发或服务启动。

可复查的只读查询为 Get-CimInstance Win32_OperatingSystem / Win32_Processor /
Win32_ComputerSystem / Win32_PhysicalMemory / Win32_VideoController，
以及显示驱动 `Control\Video\*\0000` 下 DriverDesc 与 HardwareInformation.qwMemorySize。
不采集序列号、凭据、文件内容、运行应用或其他无关主机状态。

## 对设计与后续验证的影响

1. 首轮采音/悬浮窗实机针对上述 Windows 11 环境；用户后续确认首版支持目标为 Windows 10/11 x64，不限定 build 26100。Windows 10 仍需独立兼容与运行证据，不能由本机查询代替。
2. 用户后续批准首轮优先 RX 9070 XT，先验证 Vulkan；CPU 可手动选择用于兼容与性能对照。
3. 用户批准 GPU 失败明确报错、不静默回退至 CPU；引擎的设备标识映射仍需兼容实验，不能以 GPU0 默认选择集显。
4. 两款模型的本地文件与 SHA256 现已由 [模型基准](model-baseline.md) 固定；发行来源、引擎版本、模型加载、实际计算设备、显存峰值与吞吐尚未验证。
5. 用户后续明确 Windows 不用 Docker；该机器仅用于 Windows 原生构建/运行/设备验收，可选 Docker CPU 基线另在 Linux 宿主验证，容器 GPU 属后续。

路线依据补充：[llama.cpp 官方构建文档](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md) 提供 Windows Vulkan 构建路线。
这仅支持将 Vulkan 列为翻译运行时的实验候选；不证明指定 UD 量化模型可用，也不证明 ASR 后端在该 GPU 上可用。
