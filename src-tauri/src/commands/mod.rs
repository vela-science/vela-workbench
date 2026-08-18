use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use tauri::State;

use crate::{
    contracts::{
        BootstrapDto, CancelResultDto, CommandErrorDto, DecisionExecutionDto, DecisionInboxDto,
        DecisionPreviewDto, DecisionRequestDto, EvidenceExportPreviewDto, EvidenceExportRequestDto,
        EvidenceExportResultDto, EvidenceItemDto, EvidenceSourceDto, GitSnapshotDto, LaunchKindDto,
        LaunchResultDto, NativeExecPreviewDto, NativeExecProfileDto, NativeExecResultDto,
        NativeToolDto, PreferencesDto, RecoveryPreviewDto, RecoveryResultDto,
        RepositorySnapshotDto, RuntimePolicyDto, SubmissionDraftDto, SubmissionImportPreviewDto,
        SubmissionPreviewDto, SubmissionResultDto, VelaBinaryDto, VelaInspectionDto,
        VerificationDraftDto, VerificationImportPreviewDto, VerificationMethodDto,
        VerificationPreviewDto, VerificationResultDto, WorktreePreviewDto, WorktreeResultDto,
    },
    ports::{self, PortError},
    preferences::PreferencesStore,
};

pub(crate) struct AppState {
    preferences: Mutex<PreferencesStore>,
    privileged: Mutex<PrivilegedState>,
}

#[derive(Default)]
struct PrivilegedState {
    tools: BTreeMap<NativeExecProfileDto, NativeToolDto>,
    active_run: Option<(String, Arc<AtomicBool>)>,
    completed_runs: BTreeMap<String, NativeExecResultDto>,
    evidence: BTreeMap<String, ports::evidence::CapturedEvidence>,
    recovery_operations: BTreeMap<String, String>,
}

impl PrivilegedState {
    fn clear(&mut self) {
        if let Some((_, cancellation)) = &self.active_run {
            cancellation.store(true, Ordering::SeqCst);
        }
        *self = Self::default();
    }

    fn remember_run(&mut self, result: NativeExecResultDto) {
        self.completed_runs.insert(result.run_id.clone(), result);
        while self.completed_runs.len() > 4 {
            if let Some(first) = self.completed_runs.keys().next().cloned() {
                self.completed_runs.remove(&first);
            }
        }
    }

    fn remember_evidence(&mut self, captured: ports::evidence::CapturedEvidence) {
        self.evidence
            .insert(evidence_key(&captured.dto.source), captured);
        while self.evidence.len() > 16 {
            if let Some(first) = self.evidence.keys().next().cloned() {
                self.evidence.remove(&first);
            }
        }
    }

    fn take_run(&mut self, run_id: &str) -> bool {
        if !self
            .active_run
            .as_ref()
            .is_some_and(|(active_id, _)| active_id == run_id)
        {
            return false;
        }
        self.active_run = None;
        true
    }
}

impl AppState {
    pub(crate) fn load() -> Result<Self, PortError> {
        Ok(Self {
            preferences: Mutex::new(PreferencesStore::load_default()?),
            privileged: Mutex::new(PrivilegedState::default()),
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

fn evidence_key(source: &EvidenceSourceDto) -> String {
    match source {
        EvidenceSourceDto::LocalFile { path, .. } => format!("file:{path}"),
        EvidenceSourceDto::CommandOutput { run_id, stream } => {
            format!("run:{run_id}:{stream}")
        }
    }
}

fn validate_run_id(run_id: &str) -> Result<(), CommandErrorDto> {
    if run_id.len() < 8
        || run_id.len() > 80
        || !run_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(CommandErrorDto::new(
            "invalid_input",
            "run id must be 8 to 80 URL-safe identifier characters",
        ));
    }
    Ok(())
}

fn confirmed(title: &str, description: &str) -> bool {
    matches!(
        MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title(title)
            .set_description(description)
            .set_buttons(MessageButtons::OkCancel)
            .show(),
        MessageDialogResult::Ok | MessageDialogResult::Yes
    )
}

fn dialog_value(value: &str) -> Result<String, CommandErrorDto> {
    serde_json::to_string(value)
        .map_err(|error| CommandErrorDto::new("internal", format!("encode dialog value: {error}")))
}

fn selected_repository(
    path: &str,
    state: &State<'_, AppState>,
) -> Result<(PathBuf, Option<PathBuf>), CommandErrorDto> {
    let canonical = std::fs::canonicalize(path).map_err(|error| {
        CommandErrorDto::new("invalid_input", format!("resolve repository: {error}"))
    })?;
    let preferences = state.preferences.lock().map_err(|_| state_error())?;
    if !preferences.contains_repository(&canonical) {
        return Err(CommandErrorDto::new(
            "not_selected",
            "repository path is not an explicit user-selected recent",
        ));
    }
    Ok((canonical, preferences.vela_binary_path()))
}

fn runtime_policy() -> RuntimePolicyDto {
    RuntimePolicyDto {
        interface_commit: ports::vela::INTERFACE_COMMIT.into(),
        interface_tree: ports::vela::INTERFACE_TREE.into(),
        runtime_version: ports::vela::RUNTIME_VERSION.into(),
        runtime_commit: ports::vela::RUNTIME_COMMIT.into(),
        runtime_sha256: ports::vela::PLATFORM_RUNTIME_SHA256.into(),
        read_only: false,
        tranche: "3".into(),
        mutation_scope:
            "bounded_local_execution_submission_verification_and_attributed_repository_decision"
                .into(),
        tranche_three_enabled: true,
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
    let preferences = state
        .preferences
        .lock()
        .map_err(|_| state_error())?
        .clear_recents()
        .map_err(CommandErrorDto::from)?;
    state.privileged.lock().map_err(|_| state_error())?.clear();
    Ok(preferences)
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

#[tauri::command]
pub(crate) async fn preview_worktree(
    path: String,
    target_ref: String,
    state: State<'_, AppState>,
) -> Result<Option<WorktreePreviewDto>, CommandErrorDto> {
    let (canonical, _) = selected_repository(&path, &state)?;
    let destination = tauri::async_runtime::spawn_blocking(move || {
        FileDialog::new()
            .set_title("Choose one existing empty folder for the detached worktree")
            .pick_folder()
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("worktree dialog failed: {error}")))?;
    let Some(destination) = destination else {
        return Ok(None);
    };
    tauri::async_runtime::spawn_blocking(move || {
        ports::git::preview_worktree(&canonical, &target_ref, &destination)
    })
    .await
    .map_err(|error| CommandErrorDto::new("internal", format!("worktree preview failed: {error}")))?
    .map(Some)
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn create_worktree(
    preview: WorktreePreviewDto,
    state: State<'_, AppState>,
) -> Result<Option<WorktreeResultDto>, CommandErrorDto> {
    let (_, binary_path) = selected_repository(&preview.repository_path, &state)?;
    let target_ref = dialog_value(&preview.target_ref)?;
    let target_commit = dialog_value(&preview.target_commit)?;
    let destination = dialog_value(&preview.destination)?;
    let rollback = preview
        .rollback
        .iter()
        .map(|value| dialog_value(value))
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    let description = format!(
        "Create one detached worktree?\n\nTarget ref: {}\nResolved commit: {}\nDestination: {}\n\nRollback: {}",
        target_ref, target_commit, destination, rollback
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Create detached worktree", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let preview_for_create = preview.clone();
    let destination = tauri::async_runtime::spawn_blocking(move || {
        ports::git::create_worktree(&preview_for_create)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("worktree task failed: {error}"))
    })??;
    let destination_for_read = destination.clone();
    let snapshot = tauri::async_runtime::spawn_blocking(move || {
        inspect_path(&destination_for_read, binary_path.as_deref())
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("worktree inspect failed: {error}"))
    })??;
    state
        .preferences
        .lock()
        .map_err(|_| state_error())?
        .remember_repository(&destination)?;
    Ok(Some(WorktreeResultDto {
        destination: destination.display().to_string(),
        target_commit: preview.target_commit,
        rollback: preview.rollback,
        repository: snapshot,
    }))
}

#[tauri::command]
pub(crate) async fn select_native_tool(
    profile: NativeExecProfileDto,
    state: State<'_, AppState>,
) -> Result<Option<NativeToolDto>, CommandErrorDto> {
    let selected = if profile == NativeExecProfileDto::GitDiffCheck {
        None
    } else {
        tauri::async_runtime::spawn_blocking(move || {
            FileDialog::new()
                .set_title("Choose the exact executable for this reviewed profile")
                .pick_file()
        })
        .await
        .map_err(|error| CommandErrorDto::new("dialog", format!("tool dialog failed: {error}")))?
    };
    if profile != NativeExecProfileDto::GitDiffCheck && selected.is_none() {
        return Ok(None);
    }
    let tool = tauri::async_runtime::spawn_blocking(move || {
        ports::native_exec::inspect_tool(profile, selected.as_deref())
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("tool identity failed: {error}"))
    })??;
    state
        .privileged
        .lock()
        .map_err(|_| state_error())?
        .tools
        .insert(profile, tool.clone());
    Ok(Some(tool))
}

#[tauri::command]
pub(crate) async fn preview_native_exec(
    path: String,
    profile: NativeExecProfileDto,
    state: State<'_, AppState>,
) -> Result<NativeExecPreviewDto, CommandErrorDto> {
    let (canonical, _) = selected_repository(&path, &state)?;
    let tool = state
        .privileged
        .lock()
        .map_err(|_| state_error())?
        .tools
        .get(&profile)
        .cloned()
        .ok_or_else(|| {
            CommandErrorDto::new(
                "tool_not_selected",
                "select the exact native tool before previewing this profile",
            )
        })?;
    tauri::async_runtime::spawn_blocking(move || {
        let git = ports::git::inspect(&canonical)?;
        ports::native_exec::preview(&canonical, &git, profile, &tool)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("execution preview failed: {error}"))
    })?
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn run_native_exec(
    run_id: String,
    preview: NativeExecPreviewDto,
    state: State<'_, AppState>,
) -> Result<NativeExecResultDto, CommandErrorDto> {
    validate_run_id(&run_id)?;
    let (canonical, _) = selected_repository(&preview.repository_path, &state)?;
    {
        let mut privileged = state.privileged.lock().map_err(|_| state_error())?;
        if privileged.active_run.is_some() {
            return Err(CommandErrorDto::new(
                "busy",
                "one native command is already running; cancel or wait for it",
            ));
        }
        let selected_tool = privileged.tools.get(&preview.profile).ok_or_else(|| {
            CommandErrorDto::new("tool_not_selected", "native tool selection expired")
        })?;
        if selected_tool != &preview.executable {
            return Err(CommandErrorDto::new(
                "stale",
                "native tool identity differs from the reviewed preview",
            ));
        }
        let cancellation = Arc::new(AtomicBool::new(false));
        privileged.active_run = Some((run_id.clone(), cancellation));
    }
    let cancellation = state
        .privileged
        .lock()
        .map_err(|_| state_error())?
        .active_run
        .as_ref()
        .map(|(_, cancellation)| Arc::clone(cancellation))
        .ok_or_else(state_error)?;
    let run_id_for_task = run_id.clone();
    let task_result = tauri::async_runtime::spawn_blocking(move || {
        let current_git = ports::git::inspect(&canonical)?;
        ports::native_exec::run(run_id_for_task, &current_git, &preview, cancellation)
    })
    .await;
    let mut privileged = state.privileged.lock().map_err(|_| state_error())?;
    let may_store = privileged.take_run(&run_id);
    let result = task_result.map_err(|error| {
        CommandErrorDto::new("internal", format!("native execution task failed: {error}"))
    })?;
    let result = result.map_err(CommandErrorDto::from)?;
    if may_store {
        privileged.remember_run(result.clone());
    }
    Ok(result)
}

#[tauri::command]
pub(crate) fn cancel_native_exec(
    run_id: String,
    state: State<'_, AppState>,
) -> Result<CancelResultDto, CommandErrorDto> {
    validate_run_id(&run_id)?;
    let privileged = state.privileged.lock().map_err(|_| state_error())?;
    let requested = if let Some((active_id, cancellation)) = &privileged.active_run {
        if active_id != &run_id {
            false
        } else {
            cancellation.store(true, Ordering::SeqCst);
            true
        }
    } else {
        false
    };
    Ok(CancelResultDto {
        run_id,
        cancellation_requested: requested,
    })
}

#[tauri::command]
pub(crate) async fn select_evidence_file(
    path: String,
    state: State<'_, AppState>,
) -> Result<Option<EvidenceItemDto>, CommandErrorDto> {
    let (canonical, _) = selected_repository(&path, &state)?;
    let dialog_root = canonical.clone();
    let selected = tauri::async_runtime::spawn_blocking(move || {
        FileDialog::new()
            .set_title("Choose one exact local evidence file")
            .set_directory(dialog_root)
            .pick_file()
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("evidence dialog failed: {error}")))?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let captured = tauri::async_runtime::spawn_blocking(move || {
        let git = ports::git::inspect(&canonical)?;
        ports::evidence::capture_file(&canonical, &git, &selected)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("evidence task failed: {error}"))
    })??;
    let dto = captured.dto.clone();
    state
        .privileged
        .lock()
        .map_err(|_| state_error())?
        .remember_evidence(captured);
    Ok(Some(dto))
}

fn resolve_evidence(
    root: &Path,
    source: &EvidenceSourceDto,
    state: &State<'_, AppState>,
) -> Result<ports::evidence::CapturedEvidence, CommandErrorDto> {
    match source {
        EvidenceSourceDto::LocalFile { path, .. } => {
            let original = state
                .privileged
                .lock()
                .map_err(|_| state_error())?
                .evidence
                .get(&evidence_key(source))
                .cloned()
                .ok_or_else(|| {
                    CommandErrorDto::new(
                        "evidence_not_selected",
                        "evidence file is not an explicit current selection",
                    )
                })?;
            let git = ports::git::inspect(root).map_err(CommandErrorDto::from)?;
            let current = ports::evidence::capture_file(root, &git, Path::new(path))
                .map_err(CommandErrorDto::from)?;
            if current.dto.sha256 != original.dto.sha256
                || current.dto.size != original.dto.size
                || current.dto.source != original.dto.source
            {
                return Err(CommandErrorDto::new(
                    "stale",
                    "selected evidence changed after capture; select it again",
                ));
            }
            Ok(original)
        }
        EvidenceSourceDto::CommandOutput { run_id, stream } => {
            let result = state
                .privileged
                .lock()
                .map_err(|_| state_error())?
                .completed_runs
                .get(run_id)
                .cloned()
                .ok_or_else(|| {
                    CommandErrorDto::new(
                        "run_not_available",
                        "completed run output is no longer in bounded memory",
                    )
                })?;
            ports::evidence::capture_output(&result, stream).map_err(Into::into)
        }
    }
}

#[tauri::command]
pub(crate) async fn preview_evidence_export(
    repository_path: String,
    request: EvidenceExportRequestDto,
    state: State<'_, AppState>,
) -> Result<Option<EvidenceExportPreviewDto>, CommandErrorDto> {
    let (canonical, _) = selected_repository(&repository_path, &state)?;
    let captured = resolve_evidence(&canonical, &request.source, &state)?;
    let file_name = captured.dto.display_name.clone();
    let destination = tauri::async_runtime::spawn_blocking(move || {
        FileDialog::new()
            .set_title("Choose one new local evidence export")
            .set_file_name(file_name)
            .save_file()
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("export dialog failed: {error}")))?;
    let Some(destination) = destination else {
        return Ok(None);
    };
    tauri::async_runtime::spawn_blocking(move || {
        ports::evidence::preview_export(&captured, request, &destination)
    })
    .await
    .map_err(|error| CommandErrorDto::new("internal", format!("export preview failed: {error}")))?
    .map(Some)
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn export_evidence(
    repository_path: String,
    preview: EvidenceExportPreviewDto,
    state: State<'_, AppState>,
) -> Result<Option<EvidenceExportResultDto>, CommandErrorDto> {
    let (canonical, _) = selected_repository(&repository_path, &state)?;
    let destination = dialog_value(&preview.destination)?;
    let output_sha256 = dialog_value(&preview.output_sha256)?;
    let description = format!(
        "Create one {} evidence file?\n\nDestination: {}\nDigest: {}\nSize: {} bytes\nRedaction confirmed: {}\n\nThe selected source evidence will not be modified.",
        if preview.derived {
            "derived"
        } else {
            "exact-copy"
        },
        destination,
        output_sha256,
        preview.output_size,
        preview.redaction_confirmed
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Export reviewed evidence bytes", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let captured = resolve_evidence(&canonical, &preview.request.source, &state)?;
    tauri::async_runtime::spawn_blocking(move || ports::evidence::export(&captured, &preview))
        .await
        .map_err(|error| {
            CommandErrorDto::new("internal", format!("evidence export failed: {error}"))
        })?
        .map(Some)
        .map_err(Into::into)
}

fn resolved_producer_checks(
    draft: &SubmissionDraftDto,
    git: &GitSnapshotDto,
    state: &State<'_, AppState>,
) -> Result<Vec<String>, CommandErrorDto> {
    let privileged = state.privileged.lock().map_err(|_| state_error())?;
    draft
        .producer_check_run_ids
        .iter()
        .map(|run_id| {
            let result = privileged.completed_runs.get(run_id).ok_or_else(|| {
                CommandErrorDto::new(
                    "run_not_available",
                    format!("producer check run {run_id} is no longer in bounded memory"),
                )
            })?;
            if result.source_commit != git.head_commit || result.source_tree != git.head_tree {
                return Err(CommandErrorDto::new(
                    "stale",
                    format!("producer check run {run_id} belongs to a different source revision"),
                ));
            }
            Ok(format!(
                "{}:{}",
                result.producer_check_method, result.producer_check_outcome
            ))
        })
        .collect()
}

fn preflight_submission_draft(draft: &SubmissionDraftDto) -> Result<(), CommandErrorDto> {
    if draft.artifacts.is_empty()
        || draft.artifacts.len() > 32
        || draft.caveats.is_empty()
        || draft.caveats.len() > 32
        || draft.conditions.len() > 32
        || draft.verification_requirements.len() > 32
        || draft.producer_check_run_ids.len() > 16
    {
        return Err(CommandErrorDto::new(
            "invalid_input",
            "Submission draft exceeds the closed bounded item counts",
        ));
    }
    Ok(())
}

fn validate_submission_artifacts_selected(
    root: &Path,
    draft: &SubmissionDraftDto,
    state: &State<'_, AppState>,
) -> Result<(), CommandErrorDto> {
    let privileged = state.privileged.lock().map_err(|_| state_error())?;
    for artifact in &draft.artifacts {
        let absolute = root.join(&artifact.path);
        let canonical = std::fs::canonicalize(&absolute).map_err(|error| {
            CommandErrorDto::new(
                "invalid_input",
                format!("resolve selected Artifact {}: {error}", artifact.path),
            )
        })?;
        let key = format!("file:{}", canonical.display());
        let captured = privileged.evidence.get(&key).ok_or_else(|| {
            CommandErrorDto::new(
                "evidence_not_selected",
                format!(
                    "Submission Artifact {} is not an explicit current evidence selection",
                    artifact.path
                ),
            )
        })?;
        if captured.dto.sha256 != artifact.sha256 || captured.dto.size != artifact.size {
            return Err(CommandErrorDto::new(
                "stale",
                format!(
                    "Submission Artifact {} differs from its captured digest or size",
                    artifact.path
                ),
            ));
        }
        let current_git = ports::git::inspect(root).map_err(CommandErrorDto::from)?;
        if captured.dto.source_commit != current_git.head_commit
            || captured.dto.source_tree != current_git.head_tree
        {
            return Err(CommandErrorDto::new(
                "stale",
                format!(
                    "Submission Artifact {} was captured at a different source revision",
                    artifact.path
                ),
            ));
        }
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn preview_submission_draft(
    path: String,
    draft: SubmissionDraftDto,
    state: State<'_, AppState>,
) -> Result<SubmissionPreviewDto, CommandErrorDto> {
    preflight_submission_draft(&draft)?;
    let (canonical, binary) = selected_repository(&path, &state)?;
    let binary = binary.ok_or_else(|| {
        CommandErrorDto::new(
            "vela_unavailable",
            "select the pinned signed Vela v0.977.1 runtime before reviewing a Submission",
        )
    })?;
    let git = ports::git::inspect(&canonical)?;
    validate_submission_artifacts_selected(&canonical, &draft, &state)?;
    let checks = resolved_producer_checks(&draft, &git, &state)?;
    tauri::async_runtime::spawn_blocking(move || {
        ports::vela::preview_submission_draft(&canonical, &binary, &git, draft, checks)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("Submission preview failed: {error}"))
    })?
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn submit_submission_draft(
    preview: SubmissionPreviewDto,
    state: State<'_, AppState>,
) -> Result<Option<SubmissionResultDto>, CommandErrorDto> {
    preflight_submission_draft(&preview.draft)?;
    let (canonical, binary) = selected_repository(&preview.repository_path, &state)?;
    let binary = binary.ok_or_else(|| {
        CommandErrorDto::new("vela_unavailable", "Vela runtime selection is unavailable")
    })?;
    let git = ports::git::inspect(&canonical)?;
    validate_submission_artifacts_selected(&canonical, &preview.draft, &state)?;
    let checks = resolved_producer_checks(&preview.draft, &git, &state)?;
    let rebuilt = ports::vela::preview_submission_draft(
        &canonical,
        &binary,
        &git,
        preview.draft.clone(),
        checks,
    )?;
    if rebuilt != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "Submission draft, source, Artifact, producer check, or Vela identity changed; review again",
        ));
    }
    let producer = dialog_value(&preview.draft.producer)?;
    let repository = dialog_value(&preview.repository_path)?;
    let source_commit = dialog_value(&preview.source_commit)?;
    let description = format!(
        "Submit one producer-authenticated pending Proposal?\n\nProducer: {}\nRepository: {}\nSource commit: {}\nArtifacts: {} ({} bytes)\n\nAccepted-event delta must remain zero. No Verification or Decision is available.",
        producer,
        repository,
        source_commit,
        preview.draft.artifacts.len(),
        preview.artifact_total_bytes
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Import ordinary Submission v3", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let git = ports::git::inspect(&canonical)?;
    validate_submission_artifacts_selected(&canonical, &preview.draft, &state)?;
    let checks = resolved_producer_checks(&preview.draft, &git, &state)?;
    let confirmed_preview = ports::vela::preview_submission_draft(
        &canonical,
        &binary,
        &git,
        preview.draft.clone(),
        checks,
    )?;
    if confirmed_preview != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "Submission inputs changed during confirmation; review the exact operation again",
        ));
    }
    tauri::async_runtime::spawn_blocking(move || ports::vela::submit_draft(&binary, &preview))
        .await
        .map_err(|error| {
            CommandErrorDto::new("internal", format!("Submission task failed: {error}"))
        })?
        .map(Some)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn select_submission_import(
    path: String,
    state: State<'_, AppState>,
) -> Result<Option<SubmissionImportPreviewDto>, CommandErrorDto> {
    let (canonical, binary) = selected_repository(&path, &state)?;
    let binary = binary.ok_or_else(|| {
        CommandErrorDto::new(
            "vela_unavailable",
            "select the pinned signed Vela runtime before importing",
        )
    })?;
    let envelope = tauri::async_runtime::spawn_blocking(move || {
        FileDialog::new()
            .set_title("Choose one signed Submission v3 envelope")
            .pick_file()
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("dialog", format!("Submission dialog failed: {error}"))
    })?;
    let Some(envelope) = envelope else {
        return Ok(None);
    };
    tauri::async_runtime::spawn_blocking(move || {
        let git = ports::git::inspect(&canonical)?;
        ports::vela::preview_submission_import(&canonical, &binary, &git, &envelope)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new(
            "internal",
            format!("Submission import preview failed: {error}"),
        )
    })?
    .map(Some)
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn import_submission(
    preview: SubmissionImportPreviewDto,
    state: State<'_, AppState>,
) -> Result<Option<SubmissionResultDto>, CommandErrorDto> {
    let (canonical, binary) = selected_repository(&preview.repository_path, &state)?;
    let binary = binary.ok_or_else(|| {
        CommandErrorDto::new("vela_unavailable", "Vela runtime selection is unavailable")
    })?;
    let git = ports::git::inspect(&canonical)?;
    let rebuilt = ports::vela::preview_submission_import(
        &canonical,
        &binary,
        &git,
        Path::new(&preview.envelope_path),
    )?;
    if rebuilt != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "signed Submission envelope, Artifact, source, or Vela identity changed; review again",
        ));
    }
    let assertion_summary = preview.assertion.chars().take(256).collect::<String>();
    let assertion_summary = dialog_value(&assertion_summary)?;
    let producer_summary = dialog_value(&preview.producer)?;
    let envelope_path = dialog_value(&preview.envelope_path)?;
    let envelope_sha256 = dialog_value(&preview.envelope_sha256)?;
    let description = format!(
        "Import one signed Submission v3?\n\nProducer: {}\nAssertion: {}\nEnvelope: {}\nDigest: {}\nArtifacts: {}\n\nThe signed Vela CLI will verify the signature. Accepted-event delta must remain zero.",
        producer_summary,
        assertion_summary,
        envelope_path,
        envelope_sha256,
        preview.artifacts.len()
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Import signed Submission v3", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let git = ports::git::inspect(&canonical)?;
    let confirmed_preview = ports::vela::preview_submission_import(
        &canonical,
        &binary,
        &git,
        Path::new(&preview.envelope_path),
    )?;
    if confirmed_preview != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "signed Submission inputs changed during confirmation; review the exact import again",
        ));
    }
    tauri::async_runtime::spawn_blocking(move || ports::vela::import_submission(&binary, &preview))
        .await
        .map_err(|error| {
            CommandErrorDto::new("internal", format!("Submission import failed: {error}"))
        })?
        .map(Some)
        .map_err(Into::into)
}

fn tranche_three_context(
    path: &str,
    state: &State<'_, AppState>,
) -> Result<(PathBuf, PathBuf, GitSnapshotDto), CommandErrorDto> {
    let (repository, binary) = selected_repository(path, state)?;
    let binary = binary.ok_or_else(|| {
        CommandErrorDto::new(
            "vela_unavailable",
            "select the exact signed Vela v0.977.1 runtime before using Tranche 3",
        )
    })?;
    let git = ports::git::inspect(&repository)?;
    Ok((repository, binary, git))
}

fn remember_recovery(
    repository: &Path,
    result: &VerificationResultDto,
    state: &State<'_, AppState>,
) -> Result<(), CommandErrorDto> {
    if let Some(refusal) = &result.refusal
        && refusal.code.as_deref() == Some("repository_incomplete")
        && let Some(operation_id) = &refusal.operation_id
    {
        state
            .privileged
            .lock()
            .map_err(|_| state_error())?
            .recovery_operations
            .insert(repository.display().to_string(), operation_id.clone());
    }
    Ok(())
}

fn remember_decision_recovery(
    repository: &Path,
    result: &DecisionExecutionDto,
    state: &State<'_, AppState>,
) -> Result<(), CommandErrorDto> {
    if let Some(refusal) = &result.refusal
        && refusal.code.as_deref() == Some("repository_incomplete")
        && let Some(operation_id) = &refusal.operation_id
    {
        state
            .privileged
            .lock()
            .map_err(|_| state_error())?
            .recovery_operations
            .insert(repository.display().to_string(), operation_id.clone());
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn refresh_decision_inbox(
    path: String,
    state: State<'_, AppState>,
) -> Result<DecisionInboxDto, CommandErrorDto> {
    let (repository, binary, _) = tranche_three_context(&path, &state)?;
    tauri::async_runtime::spawn_blocking(move || {
        ports::tranche_three::decision_inbox(&repository, &binary)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("Decision Inbox task failed: {error}"))
    })?
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn select_verification_method(
    path: String,
    state: State<'_, AppState>,
) -> Result<Option<VerificationMethodDto>, CommandErrorDto> {
    let (repository, _, _) = tranche_three_context(&path, &state)?;
    let selected = tauri::async_runtime::spawn_blocking(|| {
        FileDialog::new()
            .set_title("Choose one retained Verification Method JSON file")
            .pick_file()
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("Method dialog failed: {error}")))?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    tauri::async_runtime::spawn_blocking(move || {
        ports::tranche_three::inspect_verification_method(&repository, &selected)
    })
    .await
    .map_err(|error| CommandErrorDto::new("internal", format!("Method task failed: {error}")))?
    .map(Some)
    .map_err(Into::into)
}

fn validate_verification_outputs_selected(
    repository: &Path,
    draft: &VerificationDraftDto,
    state: &State<'_, AppState>,
) -> Result<(), CommandErrorDto> {
    let privileged = state.privileged.lock().map_err(|_| state_error())?;
    for relative in &draft.output_paths {
        let canonical = std::fs::canonicalize(repository.join(relative)).map_err(|error| {
            CommandErrorDto::new(
                "invalid_input",
                format!("resolve Verification output {relative}: {error}"),
            )
        })?;
        if !privileged
            .evidence
            .contains_key(&format!("file:{}", canonical.display()))
        {
            return Err(CommandErrorDto::new(
                "evidence_not_selected",
                format!(
                    "Verification output {relative} is not an explicit current evidence selection"
                ),
            ));
        }
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn preview_verification_record(
    path: String,
    draft: VerificationDraftDto,
    state: State<'_, AppState>,
) -> Result<VerificationPreviewDto, CommandErrorDto> {
    let (repository, binary, git) = tranche_three_context(&path, &state)?;
    validate_verification_outputs_selected(&repository, &draft, &state)?;
    tauri::async_runtime::spawn_blocking(move || {
        ports::tranche_three::preview_verification_record(&repository, &binary, &git, draft)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new(
            "internal",
            format!("Verification preview task failed: {error}"),
        )
    })?
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn record_verification(
    preview: VerificationPreviewDto,
    state: State<'_, AppState>,
) -> Result<Option<VerificationResultDto>, CommandErrorDto> {
    let (repository, binary, git) = tranche_three_context(&preview.repository_path, &state)?;
    validate_verification_outputs_selected(&repository, &preview.draft, &state)?;
    let rebuilt = ports::tranche_three::preview_verification_record(
        &repository,
        &binary,
        &git,
        preview.draft.clone(),
    )?;
    if rebuilt != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "Verification inputs, evidence, Proposal, source, or Vela identity changed; review again",
        ));
    }
    let actor = dialog_value(&preview.draft.actor)?;
    let proposal = dialog_value(&preview.draft.proposal_id)?;
    let method = dialog_value(&preview.draft.method.repository_relative_path)?;
    let description = format!(
        "Record one scoped Verification?\n\nAttesting actor: {actor}\nProposal: {proposal}\nMethod: {method}\nOutcome: {}\nDeclared independent of: {}\nShared dependencies: {}\n\nAuthority effect: none. This does not accept, reject, create an Event, or change Standing.",
        preview.draft.outcome,
        preview.draft.independent_of.len(),
        preview.draft.shared_dependencies.len()
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Record scoped Verification", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let git = ports::git::inspect(&repository)?;
    validate_verification_outputs_selected(&repository, &preview.draft, &state)?;
    let final_preview = ports::tranche_three::preview_verification_record(
        &repository,
        &binary,
        &git,
        preview.draft.clone(),
    )?;
    if final_preview != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "Verification changed after confirmation; nothing was recorded",
        ));
    }
    let result = tauri::async_runtime::spawn_blocking({
        let repository = repository.clone();
        let binary = binary.clone();
        move || ports::tranche_three::record_verification(&repository, &binary, &preview)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("Verification task failed: {error}"))
    })??;
    remember_recovery(&repository, &result, &state)?;
    Ok(Some(result))
}

#[tauri::command]
pub(crate) async fn select_verification_import(
    path: String,
    state: State<'_, AppState>,
) -> Result<Option<VerificationImportPreviewDto>, CommandErrorDto> {
    let (repository, binary, git) = tranche_three_context(&path, &state)?;
    let selected = tauri::async_runtime::spawn_blocking(|| {
        FileDialog::new()
            .set_title("Choose one signed Verification Record v2 envelope")
            .pick_file()
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new(
            "dialog",
            format!("Verification import dialog failed: {error}"),
        )
    })?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    tauri::async_runtime::spawn_blocking(move || {
        ports::tranche_three::preview_verification_import(&repository, &binary, &git, &selected)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new(
            "internal",
            format!("Verification import task failed: {error}"),
        )
    })?
    .map(Some)
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn import_verification(
    preview: VerificationImportPreviewDto,
    state: State<'_, AppState>,
) -> Result<Option<VerificationResultDto>, CommandErrorDto> {
    let (repository, binary, git) = tranche_three_context(&preview.repository_path, &state)?;
    let rebuilt = ports::tranche_three::preview_verification_import(
        &repository,
        &binary,
        &git,
        Path::new(&preview.envelope_path),
    )?;
    if rebuilt != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "Verification envelope, Proposal, source, or Vela identity changed; review again",
        ));
    }
    let verifier = dialog_value(&preview.verifier)?;
    let proposal = dialog_value(&preview.proposal_id)?;
    let digest = dialog_value(&preview.envelope_sha256)?;
    let description = format!(
        "Import one signed scoped Verification?\n\nVerifier: {verifier}\nProposal: {proposal}\nEnvelope: {digest}\nOutcome: {}\n\nAuthority effect: none. The signed Vela CLI verifies the exact signature and current bindings.",
        preview.outcome
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Import scoped Verification", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let git = ports::git::inspect(&repository)?;
    let final_preview = ports::tranche_three::preview_verification_import(
        &repository,
        &binary,
        &git,
        Path::new(&preview.envelope_path),
    )?;
    if final_preview != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "Verification import changed after confirmation; nothing was imported",
        ));
    }
    let result = tauri::async_runtime::spawn_blocking({
        let repository = repository.clone();
        let binary = binary.clone();
        move || ports::tranche_three::import_verification(&repository, &binary, &preview)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("Verification import failed: {error}"))
    })??;
    remember_recovery(&repository, &result, &state)?;
    Ok(Some(result))
}

#[tauri::command]
pub(crate) async fn preview_decision(
    path: String,
    request: DecisionRequestDto,
    state: State<'_, AppState>,
) -> Result<DecisionPreviewDto, CommandErrorDto> {
    let (repository, binary, git) = tranche_three_context(&path, &state)?;
    tauri::async_runtime::spawn_blocking(move || {
        ports::tranche_three::preview_decision(&repository, &binary, &git, request)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("Decision preview task failed: {error}"))
    })?
    .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn execute_decision(
    preview: DecisionPreviewDto,
    state: State<'_, AppState>,
) -> Result<Option<DecisionExecutionDto>, CommandErrorDto> {
    let (repository, binary, git) = tranche_three_context(&preview.repository_path, &state)?;
    let rebuilt = ports::tranche_three::preview_decision(
        &repository,
        &binary,
        &git,
        preview.request.clone(),
    )?;
    if rebuilt != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "Decision Inbox entry, Proposal, Verification set, source, or Vela identity changed; review again",
        ));
    }
    let performer = dialog_value(&preview.request.performer)?;
    let entry = dialog_value(&preview.entry.entry_root)?;
    let proposal = dialog_value(&preview.entry.proposal_id)?;
    let successor = dialog_value(&preview.expected_successor.repository_root)?;
    let description = format!(
        "Execute one attributed {:?} Decision?\n\nPerformer: {performer} ({})\nRepository authority principal: {}\nAuthentication: {}\nTransaction signer: {}\nProposal: {proposal}\nEntry root: {entry}\nExpected successor repository: {successor}\nVerification records: {}\n\nThis changes Repository scientific state if Vela authenticates, authorizes, and commits it. Do not retry after a post-commit receipt failure.",
        preview.request.action,
        preview.performer_kind,
        preview.repository_authority_principal,
        preview.authentication,
        preview.transaction_signer,
        preview.entry.verifications.len()
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Execute attributed Repository Decision", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let git = ports::git::inspect(&repository)?;
    let final_preview = ports::tranche_three::preview_decision(
        &repository,
        &binary,
        &git,
        preview.request.clone(),
    )?;
    if final_preview != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "Decision changed after confirmation; no authority command was started",
        ));
    }
    let result = tauri::async_runtime::spawn_blocking({
        let repository = repository.clone();
        let binary = binary.clone();
        move || ports::tranche_three::execute_decision(&repository, &binary, &preview)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("Decision task failed: {error}"))
    })??;
    remember_decision_recovery(&repository, &result, &state)?;
    Ok(Some(result))
}

#[tauri::command]
pub(crate) async fn preview_recovery(
    path: String,
    operation_id: String,
    state: State<'_, AppState>,
) -> Result<RecoveryPreviewDto, CommandErrorDto> {
    let (repository, binary, git) = tranche_three_context(&path, &state)?;
    let remembered = state
        .privileged
        .lock()
        .map_err(|_| state_error())?
        .recovery_operations
        .get(&repository.display().to_string())
        .cloned();
    if remembered.as_deref() != Some(operation_id.as_str()) {
        return Err(CommandErrorDto::new(
            "not_selected",
            "recovery operation was not surfaced by a structured repository_incomplete refusal in this process",
        ));
    }
    ports::tranche_three::preview_recovery(&repository, &binary, &git, &operation_id)
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn recover_transaction(
    preview: RecoveryPreviewDto,
    state: State<'_, AppState>,
) -> Result<Option<RecoveryResultDto>, CommandErrorDto> {
    let (repository, binary, git) = tranche_three_context(&preview.repository_path, &state)?;
    let remembered = state
        .privileged
        .lock()
        .map_err(|_| state_error())?
        .recovery_operations
        .get(&repository.display().to_string())
        .cloned();
    if remembered.as_deref() != Some(preview.operation_id.as_str()) {
        return Err(CommandErrorDto::new(
            "not_selected",
            "recovery operation is not the exact surfaced operation",
        ));
    }
    let rebuilt =
        ports::tranche_three::preview_recovery(&repository, &binary, &git, &preview.operation_id)?;
    if rebuilt != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "recovery source or Vela identity changed; review again",
        ));
    }
    let operation = dialog_value(&preview.operation_id)?;
    let description = format!(
        "Recover one exact Vela transaction?\n\nOperation: {operation}\nRepository: {}\n\nThis never retries or chooses a Decision. Vela applies only the exact signed recovery journal.",
        preview.repository_path
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Recover exact Vela transaction", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let git = ports::git::inspect(&repository)?;
    let final_preview =
        ports::tranche_three::preview_recovery(&repository, &binary, &git, &preview.operation_id)?;
    if final_preview != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "recovery changed after confirmation; nothing was recovered",
        ));
    }
    let repository_key = preview.repository_path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        ports::tranche_three::recover_transaction(&repository, &binary, &preview)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("recovery task failed: {error}"))
    })??;
    if result.refusal.is_none() && result.repository_blocked_after == Some(false) {
        state
            .privileged
            .lock()
            .map_err(|_| state_error())?
            .recovery_operations
            .remove(&repository_key);
    }
    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::Path,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    };

    use sha2::{Digest, Sha256};

    use super::{PrivilegedState, dialog_value, inspect_path, preflight_submission_draft};
    use crate::contracts::{
        NativeExecProfileDto, NativeExecResultDto, NativeExecStateDto, NativeOutputDto,
        SubmissionDraftDto,
    };

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

    #[test]
    fn clear_during_run_cannot_repopulate_completed_output() {
        let mut state = PrivilegedState::default();
        let cancellation = Arc::new(AtomicBool::new(false));
        state.active_run = Some(("run-clear-race".into(), Arc::clone(&cancellation)));
        state.clear();
        assert!(cancellation.load(Ordering::SeqCst));
        let output = NativeOutputDto {
            stream: "stdout".into(),
            sha256: format!("sha256:{}", "0".repeat(64)),
            size: 0,
            content_base64: String::new(),
            content_utf8: Some(String::new()),
            truncated: false,
        };
        let result = NativeExecResultDto {
            run_id: "run-clear-race".into(),
            profile: NativeExecProfileDto::GitDiffCheck,
            state: NativeExecStateDto::Cancelled,
            exit_code: None,
            started_at_unix_ms: 1,
            completed_at_unix_ms: 2,
            source_commit: "1".repeat(40),
            source_tree: "2".repeat(40),
            executable_sha256: format!("sha256:{}", "3".repeat(64)),
            stdout: output.clone(),
            stderr: NativeOutputDto {
                stream: "stderr".into(),
                ..output
            },
            producer_check_method: "vela-workbench-git-diff-check".into(),
            producer_check_outcome: "skipped".into(),
        };
        let may_store = state.take_run("run-clear-race");
        if may_store {
            state.remember_run(result);
        }
        assert!(!may_store);
        assert!(state.completed_runs.is_empty());
    }

    #[test]
    fn submission_preflight_is_bounded_before_filesystem_work() {
        let draft = SubmissionDraftDto {
            assertion: "bounded result".into(),
            claim_type: "computational".into(),
            conditions: vec!["condition".into(); 33],
            replayability: "exact".into(),
            artifacts: vec![],
            caveats: vec!["bounded scope".into()],
            producer_check_run_ids: vec![],
            verification_requirements: vec![],
            source_run: None,
            producer: "agent:test".into(),
        };
        assert!(preflight_submission_draft(&draft).is_err());
    }

    #[test]
    fn native_dialog_values_escape_untrusted_control_text() {
        let encoded = dialog_value("claim\nDigest: fake\u{1b}").expect("JSON string");
        assert!(!encoded.contains('\n'));
        assert!(encoded.contains("\\n"));
        assert!(encoded.contains("\\u001b"));
    }
}
