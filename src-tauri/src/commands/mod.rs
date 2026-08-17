use std::{
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use rfd::FileDialog;
use tauri::State;

use crate::{
    contracts::{
        BootstrapDto, CommandErrorDto, GitSnapshotDto, LaunchKindDto, LaunchResultDto,
        PreferencesDto, RepositorySnapshotDto, RuntimePolicyDto, VelaBinaryDto, VelaInspectionDto,
    },
    ports::{self, PortError},
    preferences::PreferencesStore,
};

pub(crate) struct AppState {
    preferences: Mutex<PreferencesStore>,
}

impl AppState {
    pub(crate) fn load() -> Result<Self, PortError> {
        Ok(Self {
            preferences: Mutex::new(PreferencesStore::load_default()?),
        })
    }
}

impl From<PortError> for CommandErrorDto {
    fn from(error: PortError) -> Self {
        Self::new(error.kind(), error.to_string())
    }
}

fn state_error() -> CommandErrorDto {
    CommandErrorDto::new("internal", "Workbench preference state is unavailable")
}

fn runtime_policy() -> RuntimePolicyDto {
    RuntimePolicyDto {
        interface_commit: ports::vela::INTERFACE_COMMIT.into(),
        interface_tree: ports::vela::INTERFACE_TREE.into(),
        runtime_version: ports::vela::RUNTIME_VERSION.into(),
        runtime_commit: ports::vela::RUNTIME_COMMIT.into(),
        runtime_sha256: ports::vela::PLATFORM_RUNTIME_SHA256.into(),
        read_only: true,
    }
}

#[tauri::command]
pub(crate) fn bootstrap(state: State<'_, AppState>) -> Result<BootstrapDto, CommandErrorDto> {
    let preferences = state.preferences.lock().map_err(|_| state_error())?.dto();
    Ok(BootstrapDto {
        preferences,
        runtime: runtime_policy(),
    })
}

fn inspect_path(
    path: &Path,
    vela_binary_path: Option<&Path>,
) -> Result<RepositorySnapshotDto, PortError> {
    let git = ports::git::inspect(path)?;
    let canonical = PathBuf::from(&git.root);
    let (classification, classification_basis, vela) =
        ports::vela::inspect_repository(&canonical, vela_binary_path)?;
    cross_check_source(&git, &vela)?;
    let confirmed_git = ports::git::inspect(&canonical)?;
    if confirmed_git != git {
        return Err(PortError::Process(
            "repository source changed during Workbench inspection; refresh to obtain one exact observation"
                .into(),
        ));
    }
    let entire = ports::entire::availability(&git);
    let name = canonical
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("repository")
        .to_string();
    let observed_at_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX);
    Ok(RepositorySnapshotDto {
        path: canonical.display().to_string(),
        name,
        observed_at_unix_ms,
        classification,
        classification_basis,
        git,
        vela,
        entire,
        reviewed_problem_url: None,
    })
}

fn cross_check_source(git: &GitSnapshotDto, vela: &VelaInspectionDto) -> Result<(), PortError> {
    if let Some(status) = &vela.status
        && (status.repository_commit.as_deref() != Some(git.head_commit.as_str())
            || status.repository_tree.as_deref() != Some(git.head_tree.as_str()))
    {
        return Err(PortError::Parse(
            "Vela repository_head commit/tree disagrees with the selected Git source".into(),
        ));
    }
    // Native integration `revision` is manifest-declared source context and is
    // not defined as the current Git HEAD. The second Git snapshot below binds
    // the validated manifest observation to a stable local source context.
    Ok(())
}

#[tauri::command]
pub(crate) async fn select_repository(
    state: State<'_, AppState>,
) -> Result<Option<RepositorySnapshotDto>, CommandErrorDto> {
    let selected = tauri::async_runtime::spawn_blocking(|| {
        FileDialog::new()
            .set_title("Choose an existing Git repository")
            .pick_folder()
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("dialog", format!("repository dialog failed: {error}"))
    })?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let binary_path = state
        .preferences
        .lock()
        .map_err(|_| state_error())?
        .vela_binary_path();
    let selected_for_read = selected.clone();
    let snapshot = tauri::async_runtime::spawn_blocking(move || {
        inspect_path(&selected_for_read, binary_path.as_deref())
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("repository task failed: {error}"))
    })??;
    state
        .preferences
        .lock()
        .map_err(|_| state_error())?
        .remember_repository(Path::new(&snapshot.path))?;
    Ok(Some(snapshot))
}

#[tauri::command]
pub(crate) async fn inspect_repository(
    path: String,
    state: State<'_, AppState>,
) -> Result<RepositorySnapshotDto, CommandErrorDto> {
    let canonical = std::fs::canonicalize(&path).map_err(|error| {
        CommandErrorDto::new("invalid_input", format!("resolve repository: {error}"))
    })?;
    let binary_path = {
        let preferences = state.preferences.lock().map_err(|_| state_error())?;
        if !preferences.contains_repository(&canonical) {
            return Err(CommandErrorDto::new(
                "not_selected",
                "repository path is not an explicit user-selected recent",
            ));
        }
        preferences.vela_binary_path()
    };
    tauri::async_runtime::spawn_blocking(move || inspect_path(&canonical, binary_path.as_deref()))
        .await
        .map_err(|error| {
            CommandErrorDto::new("internal", format!("repository task failed: {error}"))
        })?
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn select_vela_binary(
    state: State<'_, AppState>,
) -> Result<Option<VelaBinaryDto>, CommandErrorDto> {
    let selected = tauri::async_runtime::spawn_blocking(|| {
        FileDialog::new()
            .set_title("Choose the installed signed Vela executable")
            .pick_file()
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("Vela dialog failed: {error}")))?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let selected_for_read = selected.clone();
    let identity = tauri::async_runtime::spawn_blocking(move || {
        ports::vela::inspect_binary(&selected_for_read)
    })
    .await
    .map_err(|error| CommandErrorDto::new("internal", format!("Vela task failed: {error}")))??;
    if identity.state != crate::contracts::VelaBinaryStateDto::SignedRuntimeBaseline {
        return Err(CommandErrorDto::new(
            "unsupported",
            "selected file was not executed because its hash is not the pinned signed Vela runtime",
        )
        .with_detail(identity.sha256));
    }
    state
        .preferences
        .lock()
        .map_err(|_| state_error())?
        .set_vela_binary(Path::new(&identity.path))?;
    Ok(Some(identity))
}

#[tauri::command]
pub(crate) fn clear_recents(state: State<'_, AppState>) -> Result<PreferencesDto, CommandErrorDto> {
    state
        .preferences
        .lock()
        .map_err(|_| state_error())?
        .clear_recents()
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn launch_repository(
    path: String,
    kind: LaunchKindDto,
    state: State<'_, AppState>,
) -> Result<LaunchResultDto, CommandErrorDto> {
    let canonical = std::fs::canonicalize(&path).map_err(|error| {
        CommandErrorDto::new("invalid_input", format!("resolve repository: {error}"))
    })?;
    if !state
        .preferences
        .lock()
        .map_err(|_| state_error())?
        .contains_repository(&canonical)
    {
        return Err(CommandErrorDto::new(
            "not_selected",
            "repository path is not an explicit user-selected recent",
        ));
    }
    tauri::async_runtime::spawn_blocking(move || ports::launch::launch(&canonical, kind))
        .await
        .map_err(|error| CommandErrorDto::new("internal", format!("launch task failed: {error}")))?
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::Path};

    use sha2::{Digest, Sha256};

    use super::inspect_path;

    fn file_manifest(path: &Path, root: &Path, out: &mut BTreeMap<String, String>) {
        let mut entries: Vec<_> = fs::read_dir(path)
            .expect("read directory")
            .map(|entry| entry.expect("directory entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            let metadata = fs::symlink_metadata(&entry).expect("metadata");
            let relative = entry
                .strip_prefix(root)
                .expect("relative")
                .display()
                .to_string();
            if metadata.file_type().is_symlink() {
                out.insert(
                    relative,
                    format!("symlink:{}", fs::read_link(&entry).expect("link").display()),
                );
            } else if metadata.is_dir() {
                file_manifest(&entry, root, out);
            } else if metadata.is_file() {
                let digest = Sha256::digest(fs::read(&entry).expect("read file"));
                out.insert(
                    relative,
                    digest.iter().map(|byte| format!("{byte:02x}")).collect(),
                );
            }
        }
    }

    #[test]
    fn live_full_git_and_vela_inspection_is_byte_preserving_when_requested() {
        let (Ok(repository), Ok(binary)) = (
            std::env::var("VELA_WORKBENCH_SMOKE_REPO"),
            std::env::var("VELA_WORKBENCH_SMOKE_BINARY"),
        ) else {
            return;
        };
        let repository = Path::new(&repository).canonicalize().expect("repository");
        let binary = Path::new(&binary).canonicalize().expect("binary");
        let mut before = BTreeMap::new();
        file_manifest(&repository, &repository, &mut before);
        let snapshot = inspect_path(&repository, Some(&binary)).expect("full exact inspection");
        let mut after = BTreeMap::new();
        file_manifest(&repository, &repository, &mut after);
        assert_eq!(before, after);
        if let Some(status) = snapshot.vela.status {
            assert_eq!(
                snapshot.git.head_commit,
                status.repository_commit.expect("commit")
            );
        } else {
            assert!(
                !snapshot
                    .vela
                    .integration
                    .expect("status or integration")
                    .revision
                    .is_empty()
            );
        }
    }
}
