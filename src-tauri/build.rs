fn main() {
    cc::Build::new()
        .cpp(true)
        .file("src/speech_device_native.cpp")
        .flag_if_supported("/EHsc")
        .compile("speech_device_probe");
    println!("cargo:rerun-if-changed=src/speech_device_native.cpp");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        stage_vulkan_runtime();
    }
    tauri_build::build()
}

fn stage_vulkan_runtime() {
    use std::path::PathBuf;
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo project path is missing."));
    let runtime = root.join("../artifacts/build-tools/vulkan-runtime");
    let architecture = std::env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo target architecture is missing.");
    let directory = match architecture.as_str() {
        "x86_64" => "x64",
        "x86" => "x86",
        _ => panic!("The pinned Windows Vulkan runtime supports x64 and x86 targets only."),
    };
    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output path is missing."));
    let profile = output.ancestors().nth(3).expect("Cargo profile directory is missing.");
    for relative in [format!("{directory}/vulkan-1.dll"), "VulkanRT-License.txt".into()] {
        let source = runtime.join(relative);
        println!("cargo:rerun-if-changed={}", source.display());
        std::fs::copy(&source, profile.join(source.file_name().expect("Runtime filename is missing.")))
            .expect("Prepare the Vulkan runtime with scripts/cargo.mjs, scripts/desktop.mjs or scripts/package.mjs before building.");
    }
}
