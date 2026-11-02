// Pipeline derived from CrispASR's MIT-licensed Qwen adapter; see NOTICE.
#include "synesthesia_control.h"
#include "qwen3_asr.h"
#include "core/bpe.h"
#include <algorithm>
#include <cmath>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <stdexcept>

extern "C" bool qwen3_asr_synesthesia_prepare(qwen3_asr_context*);
extern "C" void qwen3_asr_synesthesia_clear(qwen3_asr_context*);
static qwen3_asr_context* context = nullptr;
using Float = std::unique_ptr<float, decltype(&std::free)>;
using Int = std::unique_ptr<int32_t, decltype(&std::free)>;
static void check() { if (synesthesia::cancelled) throw std::runtime_error("cancel"); }
static int token(const char* text) {
    int count = 0;
    Int ids(qwen3_asr_tokenize(context, text, &count), std::free);
    if (!ids || count != 1) throw std::runtime_error("token");
    return *ids;
}
static void clear() { if (context) qwen3_asr_synesthesia_clear(context); }
static std::string transcribe(const std::vector<uint8_t>& data) {
    check();
    synesthesia::stage = 1;
    if (data.size() < 3 || data.size() > 320001 || (data.size() - 1) % 2 || (data[0] != 1 && data[0] != 2)) throw std::runtime_error("audio");
    std::vector<float> audio((data.size() - 1) / 2);
    for (size_t i = 0; i < audio.size(); ++i) {
        const auto raw = uint16_t(data[1 + i * 2]) | (uint16_t(data[2 + i * 2]) << 8);
        audio[i] = float(int16_t(raw)) / 32768.0f;
    }
    int n_mels = 0, t_mel = 0;
    Float mel(qwen3_asr_compute_mel(context, audio.data(), int(audio.size()), &n_mels, &t_mel), std::free);
    std::fill(audio.begin(), audio.end(), 0);
    check();
    if (!mel) throw std::runtime_error("mel");
    int frames = 0, dimension = 0;
    synesthesia::stage = 2;
    Float encoded(qwen3_asr_run_encoder(context, mel.get(), n_mels, t_mel, &frames, &dimension), std::free);
    check();
    if (!encoded || frames <= 0 || frames > 1024 || dimension <= 0 || dimension > 4096) throw std::runtime_error("encoder");
    std::string prompt = "<|im_start|>system\n<|im_end|>\n<|im_start|>user\n<|audio_start|>";
    for (int i = 0; i < frames; ++i) prompt += "<|audio_pad|>";
    prompt += "<|audio_end|><|im_end|>\n<|im_start|>assistant\nlanguage ";
    prompt += data[0] == 1 ? "English<asr_text>" : "Japanese<asr_text>";
    int count = 0;
    Int ids(qwen3_asr_tokenize(context, prompt.c_str(), &count), std::free);
    std::fill(prompt.begin(), prompt.end(), '\0');
    if (!ids || count <= 0 || count > 1792) throw std::runtime_error("context");
    Float embedded(qwen3_asr_embed_tokens(context, ids.get(), count), std::free);
    if (!embedded) throw std::runtime_error("embed");
    const int pad = token("<|audio_pad|>");
    int spliced = 0;
    for (int i = 0; i < count; ++i) {
        if (ids.get()[i] == pad) {
            if (spliced >= frames) throw std::runtime_error("splice");
            std::memcpy(embedded.get() + size_t(i) * dimension, encoded.get() + size_t(spliced) * dimension, size_t(dimension) * sizeof(float));
            ++spliced;
        }
    }
    if (spliced != frames || !qwen3_asr_kv_init(context, 2048)) throw std::runtime_error("context");
    qwen3_asr_kv_reset(context);
    check();
    int outputs = 0, vocabulary = 0;
    synesthesia::stage = 3;
    Float logits(qwen3_asr_run_llm_kv(context, embedded.get(), count, 0, &outputs, &vocabulary), std::free);
    const int eos = token("<|im_end|>");
    std::string text;
    int past = count;
    bool ended = false;
    for (int step = 0; step < 256; ++step) {
        check();
        synesthesia::stage = 4;
        if (!logits || outputs <= 0 || vocabulary <= 0 || vocabulary > 200000) throw std::runtime_error("decode");
        const auto last = logits.get() + size_t(outputs - 1) * vocabulary;
        const int32_t next = int32_t(std::max_element(last, last + vocabulary) - last);
        if (next == eos) { ended = true; break; }
        const auto piece = qwen3_asr_token_text(context, next);
        if (!piece) throw std::runtime_error("token");
        if (std::strncmp(piece, "<|", 2) != 0) text += core_bpe::token_bytes_to_utf8(piece);
        if (text.size() > 8192) throw std::runtime_error("text");
        if (step == 255) break;
        Float tail(qwen3_asr_embed_tokens(context, &next, 1), std::free);
        if (!tail) throw std::runtime_error("embed");
        logits.reset(qwen3_asr_run_llm_kv(context, tail.get(), 1, past++, &outputs, &vocabulary));
    }
    if (!ended) throw std::runtime_error("output_limit");
    return text;
}
int main(int argc, char** argv) {
    try {
        auto selection = synesthesia::select_device(argc, argv);
        if (std::strcmp(argv[1], "--probe") == 0) { synesthesia::ready(selection); return 0; }
        auto parameters = qwen3_asr_context_default_params();
        parameters.n_threads = 8;
        parameters.verbosity = 0;
        parameters.use_gpu = selection.vulkan;
        context = qwen3_asr_init_from_file(argv[1], parameters);
        if (!context || !qwen3_asr_synesthesia_prepare(context) || !qwen3_asr_kv_init(context, 2048)) throw std::runtime_error("load");
        synesthesia::ready(selection);
        int result = synesthesia::run(transcribe, clear);
        qwen3_asr_free(context);
        return result;
    } catch (...) {
        if (context) qwen3_asr_free(context);
        try { synesthesia::reply(13, 0); } catch (...) {}
        return 2;
    }
}
