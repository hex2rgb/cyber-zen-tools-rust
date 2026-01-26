# GGUF Tokenizer 说明

## 理论 vs 实际实现

理论上，**GGUF 格式是自包含的**，内部存储了完整的 tokenizer 信息（词表、特殊 token 等）。

但在实际的 Candle Rust 实现中，`candle-transformers::models::quantized_qwen2::ModelWeights` 的 `from_gguf` 方法**仍然需要使用外部的 `tokenizers` crate** 来处理文本的编码和解码。

## 当前实现

当前代码使用 `tokenizers::Tokenizer` 来：
1. 将文本编码为 token IDs (`tokenizer.encode()`)
2. 将 token IDs 解码为文本 (`tokenizer.decode()`)
3. 获取词汇表信息 (`tokenizer.get_vocab()`)

因此，即使 GGUF 文件理论上包含 tokenizer 信息，Candle 的实现仍然需要 `tokenizer.json` 文件。

## 解决方案

需要从原始模型仓库下载 `tokenizer.json`：

```bash
huggingface-cli download Qwen/Qwen2.5-Coder-0.5B-Instruct \
  tokenizer.json \
  --local-dir ~/.cyber-zen/models/qwen2.5-coder-0.5b-gguf \
  --local-dir-use-symlinks False
```

## 参考

- GGUF 格式：https://github.com/ggerganov/ggml/blob/master/docs/gguf.md
- Candle Transformers：https://github.com/huggingface/candle/tree/main/candle-transformers
- Qwen2.5-Coder 模型：https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct

