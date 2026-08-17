use std::{
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{contracts::PreferencesDto, ports::PortError};

const MAX_RECENTS: usize = 12;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PreferencesFile {
    recent_repositories: Vec<String>,
    vela_binary_path: Option<String>,
}

pub(crate) struct PreferencesStore {
    path: PathBuf,
    value: PreferencesFile,
}

impl PreferencesStore {
    pub(crate) fn load_default() -> Result<Self, PortError> {
        let dirs = ProjectDirs::from("science", "Vela", "Vela Workbench")
            .ok_or_else(|| PortError::Unavailable("resolve application data directory".into()))?;
        Self::load(dirs.data_local_dir().join("preferences.json"))
    }

    pub(crate) fn load(path: PathBuf) -> Result<Self, PortError> {
        let value = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| {
                PortError::Parse(format!("preferences file is invalid: {error}"))
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                PreferencesFile::default()
            }
            Err(error) => {
                return Err(PortError::Process(format!("read preferences: {error}")));
            }
        };
        Ok(Self { path, value })
    }

    pub(crate) fn dto(&self) -> PreferencesDto {
        PreferencesDto {
            recent_repositories: self.value.recent_repositories.clone(),
            vela_binary_path: self.value.vela_binary_path.clone(),
        }
    }

    pub(crate) fn contains_repository(&self, path: &Path) -> bool {
        let expected = path.display().to_string();
        self.value
            .recent_repositories
            .iter()
            .any(|item| item == &expected)
    }

    pub(crate) fn remember_repository(&mut self, path: &Path) -> Result<(), PortError> {
        let value = path.display().to_string();
        self.value.recent_repositories.retain(|item| item != &value);
        self.value.recent_repositories.insert(0, value);
        self.value.recent_repositories.truncate(MAX_RECENTS);
        self.save()
    }

    pub(crate) fn set_vela_binary(&mut self, path: &Path) -> Result<(), PortError> {
        self.value.vela_binary_path = Some(path.display().to_string());
        self.save()
    }

    pub(crate) fn clear_recents(&mut self) -> Result<PreferencesDto, PortError> {
        self.value.recent_repositories.clear();
        self.value.vela_binary_path = None;
        self.save()?;
        Ok(self.dto())
    }

    pub(crate) fn vela_binary_path(&self) -> Option<PathBuf> {
        self.value.vela_binary_path.as_deref().map(PathBuf::from)
    }

    fn save(&self) -> Result<(), PortError> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| PortError::Process("preferences path has no parent".into()))?;
        fs::create_dir_all(parent).map_err(|error| {
            PortError::Process(format!("create preferences directory: {error}"))
        })?;
        let bytes = serde_json::to_vec_pretty(&self.value)
            .map_err(|error| PortError::Process(format!("encode preferences: {error}")))?;
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, bytes).map_err(|error| {
            PortError::Process(format!("write preferences temporary file: {error}"))
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600)).map_err(
                |error| PortError::Process(format!("protect preferences temporary file: {error}")),
            )?;
        }
        fs::rename(&temporary, &self.path)
            .map_err(|error| PortError::Process(format!("install preferences file: {error}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PreferencesStore;

    #[test]
    fn preferences_hold_only_clearable_paths_and_tool_choice() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("preferences.json");
        let mut store = PreferencesStore::load(path.clone()).expect("store loads");
        store
            .remember_repository(temp.path())
            .expect("recent saves");
        store
            .set_vela_binary(&temp.path().join("vela"))
            .expect("tool choice saves");
        let encoded = std::fs::read_to_string(path).expect("preferences readable");
        assert!(encoded.contains("recent_repositories"));
        assert!(encoded.contains("vela_binary_path"));
        assert!(!encoded.contains("claim"));
        assert!(!encoded.contains("transcript"));
        store.clear_recents().expect("recents clear");
        assert!(store.dto().recent_repositories.is_empty());
        assert!(store.dto().vela_binary_path.is_none());
    }

    #[test]
    fn deleting_preferences_does_not_touch_selected_source() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repository = temp.path().join("repository");
        std::fs::create_dir(&repository).expect("repository");
        std::fs::write(repository.join("evidence.txt"), "private evidence\n").expect("evidence");
        let preferences = temp.path().join("app-data/preferences.json");
        let mut store = PreferencesStore::load(preferences.clone()).expect("store");
        store.remember_repository(&repository).expect("remember");
        drop(store);
        std::fs::remove_file(preferences).expect("delete preferences");
        assert_eq!(
            std::fs::read_to_string(repository.join("evidence.txt")).expect("source survives"),
            "private evidence\n"
        );
    }
}
