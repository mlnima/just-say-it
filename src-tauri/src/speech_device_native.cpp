struct ggml_backend_device;
struct ggml_backend;

extern "C" ggml_backend * ggml_backend_dev_init(ggml_backend_device *, const char *);
extern "C" void ggml_backend_free(ggml_backend *);

// A driver can throw a C++ exception while GGML creates a Vulkan device.
// Contain it here: exceptions must never unwind through the Rust FFI caller.
extern "C" bool dictation_probe_gpu(ggml_backend_device * device) noexcept {
    try {
        auto * backend = ggml_backend_dev_init(device, nullptr);
        if (!backend) {
            return false;
        }
        ggml_backend_free(backend);
        return true;
    } catch (...) {
        return false;
    }
}
