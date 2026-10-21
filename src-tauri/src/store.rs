use dictation_core::{defaults, settings::Settings};
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
        let mut store = Self {
            path: directory.join("settings.json"),
            load_error: None,
        };
        let loaded = match std::fs::read(&store.path) {
            Ok(bytes) => decode(&bytes).and_then(|(settings, migrated)| {
                if migrated {
                    store.save(&settings).map_err(|_| {
                        "Settings migration could not be saved. The file was preserved."
                    })?;
                }
                Ok(settings)
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(_) => {
                Err("Settings could not be opened. Check file permissions; the file was preserved.")
            }
        };
        match loaded {
            Ok(settings) => (store, settings),
            Err(error) => {
                store.load_error = Some(error.into());
                (store, Settings::default())
            }
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

fn decode(bytes: &[u8]) -> Result<(Settings, bool), &'static str> {
    let unreadable = "Settings could not be read. The file was preserved.";
    let mut value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| unreadable)?;
    let version = value["schema_version"].as_u64();
    let migrated = matches!(version, Some(1 | 2 | 3));
    if version == Some(1) {
        value
            .as_object_mut()
            .ok_or(unreadable)?
            .remove("cancel_shortcut");
    }
    if migrated {
        if matches!(version, Some(1 | 2))
            && value.get("model_retention").is_none_or(serde_json::Value::is_null)
        {
            value["model_retention"] = serde_json::to_value(defaults::MODEL_RETENTION)
                .map_err(|_| unreadable)?;
        }
        value["schema_version"] = defaults::SCHEMA_VERSION.into();
    }
    let settings: Settings = serde_json::from_value(value).map_err(|_| unreadable)?;
    settings
        .validate()
        .map_err(|_| "Settings are invalid. The file was preserved.")?;
    Ok((settings, migrated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_shortcuts_once_without_changing_other_preferences() {
        for shortcut in [None, Some("Control+Shift+F9".into())] {
            let folder = tempfile::tempdir().unwrap();
            let settings = Settings {
                toggle_shortcut: shortcut,
                wake_phrase: Some("hello notebook".into()),
                launch_at_login: Some(false),
                paused: true,
                ..Settings::default()
            };
            let mut saved = serde_json::to_value(&settings).unwrap();
            saved["schema_version"] = 1.into();
            saved["cancel_shortcut"] = "Control+Shift+Backspace".into();
            let path = folder.path().join("settings.json");
            std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();

            let (store, restored) = Store::open(folder.path());
            assert!(store.load_error.is_none());
            assert_eq!(restored, settings);
            let migrated = std::fs::read(&path).unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&migrated).unwrap(),
                serde_json::to_value(&settings).unwrap()
            );
            assert_eq!(Store::open(folder.path()).1, settings);
            assert_eq!(std::fs::read(&path).unwrap(), migrated);
        }
    }

    #[test]
    fn migration_preserves_invalid_or_unknown_settings_without_overwriting() {
        for (version, extra) in [(1, "unknown_field"), (99, "cancel_shortcut")] {
            let folder = tempfile::tempdir().unwrap();
            let mut saved = serde_json::to_value(Settings::default()).unwrap();
            saved["schema_version"] = version.into();
            saved[extra] = "unrecognized".into();
            let bytes = serde_json::to_vec(&saved).unwrap();
            let path = folder.path().join("settings.json");
            std::fs::write(&path, &bytes).unwrap();

            let (store, _) = Store::open(folder.path());
            assert!(store.load_error.is_some());
            assert!(store.save(&Settings::default()).is_err());
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
}
