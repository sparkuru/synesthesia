#include "synesthesia_control.h"
#include "llama.h"
#include "chat.h"
#include <algorithm>
#include <cstring>
#include <stdexcept>

static llama_model* model = nullptr;
static llama_context* context = nullptr;
static common_chat_templates_ptr templates;
static llama_context_params context_parameters;
extern "C" bool synesthesia_llama_device(const llama_model*, ggml_backend_dev_t);
static bool abort_decode(void*) { return synesthesia::cancelled; }
static void clear() {
    if (context) {
        llama_synchronize(context);
        llama_memory_clear(llama_get_memory(context), true);
        llama_free(context);
        context = nullptr;
    }
    if (model) {
        context = llama_init_from_model(model, context_parameters);
        if (!context) throw std::runtime_error("context");
    }
}
static void check() { if (synesthesia::cancelled) throw std::runtime_error("cancel"); }
static std::string translate(const std::vector<uint8_t>& data) {
    check();
    if (data.empty() || data.size() > 8192) throw std::runtime_error("input");
    common_chat_msg message;
    message.role = "user";
    message.content = "Translate the following text into Simplified Chinese, without adding any explanation:\n\n";
    message.content.append(reinterpret_cast<const char*>(data.data()), data.size());
    auto prompt = common_chat_format_single(templates.get(), {}, message, true, true);
    std::fill(message.content.begin(), message.content.end(), '\0');
    auto vocabulary = llama_model_get_vocab(model);
    std::vector<llama_token> tokens(1536);
    int count = llama_tokenize(vocabulary, prompt.data(), int(prompt.size()), tokens.data(), int(tokens.size()), true, true);
    std::fill(prompt.begin(), prompt.end(), '\0');
    if (count <= 0 || count > int(tokens.size())) throw std::runtime_error("context");
    tokens.resize(size_t(count));
    llama_memory_clear(llama_get_memory(context), true);
    for (int offset = 0; offset < count; offset += 256) {
        check();
        int n = std::min(256, count - offset);
        synesthesia::graphs.fetch_add(1);
        if (llama_decode(context, llama_batch_get_one(tokens.data() + offset, n)) != 0) throw std::runtime_error("prefill");
    }
    std::fill(tokens.begin(), tokens.end(), 0);
    std::string text;
    bool ended = false;
    for (int step = 0; step < 512; ++step) {
        check();
        auto logits = llama_get_logits_ith(context, -1);
        int size = llama_vocab_n_tokens(vocabulary);
        if (!logits || size <= 0 || size > 200000) throw std::runtime_error("logits");
        llama_token next = llama_token(std::max_element(logits, logits + size) - logits);
        if (llama_vocab_is_eog(vocabulary, next)) { ended = true; break; }
        char piece[1024];
        int n = llama_token_to_piece(vocabulary, next, piece, sizeof(piece), 0, false);
        if (n < 0 || n > int(sizeof(piece))) throw std::runtime_error("token");
        text.append(piece, size_t(n));
        std::fill(std::begin(piece), std::end(piece), '\0');
        if (text.size() > 8192) throw std::runtime_error("text");
        if (step == 511) break;
        check();
        synesthesia::graphs.fetch_add(1);
        if (llama_decode(context, llama_batch_get_one(&next, 1)) != 0) throw std::runtime_error("decode");
    }
    if (!ended) throw std::runtime_error("output_limit");
    return text;
}
int main(int argc, char** argv) {
    try {
        auto selection = synesthesia::select_device(argc, argv);
        if (std::strcmp(argv[1], "--probe") == 0) { synesthesia::ready(selection); return 0; }
        llama_log_set([](ggml_log_level, const char*, void*) {}, nullptr);
        llama_backend_init();
        ggml_backend_dev_t devices[] = {selection.native, nullptr};
        auto parameters = llama_model_default_params();
        parameters.devices = devices;
        parameters.n_gpu_layers = selection.vulkan ? -1 : 0;
        parameters.split_mode = LLAMA_SPLIT_MODE_NONE;
        model = llama_model_load_from_file(argv[1], parameters);
        if (!model || !synesthesia_llama_device(model, selection.native)) throw std::runtime_error("load");
        auto cp = llama_context_default_params();
        cp.n_ctx = 2048; cp.n_batch = 256; cp.n_ubatch = 256; cp.n_seq_max = 1;
        cp.n_threads = 8; cp.n_threads_batch = 8;
        cp.abort_callback = abort_decode; cp.no_perf = true;
        cp.offload_kqv = selection.vulkan; cp.op_offload = selection.vulkan;
        context_parameters = cp;
        context = llama_init_from_model(model, cp);
        if (!context) throw std::runtime_error("context");
        templates = common_chat_templates_init(model, "");
        if (!templates || !llama_model_chat_template(model, nullptr)) throw std::runtime_error("template");
        synesthesia::ready(selection);
        int result = synesthesia::run(translate, clear);
        templates.reset();
        llama_free(context); llama_model_free(model); llama_backend_free();
        return result;
    } catch (...) {
        templates.reset();
        if (context) llama_free(context);
        if (model) llama_model_free(model);
        llama_backend_free();
        try { synesthesia::reply(13, 0); } catch (...) {}
        return 2;
    }
}
