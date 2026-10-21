use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub url: String,
    pub sha256: String,
    pub bytes: u64,
    pub file_name: String,
    pub archive_directory: Option<String>,
    pub license: String,
    pub source: String,
    pub languages: String,
}

#[derive(Serialize)]
pub struct ModelEntry {
    #[serde(flatten)]
    pub model: Model,
    pub installed_path: Option<String>,
}

pub fn catalog() -> Result<Vec<Model>, String> {
    serde_json::from_str(include_str!("../model-catalog.json"))
        .map_err(|_| "Bundled model catalog is invalid.".into())
}

pub fn model_path(root: &Path, model: &Model) -> PathBuf {
    let base = root.join(&model.id);
    model
        .archive_directory
        .as_ref()
        .map_or_else(|| base.join(&model.file_name), |name| base.join(name))
}

pub fn list(root: &Path) -> Result<Vec<ModelEntry>, String> {
    Ok(catalog()?
        .into_iter()
        .map(|model| {
            let path = model_path(root, &model);
            let installed_path = root
                .join(&model.id)
                .join("verified.json")
                .is_file()
                .then(|| path.to_string_lossy().into_owned())
                .filter(|_| path.exists());
            ModelEntry {
                model,
                installed_path,
            }
        })
        .collect())
}

pub fn validate(kind: &str, path: &str) -> Result<(), String> {
    match kind {
        "speech" => crate::speech::LocalWhisper::load(
            path, dictation_core::settings::SpeechDevice::Cpu,
        ).map(|_| ()),
        "wake" => crate::wake::LocalWake::load(path, "hello world").map(|_| ()),
        _ => Err("Unknown model kind.".into()),
    }
}
