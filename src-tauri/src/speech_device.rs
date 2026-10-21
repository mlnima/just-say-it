use serde::Serialize;
use std::ffi::CStr;
use whisper_rs::whisper_rs_sys as native;

extern "C" {
    fn dictation_probe_gpu(device: native::ggml_backend_dev_t) -> bool;
}

#[derive(Clone, Serialize)]
pub struct GpuDevice {
    pub index: i32,
    pub name: String,
}

#[derive(Clone, Default, Serialize)]
pub struct GpuAvailability {
    pub checked: bool,
    pub device: Option<GpuDevice>,
    pub reason: Option<String>,
}

pub fn detect_gpu() -> GpuAvailability {
    let mut index = 0;
    let mut found = false;
    // SAFETY: GGML owns the registered devices for the process lifetime. Each
    // enumeration index is bounded by its registry count; no registry is unloaded.
    unsafe {
        for position in 0..native::ggml_backend_dev_count() {
            let device = native::ggml_backend_dev_get(position);
            if device.is_null() {
                continue;
            }
            let kind = native::ggml_backend_dev_type(device);
            if kind != native::ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_GPU
                && kind != native::ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_IGPU
            {
                continue;
            }
            found = true;
            // The native probe owns/frees its backend and contains driver exceptions.
            // No model, audio, or inference graph is attached to it.
            if dictation_probe_gpu(device) {
                let description = native::ggml_backend_dev_description(device);
                let name = if description.is_null() {
                    native::ggml_backend_dev_name(device)
                } else {
                    description
                };
                if !name.is_null() {
                    return GpuAvailability {
                        checked: true,
                        device: Some(GpuDevice {
                            index,
                            name: CStr::from_ptr(name).to_string_lossy().into_owned(),
                        }),
                        reason: None,
                    };
                }
            }
            index += 1;
        }
    }
    GpuAvailability {
        checked: true,
        device: None,
        reason: Some(if found {
            "A GPU was detected, but its inference backend could not initialize. Check its driver or select CPU."
        } else {
            "Whisper found no compatible GPU through this build's acceleration backend. Check your graphics driver or select CPU."
        }.into()),
    }
}

pub fn require_gpu() -> Result<GpuDevice, String> {
    let available = detect_gpu();
    available.device.ok_or_else(|| {
        available.reason.unwrap_or_else(|| "GPU acceleration is unavailable.".into())
    })
}
