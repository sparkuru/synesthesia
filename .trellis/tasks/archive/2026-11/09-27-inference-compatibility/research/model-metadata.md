# Existing model metadata inspection

On 2026-09-27, [the bounded inspection script](read-gguf-metadata.ps1) read the two approved files on the Windows baseline machine. It uses read-only streams and skips tensor data. It emits only selected scalar metadata and the embedded chat template's byte length/hash, never its prompt text. The script ran through noninteractive PowerShell over the existing SSH connection, exit code 0; no remote script/model file was written and no inference ran.

| Field | Qwen ASR | Hy translation |
| --- | --- | --- |
| File | `qwen3-asr-1.7b-q4_k.gguf` | `Hy-MT2-1.8B-UD-Q3_K_XL.gguf` |
| GGUF version | 3 | 3 |
| Architecture | `qwen3asr` | `hunyuan-dense` |
| Tensor count | 710 | 354 |
| Metadata count | 34 | 46 |
| Tokenizer model | `gpt2` | `gpt2` |
| General quantization version | 2 | 2 |
| General file type | 12 | 12 |
| General name | Not selected/present in output | `Hy-Mt2-1.8B` |
| Blocks | Not selected/present in output | 32 |
| Context length | Not selected/present in output | 262144 |
| Embedding length | Not selected/present in output | 2048 |
| BOS / EOS token ID | Not selected/present in output | 120000 / 120020 |
| Embedded chat template UTF-8 bytes | Not selected/present in output | 654 |
| Embedded chat template SHA256 | Not selected/present in output | `b7491ec0e9c869dfce20f2176758099bf248d979dd05530ede99deb21698acee` |

The inspector limits metadata entry count, array length/depth, selected string length, and stream seeks. It is an experiment helper for these specific files, not the application's GGUF loader. No claim about fields outside its whitelist follows from their absence in this table.

`general.file_type=12` does not describe all tensor types in a mixed quantization file. The publisher's exact byte identity and pinned engine architecture/tensor support must be checked separately. A readable header does not prove GPU initialization, model execution, translation quality, prompt correctness, cancellation, or resident-model lifecycle support.

## Actual mixed tensor types

A second read-only run of the extended script also walked tensor descriptors, exit code 0. It still did not read tensor payloads or execute inference. Type labels follow the [pinned GGML enum](https://github.com/ggml-org/llama.cpp/blob/7fe450e19305b828c199d602c23a8337aaa1f03b/ggml/include/ggml.h#L368).

| Numeric type | Label | Qwen ASR tensors | Hy translation tensors |
| --- | --- | --- | --- |
| 0 | F32 | 362 | 129 |
| 1 | F16 | 3 | 0 |
| 8 | Q8_0 | 147 | 0 |
| 11 | Q3_K | 0 | 66 |
| 12 | Q4_K | 198 | 72 |
| 13 | Q5_K | 0 | 71 |
| 14 | Q6_K | 0 | 1 |
| 18 | IQ3_XXS | 0 | 5 |
| 21 | IQ3_S | 0 | 10 |
| Total | | 710 | 354 |

The translation preset therefore uses both K-quant and IQ3 tensors. The fixed [llama.cpp Vulkan implementation](https://github.com/ggml-org/llama.cpp/blob/7fe450e19305b828c199d602c23a8337aaa1f03b/ggml/src/ggml-vulkan/ggml-vulkan.cpp#L2758) contains IQ3_XXS/IQ3_S matvec, dequantization and row-gather pipelines, plus type support dispatch. This is source-level evidence for these formats, not proof of successful graph execution on the RX 9070 XT. Actual loaded/offloaded buffers and inference remain required.

Model file hashes and stable-size observations remain in the [parent model baseline](../../09-27-synesthesia-mvp/research/model-baseline.md). Source mapping and engine compatibility are recorded in [native engine readiness](native-engine-readiness.md).
