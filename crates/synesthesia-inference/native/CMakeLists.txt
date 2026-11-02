cmake_minimum_required(VERSION 3.22)
project(synesthesia_native LANGUAGES C CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(GGML_BACKEND_DL OFF CACHE BOOL "" FORCE)
set(GGML_NATIVE OFF CACHE BOOL "" FORCE)
set(GGML_CUDA OFF CACHE BOOL "" FORCE)
set(GGML_METAL OFF CACHE BOOL "" FORCE)
set(GGML_RPC OFF CACHE BOOL "" FORCE)
set(GGML_OPENMP OFF CACHE BOOL "" FORCE)
foreach(isa SSE42 AVX AVX2 AVX_VNNI AVX512 AVX512_VBMI AVX512_VNNI AVX512_BF16 AMX_TILE AMX_INT8 AMX_BF16 FMA F16C BMI2)
    set(GGML_${isa} OFF CACHE BOOL "Generic x64 CPU baseline" FORCE)
endforeach()
option(GGML_VULKAN "Build the requested Vulkan route" ON)
set(SYNESTHESIA_ENGINE "" CACHE STRING "asr or translation")
set(SYNESTHESIA_SOURCE "" CACHE PATH "Verified pinned source tree")

function(replace_exact path old new)
    file(READ "${path}" body)
    string(FIND "${body}" "${new}" already)
    if(NOT already EQUAL -1)
        return()
    endif()
    string(FIND "${body}" "${old}" found)
    if(found EQUAL -1)
        message(FATAL_ERROR "Pinned source patch mismatch: ${path}")
    endif()
    string(REPLACE "${old}" "${new}" body "${body}")
    file(WRITE "${path}" "${body}")
endfunction()

if(SYNESTHESIA_ENGINE STREQUAL "asr")
    # Upstream src/CMakeLists uses top-level source paths even as a subproject.
    set(SYNESTHESIA_CRISP_ROOT "${SYNESTHESIA_SOURCE}")
    replace_exact("${SYNESTHESIA_SOURCE}/src/CMakeLists.txt"
        [=[${CMAKE_SOURCE_DIR}/]=] [=[${SYNESTHESIA_CRISP_ROOT}/]=])
    set(ggml_source "${SYNESTHESIA_SOURCE}/ggml")
    set(qwen "${SYNESTHESIA_SOURCE}/src/qwen3_asr.cpp")
    set(audio "${SYNESTHESIA_SOURCE}/crisp_audio/src/audio_tower.cpp")
    replace_exact("${qwen}" "#include \"core/gpu_backend_pref.h\""
        "#include \"synesthesia_control.h\"\n#include \"core/gpu_backend_pref.h\"")
    replace_exact("${qwen}" "params.use_gpu ? crispasr_init_gpu_backend() : core_cpu_backend::init()"
        "params.use_gpu ? synesthesia_selected_backend_init() : core_cpu_backend::init()")
    replace_exact("${qwen}" "    if (!ctx->backend)\n        ctx->backend = core_cpu_backend::init();"
        "    if (!ctx->backend) { delete ctx; return nullptr; }")
    replace_exact("${audio}" "#include \"ggml-backend.h\""
        "#include \"ggml-backend.h\"\n#include \"synesthesia_control.h\"")
    replace_exact("${audio}" "getenv(\"CRISP_AUDIO_DUMP_STAGES\")"
        "synesthesia_content_dump_disabled()")
    replace_exact("${SYNESTHESIA_SOURCE}/src/core/gguf_loader.cpp"
        "!std::getenv(\"CRISPASR_GGUF_NO_FIT\")"
        "false /* synesthesia: refuse automatic model weight spill */")
    replace_exact("${audio}"
        "        ggml_backend_dev_t gdev = ggml_backend_dev_by_type(GGML_BACKEND_DEVICE_TYPE_GPU);\n        if (gdev)\n            ctx->backend = ggml_backend_dev_init(gdev, nullptr);"
        "        ctx->backend = synesthesia_selected_backend_init();\n        if (!ctx->backend) { delete ctx; return nullptr; }")
    foreach(path "${qwen}" "${audio}")
        replace_exact("${path}" "ggml_backend_sched_graph_compute(ctx->sched, gf)"
            "synesthesia_graph_compute(ctx->sched, gf)")
    endforeach()
    replace_exact("${qwen}" "crispasr_imatrix_install(ctx->sched);"
        "/* synesthesia: no activation persistence */")
    replace_exact("${audio}" "crispembed_imatrix_install(ctx->sched);"
        "/* synesthesia: no activation persistence */")
    replace_exact("${qwen}" "    if (ctx)\n        ctx->kv_n_used = 0;"
        "    if (ctx) {\n        ggml_backend_sched_synchronize(ctx->sched);\n        if (ctx->kv_buf) ggml_backend_buffer_clear(ctx->kv_buf, 0);\n        ctx->kv_n_used = 0;\n        ggml_backend_sched_reset(ctx->sched);\n    }")
    file(READ "${qwen}" qwen_body)
    string(FIND "${qwen_body}" "qwen3_asr_synesthesia_prepare" prepare_found)
    if(prepare_found EQUAL -1)
        file(APPEND "${qwen}" "\nextern \"C\" bool qwen3_asr_synesthesia_prepare(qwen3_asr_context* ctx) { return ctx && qwen3_asr_get_audio(ctx); }\n")
    endif()
    string(FIND "${qwen_body}" "qwen3_asr_synesthesia_clear" clear_found)
    if(clear_found EQUAL -1)
        file(APPEND "${qwen}" [=[
extern "C" void crisp_audio_synesthesia_clear(crisp_audio_context*);
extern "C" void qwen3_asr_synesthesia_clear(qwen3_asr_context* ctx) {
    if (!ctx) return;
    qwen3_asr_kv_reset(ctx);
    ggml_backend_sched_free(ctx->sched);
    std::fill(ctx->compute_meta.begin(), ctx->compute_meta.end(), 0);
    std::fill(ctx->cached_enc_meta.begin(), ctx->cached_enc_meta.end(), 0);
    ctx->cached_enc_gf = nullptr;
    ctx->cached_enc_meta.clear();
    ctx->cached_enc_T_chunk = ctx->cached_enc_num_chunks = ctx->cached_enc_T_chunk_out = 0;
    ggml_backend_t backends[2] = {ctx->backend, ctx->backend_cpu};
    const int count = ctx->backend_cpu && ctx->backend_cpu != ctx->backend ? 2 : 1;
    ctx->sched = ggml_backend_sched_new(backends, nullptr, count, 16384, false, false);
    if (!ctx->sched) throw std::bad_alloc();
    if (ctx->audio_ca) crisp_audio_synesthesia_clear(ctx->audio_ca);
}
]=])
    endif()
    file(READ "${audio}" audio_body)
    string(FIND "${audio_body}" "crisp_audio_synesthesia_clear" audio_clear_found)
    if(audio_clear_found EQUAL -1)
        file(APPEND "${audio}" [=[
extern "C" void crisp_audio_synesthesia_clear(crisp_audio_context* ctx) {
    if (!ctx) return;
    ggml_backend_sched_synchronize(ctx->sched);
    ggml_backend_sched_free(ctx->sched);
    std::fill(ctx->compute_meta.begin(), ctx->compute_meta.end(), 0);
    ggml_backend_t backends[2] = {ctx->backend, ctx->backend_cpu};
    ctx->sched = ggml_backend_sched_new(backends, nullptr, ctx->backend_cpu ? 2 : 1, 16384, false, false);
    if (!ctx->sched) throw std::bad_alloc();
}
]=])
    endif()
    set(CRISPASR_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
    set(CRISPASR_BUILD_TESTS OFF CACHE BOOL "" FORCE)
    set(CRISPASR_BUILD_SERVER OFF CACHE BOOL "" FORCE)
    set(CRISPASR_CURL OFF CACHE BOOL "" FORCE)
    set(CRISPASR_PORTABLE_CPU ON CACHE BOOL "" FORCE)
    set(CRISPASR_NO_C2PA_NATIVE ON CACHE BOOL "" FORCE)
    include_directories("${CMAKE_CURRENT_SOURCE_DIR}")
    add_subdirectory("${SYNESTHESIA_SOURCE}" upstream EXCLUDE_FROM_ALL)
    add_executable(synesthesia-asr-helper asr.cpp control.cpp)
    target_include_directories(synesthesia-asr-helper PRIVATE "${SYNESTHESIA_SOURCE}/src")
    target_link_libraries(synesthesia-asr-helper PRIVATE qwen3_asr ggml-cpu)
    set(helper synesthesia-asr-helper)
elseif(SYNESTHESIA_ENGINE STREQUAL "translation")
    set(ggml_source "${SYNESTHESIA_SOURCE}/ggml")
    set(LLAMA_BUILD_TESTS OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_TOOLS OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_SERVER OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_COMMON ON CACHE BOOL "" FORCE)
    set(LLAMA_CURL OFF CACHE BOOL "" FORCE)
    set(model_cpp "${SYNESTHESIA_SOURCE}/src/llama-model.cpp")
    file(READ "${model_cpp}" model_body)
    string(FIND "${model_body}" "synesthesia_llama_device" model_identity_found)
    if(model_identity_found EQUAL -1)
        file(APPEND "${model_cpp}" [=[
extern "C" bool synesthesia_llama_device(const llama_model* model, ggml_backend_dev_t requested) {
    size_t matrices = 0;
    for (const auto& entry : model->tensors_by_name) {
        auto* tensor = entry.second;
        if (entry.first.rfind("blk.", 0) != 0 || tensor->ne[1] <= 1) continue;
        if (!tensor->buffer) return false;
        auto* actual = ggml_backend_buft_get_device(ggml_backend_buffer_get_type(tensor->buffer));
        if (requested ? actual != requested : (!actual || ggml_backend_dev_type(actual) != GGML_BACKEND_DEVICE_TYPE_CPU)) return false;
        ++matrices;
    }
    return matrices != 0;
}
]=])
    endif()
    add_subdirectory("${SYNESTHESIA_SOURCE}" upstream EXCLUDE_FROM_ALL)
    add_executable(synesthesia-translation-helper translation.cpp control.cpp)
    target_include_directories(synesthesia-translation-helper PRIVATE "${SYNESTHESIA_SOURCE}/common")
    target_link_libraries(synesthesia-translation-helper PRIVATE llama llama-common ggml-cpu)
    set(helper synesthesia-translation-helper)
else()
    message(FATAL_ERROR "Choose asr or translation")
endif()

if(GGML_VULKAN)
    set(vk_cpp "${ggml_source}/src/ggml-vulkan/ggml-vulkan.cpp")
    file(READ "${vk_cpp}" vk_body)
    string(FIND "${vk_body}" "synesthesia_vk_identity" identity_found)
    if(identity_found EQUAL -1)
        file(APPEND "${vk_cpp}" "\nextern \"C\" bool synesthesia_vk_identity(int i, uint32_t* vendor, uint32_t* device) {\n if (i < 0 || i >= (int)vk_instance.device_indices.size()) return false;\n auto p = vk_instance.instance.enumeratePhysicalDevices()[vk_instance.device_indices[i]].getProperties();\n *vendor = p.vendorID; *device = p.deviceID; return true;\n}\n")
    endif()
    target_compile_definitions(${helper} PRIVATE SYNESTHESIA_VULKAN)
    target_link_libraries(${helper} PRIVATE ggml-vulkan)
endif()
if(MSVC)
    target_compile_options(${helper} PRIVATE /W4 /utf-8)
else()
    target_compile_options(${helper} PRIVATE -Wall -Wextra -Wpedantic)
endif()
