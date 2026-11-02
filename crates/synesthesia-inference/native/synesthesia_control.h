#pragma once
#include "ggml-backend.h"
#include <atomic>
#include <cstdint>
#include <string>
#include <vector>

namespace synesthesia {
constexpr uint32_t magic = 0x31494e53;
constexpr uint32_t max_payload = 320001;
extern std::atomic<bool> cancelled;
extern std::atomic<uint64_t> graphs;
extern std::atomic<uint32_t> stage;
extern std::atomic<bool> in_graph;
struct Selection { bool vulkan = false; uint32_t vendor = 0; uint32_t device = 0; ggml_backend_dev_t native = nullptr; };
Selection select_device(int argc, char** argv);
void reply(uint32_t kind, uint64_t job, const std::vector<uint8_t>& payload = {});
void ready(const Selection& selection);
using Work = std::string (*)(const std::vector<uint8_t>&);
using Clear = void (*)();
int run(Work work, Clear clear);
}
ggml_backend_t synesthesia_selected_backend_init();
ggml_status synesthesia_graph_compute(ggml_backend_sched_t scheduler, ggml_cgraph* graph);
inline const char* synesthesia_content_dump_disabled() { return nullptr; }
