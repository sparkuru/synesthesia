#include "synesthesia_control.h"
#include "ggml-cpu.h"
#ifdef SYNESTHESIA_VULKAN
#include "ggml-vulkan.h"
extern "C" bool synesthesia_vk_identity(int, uint32_t*, uint32_t*);
#endif
#include <cstdio>
#include <cstring>
#include <mutex>
#include <stdexcept>
#include <thread>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

namespace synesthesia {
std::atomic<bool> cancelled{false};
std::atomic<uint64_t> graphs{0};
std::atomic<uint32_t> stage{0};
std::atomic<bool> in_graph{false};
static std::atomic<uint64_t> active_job{0};
static Selection selected;
static std::mutex output;
static uint64_t number(const char* s) {
    if (!s || !*s) throw std::runtime_error("argument");
    uint64_t n = 0;
    for (; *s; ++s) {
        if (*s < '0' || *s > '9' || n > (UINT32_MAX - uint64_t(*s - '0')) / 10) throw std::runtime_error("argument");
        n = n * 10 + uint64_t(*s - '0');
    }
    return n;
}
Selection select_device(int argc, char** argv) {
#ifdef _WIN32
    _setmode(_fileno(stdin), _O_BINARY);
    _setmode(_fileno(stdout), _O_BINARY);
#endif
    ggml_log_set([](ggml_log_level, const char*, void*) {}, nullptr);
    if (argc < 3) throw std::runtime_error("argument");
    if (std::strcmp(argv[2], "cpu") == 0) {
        auto probe = ggml_backend_cpu_init();
        if (!probe || !ggml_backend_is_cpu(probe)) throw std::runtime_error("backend");
        ggml_backend_free(probe);
        return selected;
    }
    if (argc != 6 || std::strcmp(argv[2], "vulkan") != 0) throw std::runtime_error("argument");
#ifdef SYNESTHESIA_VULKAN
    const uint32_t vendor = uint32_t(number(argv[3]));
    const uint32_t device = uint32_t(number(argv[4]));
    int matches = 0;
    auto reg = ggml_backend_vk_reg();
    for (int i = 0; i < ggml_backend_vk_get_device_count(); ++i) {
        uint32_t v = 0, d = 0;
        char name[256] = {};
        ggml_backend_vk_get_device_description(i, name, sizeof(name));
        if (synesthesia_vk_identity(i, &v, &d) && v == vendor && d == device && std::strcmp(name, argv[5]) == 0) {
            auto dev = ggml_backend_reg_dev_get(reg, size_t(i));
            if (ggml_backend_dev_type(dev) != GGML_BACKEND_DEVICE_TYPE_GPU) throw std::runtime_error("device");
            selected = {true, vendor, device, dev};
            ++matches;
        }
    }
    if (matches != 1) throw std::runtime_error("device");
    auto probe = synesthesia_selected_backend_init();
    if (!probe || !ggml_backend_is_vk(probe) || ggml_backend_get_device(probe) != selected.native) throw std::runtime_error("device");
    ggml_backend_free(probe);
    return selected;
#else
    (void)number;
    throw std::runtime_error("backend");
#endif
}
static void put(std::vector<uint8_t>& bytes, uint64_t value, unsigned count) {
    for (unsigned i = 0; i < count; ++i) bytes.push_back(uint8_t(value >> (i * 8)));
}
void reply(uint32_t kind, uint64_t job, const std::vector<uint8_t>& payload) {
    if (payload.size() > 8200) throw std::runtime_error("output");
    std::vector<uint8_t> header;
    put(header, magic, 4); put(header, kind, 4); put(header, job, 8); put(header, payload.size(), 4);
    std::lock_guard<std::mutex> lock(output);
    if (std::fwrite(header.data(), 1, header.size(), stdout) != header.size() ||
        std::fwrite(payload.data(), 1, payload.size(), stdout) != payload.size() || std::fflush(stdout) != 0) throw std::runtime_error("pipe");
}
void ready(const Selection& selection) {
    std::vector<uint8_t> payload;
    put(payload, selection.vulkan ? 1 : 0, 4); put(payload, selection.vendor, 4); put(payload, selection.device, 4);
    reply(10, 0, payload);
}
static uint64_t get(const uint8_t* bytes, unsigned count) {
    uint64_t value = 0;
    for (unsigned i = 0; i < count; ++i) value |= uint64_t(bytes[i]) << (i * 8);
    return value;
}
int run(Work work, Clear clear) {
    std::thread worker;
    std::atomic<bool> active{false};
    uint64_t previous_job = 0;
    try {
        for (;;) {
            uint8_t header[20];
            const auto n = std::fread(header, 1, sizeof(header), stdin);
            if (n == 0) break;
            if (n != sizeof(header) || get(header, 4) != magic) throw std::runtime_error("frame");
            auto kind = get(header + 4, 4);
            auto job = get(header + 8, 8);
            auto length = get(header + 16, 4);
            if (length > max_payload) throw std::runtime_error("frame");
            if (job == 0 || job <= previous_job) throw std::runtime_error("state");
            previous_job = job;
            std::vector<uint8_t> data(static_cast<size_t>(length));
            if (length && std::fread(data.data(), 1, data.size(), stdin) != length) throw std::runtime_error("frame");
            if (kind == 1) {
                if (active || data.empty()) throw std::runtime_error("state");
                if (worker.joinable()) worker.join();
                cancelled = false;
                active = true;
                active_job = job;
                worker = std::thread([&, job, data = std::move(data)]() mutable {
                    try {
                        auto text = work(data);
                        clear();
                        std::fill(data.begin(), data.end(), 0);
                        if (!cancelled) {
                            if (text.size() > 8192) throw std::runtime_error("output");
                            std::vector<uint8_t> payload;
                            put(payload, graphs.load(), 8);
                            payload.insert(payload.end(), text.begin(), text.end());
                            active = false;
                            active_job = 0;
                            stage = 0;
                            reply(11, job, payload);
                            std::fill(payload.begin(), payload.end(), 0);
                        }
                        std::fill(text.begin(), text.end(), '\0');
                    } catch (...) {
                        try { clear(); active = false; if (!cancelled) reply(13, job); } catch (...) {}
                    }
                    active = false;
                    active_job = 0;
                    stage = 0;
                });
            } else if (kind == 4) {
                if (length) throw std::runtime_error("frame");
                std::vector<uint8_t> progress;
                put(progress, stage.load(), 4); put(progress, in_graph.load() ? 1 : 0, 4);
                put(progress, graphs.load(), 8); put(progress, active_job.load(), 8);
                reply(14, job, progress);
            } else if (kind == 2 || kind == 3) {
                if (length || job == 0) throw std::runtime_error("frame");
                cancelled = true;
                if (worker.joinable()) worker.join();
                clear();
                std::vector<uint8_t> count;
                put(count, graphs.load(), 8);
                reply(12, job, count);
                if (kind == 3) return 0;
            } else throw std::runtime_error("frame");
        }
    } catch (...) {
        cancelled = true;
        if (worker.joinable()) worker.join();
        try { clear(); reply(13, 0); } catch (...) {}
        return 2;
    }
    cancelled = true;
    if (worker.joinable()) worker.join();
    clear();
    return 0;
}
}
ggml_backend_t synesthesia_selected_backend_init() {
    return synesthesia::selected.native ? ggml_backend_dev_init(synesthesia::selected.native, nullptr) : nullptr;
}
ggml_status synesthesia_graph_compute(ggml_backend_sched_t scheduler, ggml_cgraph* graph) {
    if (synesthesia::cancelled) return GGML_STATUS_ABORTED;
    synesthesia::graphs.fetch_add(1);
    synesthesia::in_graph = true;
    const auto result = ggml_backend_sched_graph_compute(scheduler, graph);
    ggml_backend_sched_synchronize(scheduler);
    synesthesia::in_graph = false;
    return synesthesia::cancelled ? GGML_STATUS_ABORTED : result;
}
