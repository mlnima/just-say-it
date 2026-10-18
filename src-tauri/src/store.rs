use dictation_core::settings::Settings;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub struct Store {
    path: PathBuf,
    pub load_error: Option<String>,
}

impl Store {
    pub fn open(directory: &Path) -> (Self, Settings) {
        let path = directory.join("settings.json");
        let loaded = match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice::<Settings>(&bytes)
                .map_err(|_| "Settings could not be read. The file was preserved.")
                .and_then(|settings| {
                    settings
                        .validate()
                        .map(|_| settings)
                        .map_err(|_| "Settings are invalid. The file was preserved.")
                }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(_) => {
                Err("Settings could not be opened. Check file permissions; the file was preserved.")
            }
        };
        match loaded {
            Ok(settings) => (
                Self {
                    path,
                    load_error: None,
                },
                settings,
            ),
            Err(error) => (
                Self {
                    path,
                    load_error: Some(error.into()),
                },
                Settings::default(),
            ),
        }
    }

    pub fn save(&self, settings: &Settings) -> Result<(), String> {
        if let Some(error) = &self.load_error {
            return Err(error.clone());
        }
        settings.validate()?;
        let directory = self.path.parent().ok_or("Settings path has no parent.")?;
        std::fs::create_dir_all(directory).map_err(|_| "Cannot create the settings directory.")?;
        let mut file = tempfile::NamedTempFile::new_in(directory)
            .map_err(|_| "Cannot create temporary settings file.")?;
        let bytes =
            serde_json::to_vec_pretty(settings).map_err(|_| "Cannot serialize settings.")?;
        file.write_all(&bytes)
            .and_then(|_| file.as_file().sync_all())
            .map_err(|_| "Settings write failed; previous settings remain saved.")?;
        file.persist(&self.path)
            .map_err(|_| "Atomic settings replacement failed; previous settings remain saved.")?;
        Ok(())
    }
}
