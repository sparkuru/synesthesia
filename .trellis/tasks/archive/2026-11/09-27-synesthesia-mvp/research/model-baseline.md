# 首轮模型文件基准

日期：2026-09-27。目标主机为硬件基准中的 `wkyuu@192.168.9.2`，用户提供模型目录 `E:\tmp\models`。
本轮通过 SSH / PowerShell 只读枚举指定目录并计算两款目标文件的 SHA256；未复制、修改、下载或加载模型。

## 实测文件身份

| 用途 | 文件名 | 文件大小（bytes） | SHA256 |
| --- | --- | --- | --- |
| ASR | `qwen3-asr-1.7b-q4_k.gguf` | 1,490,915,200 | `EC197CEF7CCC589FDCAE1BECC3F4A3DE119D0A41E790B898B519B1A048DAD8D4` |
| 翻译 | `Hy-MT2-1.8B-UD-Q3_K_XL.gguf` | 989,983,712 | `BF575B6238C6AE3C35DB73AF36BC0A749AEDF8DE17426C1B506E2D332DD9C9CA` |

目录根层枚举到上述两款 GGUF，无子目录；没有把结果外推为整台机器不存在其他模型。
哈希读取前后检查文件大小与 LastWriteTimeUtc 一致，查询与哈希命令退出码均为 0。
首次目录检查连接超时，一次有界重试成功；不是认证失败，未生成密码模板或改动 SSH/网络配置。

## 使用边界

- 上述值固定本轮测试对象，后续运行前复核；文件发生变化时更新基准并重新验收。
- SHA256 初次检查为本地指纹；实现准备阶段已对照发布方固定 revision 的 LFS 元数据，字节数及哈希精确匹配，见下节。这识别公开发布的字节内容，不证明用户历史下载方式或模型执行兼容。
- 文件名与哈希不能证明 GGUF 元数据、量化算法、模型架构、tokenizer、许可证或引擎加载兼容。
- 发布 revision 已固定；完整许可/notice、所需 VAD/其他资产与实际推理仍归 C2 兼容实验。
- 目录内未发现额外 GGUF 不表示无需 VAD；由实际 ASR 实验决定资产清单。
- 此路径是当前 Windows 验收环境配置，不写为软件默认路径，不打包模型，不从客户端加载模型。
- Windows 本机计算适配器可用绝对路径读取用户外部模型目录；目录含空格时通过参数 API 传递，不拼接 shell 命令。
- 远端 server 使用该主机自身路径；轻客户端只连接服务，不依赖 `E:\tmp\models`。
- Docker 需要明确的宿主目录挂载与容器内模型路径，不能把 Windows 盘符直接当作容器路径。

## 已执行的只读检查

Get-ChildItem -LiteralPath 指定模型目录（根层 GGUF 与子目录名称），
Get-Item 两款文件，以及 Get-FileHash -LiteralPath 两款文件 -Algorithm SHA256。
保留字节大小与哈希即可复查；不保留无关文件清单、模型内容或凭据。

## 实现准备补充：发布内容与内部元数据

2026-09-27 研究代理匹配来源后，主会话再次读取固定 revision 的发布方 API，确认目标文件 LFS SHA256 与上表完全一致：

- ASR：[cstr 发布元数据](https://huggingface.co/api/models/cstr/qwen3-asr-1.7b-GGUF/revision/674df5d44b50a63e7102a18895ed20e3f91de301?blobs=true)，revision `674df5d44b50a63e7102a18895ed20e3f91de301`。
- 翻译：[unsloth 发布元数据](https://huggingface.co/api/models/unsloth/Hy-MT2-1.8B-GGUF/revision/0378cc2780f462f0fd397d0cdd244d2f769de40e?blobs=true)，revision `0378cc2780f462f0fd397d0cdd244d2f769de40e`。

只读 GGUF 元数据与 tensor 描述表解析确认 ASR 架构 `qwen3asr`、翻译架构 `hunyuan-dense`；后者含 Q3/Q4/Q5/Q6_K 与 IQ3_XXS/IQ3_S 混合量化，不能仅凭名称当作纯 Q3_K。翻译模板的 654 UTF-8 字节与 SHA256 匹配固定 Tencent 上游模板。详细类型分布、脚本及边界见 [C2 模型元数据](../../09-27-inference-compatibility/research/model-metadata.md) 与 [C2 引擎研究](../../09-27-inference-compatibility/research/native-engine-readiness.md)。

这些检查均未修改/下载模型权重或运行推理；实际独显加载、流式结果、取消/常驻及质量仍未验收。
