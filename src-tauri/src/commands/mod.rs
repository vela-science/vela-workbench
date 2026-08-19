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
        NativeToolDto, OpenGaussGitIdentityDto, OpenGaussHandoffPreviewDto,
        OpenGaussHandoffReceiptDto, OpenGaussSelectedCheckDto, OpenGaussSelectedEvidenceDto,
        PreferencesDto, ProblemHandoffDto, ProblemHandoffSourceDto, RecoveryPreviewDto,
        RecoveryResultDto, RepositorySnapshotDto, RuntimePolicyDto, SubmissionDraftDto,
        SubmissionImportPreviewDto, SubmissionPreviewDto, SubmissionResultDto, VelaBinaryDto,
        VelaInspectionDto, VerificationDraftDto, VerificationImportPreviewDto,
        VerificationMethodDto, VerificationPreviewDto, VerificationResultDto, WorktreePreviewDto,
        WorktreeResultDto,
    },
    ports::{self, PortError},
    preferences::PreferencesStore,
};

pub(crate) struct AppState {
    preferences: Mutex<PreferencesStore>,
    privileged: Mutex<PrivilegedState>,
}

#[derive(Clone)]
struct CompletedRun {
    result: NativeExecResultDto,
    preview: NativeExecPreviewDto,
}

#[derive(Default)]
struct PrivilegedState {
    tools: BTreeMap<NativeExecProfileDto, NativeToolDto>,
    active_run: Option<(String, Arc<AtomicBool>)>,
    completed_runs: BTreeMap<String, CompletedRun>,
    evidence: BTreeMap<String, ports::evidence::CapturedEvidence>,
    recovery_operations: BTreeMap<String, String>,
    selected_opengauss: Option<OpenGaussHandoffPreviewDto>,
    opengauss_receipt: Option<OpenGaussHandoffReceiptDto>,
    opengauss_generation: u64,
}

impl PrivilegedState {
    fn clear(&mut self) {
        if let Some((_, cancellation)) = &self.active_run {
            cancellation.store(true, Ordering::SeqCst);
        }
        let opengauss_generation = self.opengauss_generation.wrapping_add(1);
        *self = Self::default();
        self.opengauss_generation = opengauss_generation;
    }

    fn begin_opengauss_selection(&mut self) -> u64 {
        self.opengauss_generation = self.opengauss_generation.wrapping_add(1);
        self.selected_opengauss = None;
        self.opengauss_receipt = None;
        self.opengauss_generation
    }

    fn replace_inspection_recovery(&mut self, repository: &str, operation_id: Option<&str>) {
        self.recovery_operations.remove(repository);
        if let Some(operation_id) = operation_id {
            self.recovery_operations
                .insert(repository.to_string(), operation_id.to_string());
        }
    }

    fn remember_run(&mut self, result: NativeExecResultDto, preview: NativeExecPreviewDto) {
        self.completed_runs
            .insert(result.run_id.clone(), CompletedRun { result, preview });
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

fn decision_dialog_intent(
    request: &DecisionRequestDto,
) -> Result<(String, String), CommandErrorDto> {
    let reason = dialog_value(&request.reason)?;
    let session = dialog_value(request.session_ref.as_deref().unwrap_or("none"))?;
    Ok((reason, session))
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

#[tauri::command]
pub(crate) fn review_problem_handoff(url: String) -> Result<ProblemHandoffDto, CommandErrorDto> {
    ports::problem_handoff::parse(&url).map_err(Into::into)
}

#[tauri::command]
pub(crate) fn open_problem_handoff(
    handoff: ProblemHandoffDto,
) -> Result<LaunchResultDto, CommandErrorDto> {
    let confirmed = ports::problem_handoff::parse(&handoff.handoff_url)?;
    if confirmed != handoff {
        return Err(CommandErrorDto::new(
            "stale",
            "Problem handoff fields changed after native review",
        ));
    }
    let cwd = std::env::current_dir().map_err(|error| {
        CommandErrorDto::new("internal", format!("resolve Workbench directory: {error}"))
    })?;
    ports::launch::launch_https(&cwd, &handoff.problem_url)?;
    Ok(LaunchResultDto {
        target: handoff.problem_url,
        owner: "default HTTPS browser".into(),
    })
}

#[tauri::command]
pub(crate) fn review_problem_handoff_source(
    path: String,
    handoff: ProblemHandoffDto,
    state: State<'_, AppState>,
) -> Result<ProblemHandoffSourceDto, CommandErrorDto> {
    let confirmed = ports::problem_handoff::parse(&handoff.handoff_url)?;
    if confirmed != handoff {
        return Err(CommandErrorDto::new(
            "stale",
            "Problem handoff fields changed after native review",
        ));
    }
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
    let git = ports::git::inspect(&canonical)?;
    let (remote_matches, revision_matches) =
        ports::problem_handoff::source_matches(&git.remotes, &git.head_commit, &handoff);
    let ready = remote_matches && revision_matches;
    let note = match (remote_matches, revision_matches) {
        (true, true) => {
            "The selected checkout matches the handoff source remote and exact revision."
        }
        (true, false) => {
            "The source remote matches, but the selected checkout is at a different revision. Create a detached worktree at the requested ref before beginning work."
        }
        (false, true) => {
            "The revision matches, but no selected fetch remote matches the handoff source repository."
        }
        (false, false) => {
            "The selected checkout does not match the handoff source repository or revision."
        }
    };
    Ok(ProblemHandoffSourceDto {
        repository_path: git.root,
        source_repository_url: handoff.source_repository_url,
        source_revision: handoff.source_revision,
        selected_head: git.head_commit,
        remote_matches,
        revision_matches,
        ready,
        note: note.into(),
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
    remember_inspection_recovery(&snapshot, &state)?;
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
    let snapshot = tauri::async_runtime::spawn_blocking(move || {
        inspect_path(&canonical, binary_path.as_deref())
    })
    .await
    .map_err(|error| CommandErrorDto::new("internal", format!("repository task failed: {error}")))?
    .map_err(CommandErrorDto::from)?;
    remember_inspection_recovery(&snapshot, &state)?;
    Ok(snapshot)
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

fn environment_dialog(entries: &[(String, String)]) -> Result<String, CommandErrorDto> {
    entries
        .iter()
        .map(|(name, value)| Ok(format!("{name}={}", dialog_value(value)?)))
        .collect::<Result<Vec<_>, _>>()
        .map(|values| values.join("\n"))
}

fn dto_environment_dialog(
    entries: &[crate::contracts::EnvironmentEntryDto],
) -> Result<String, CommandErrorDto> {
    entries
        .iter()
        .map(|entry| Ok(format!("{}={}", entry.name, dialog_value(&entry.value)?)))
        .collect::<Result<Vec<_>, _>>()
        .map(|values| values.join("\n"))
}

#[tauri::command]
pub(crate) async fn select_opengauss(
    path: String,
    state: State<'_, AppState>,
) -> Result<Option<OpenGaussHandoffPreviewDto>, CommandErrorDto> {
    let (canonical, _) = selected_repository(&path, &state)?;
    let selection_generation = state
        .privileged
        .lock()
        .map_err(|_| state_error())?
        .begin_opengauss_selection();
    let selected = tauri::async_runtime::spawn_blocking(|| {
        FileDialog::new()
            .set_title("Choose the exact OpenGauss executable named gauss")
            .pick_file()
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("OpenGauss dialog failed: {error}")))?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let candidate = tauri::async_runtime::spawn_blocking(move || {
        ports::opengauss::inspect_candidate(&selected)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new(
            "internal",
            format!("OpenGauss identity task failed: {error}"),
        )
    })??;
    let candidate_path = dialog_value(&candidate.path.display().to_string())?;
    let candidate_sha = dialog_value(&candidate.sha256)?;
    let cwd = dialog_value(&canonical.display().to_string())?;
    let environment = ports::environment_summary(candidate.path.parent());
    let environment = environment_dialog(&environment)?;
    let description = format!(
        "Run one fixed OpenGauss version probe?\n\nExecutable: {candidate_path}\nSHA-256: {candidate_sha}\nSize: {} bytes\nFixed argv: [\"--version\"]\nWorking directory: {cwd}\nCleared and bounded probe environment:\n{environment}\nTimeout: 20000 ms\nMaximum stdout: 262144 bytes\nMaximum stderr: 131072 bytes\n\n{}",
        candidate.size,
        ports::opengauss::TRUST_WARNING,
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Inspect selected OpenGauss executable", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let preview = tauri::async_runtime::spawn_blocking(move || {
        let git = ports::git::inspect(&canonical)?;
        ports::opengauss::preview(&canonical, &git, &candidate)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("OpenGauss inspection failed: {error}"))
    })??;
    let mut privileged = state.privileged.lock().map_err(|_| state_error())?;
    if privileged.opengauss_generation != selection_generation {
        return Err(CommandErrorDto::new(
            "stale",
            "OpenGauss selection was cleared or replaced during inspection",
        ));
    }
    privileged.selected_opengauss = Some(preview.clone());
    Ok(Some(preview))
}

#[tauri::command]
pub(crate) async fn launch_opengauss_handoff(
    preview: OpenGaussHandoffPreviewDto,
    state: State<'_, AppState>,
) -> Result<Option<OpenGaussHandoffReceiptDto>, CommandErrorDto> {
    let (canonical, _) = selected_repository(&preview.repository_path, &state)?;
    let (selected, selection_generation) = {
        let privileged = state.privileged.lock().map_err(|_| state_error())?;
        (
            privileged.selected_opengauss.clone().ok_or_else(|| {
                CommandErrorDto::new("stale", "OpenGauss selection expired; select it again")
            })?,
            privileged.opengauss_generation,
        )
    };
    if selected != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "OpenGauss handoff differs from the exact host-owned preview",
        ));
    }
    let checked = preview.clone();
    tauri::async_runtime::spawn_blocking(move || ports::opengauss::validate_static(&checked))
        .await
        .map_err(|error| {
            CommandErrorDto::new("internal", format!("OpenGauss validation failed: {error}"))
        })??;
    let tool = dialog_value(&preview.tool.path)?;
    let version = dialog_value(&preview.tool.version)?;
    let sha = dialog_value(&preview.tool.sha256)?;
    let manifest = dialog_value(&preview.project.manifest_path)?;
    let manifest_sha = dialog_value(&preview.project.manifest_sha256)?;
    let cwd = dialog_value(&preview.cwd)?;
    let backend = dialog_value(&preview.backend_identity)?;
    let launcher_environment = dto_environment_dialog(&preview.launcher_environment)?;
    let description = format!(
        "Open Terminal for an explicit interactive OpenGauss handoff?\n\nExecutable: {tool}\nVersion: {version}\nSHA-256: {sha}\nProject config: {manifest}\nConfig SHA-256: {manifest_sha}\nWorking directory: {cwd}\nInteractive argv boundary: [{tool}]\nBackend/tool identity: {backend}\nExact launcher: /usr/bin/open -a Terminal {cwd}\nCleared bounded launcher environment:\n{launcher_environment}\n\nWorkbench will re-run only the fixed --version probe, then open Terminal at the project root. It will not start OpenGauss, type a slash command, observe hidden model transport, or ingest OpenGauss state. The interactive shell environment is owned by Terminal and is not observed or constrained by Workbench.\n\n{}",
        preview.tool.trust_warning,
    );
    let approved = tauri::async_runtime::spawn_blocking(move || {
        confirmed("Open external OpenGauss work surface", &description)
    })
    .await
    .map_err(|error| CommandErrorDto::new("dialog", format!("confirmation failed: {error}")))?;
    if !approved {
        return Ok(None);
    }
    let expected = preview.clone();
    let validated = tauri::async_runtime::spawn_blocking(move || {
        ports::opengauss::revalidate_preview(&expected)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new(
            "internal",
            format!("OpenGauss revalidation failed: {error}"),
        )
    })??;
    if validated != preview {
        return Err(CommandErrorDto::new(
            "stale",
            "OpenGauss tool, project, or Repository changed after confirmation",
        ));
    }
    {
        let mut privileged = state.privileged.lock().map_err(|_| state_error())?;
        if privileged.opengauss_generation != selection_generation
            || privileged.selected_opengauss.as_ref() != Some(&preview)
        {
            return Err(CommandErrorDto::new("stale", "OpenGauss selection expired"));
        }
        privileged.selected_opengauss = None;
    }
    let launch = tauri::async_runtime::spawn_blocking(move || {
        ports::launch::launch(&canonical, LaunchKindDto::Terminal)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::new("internal", format!("Terminal handoff failed: {error}"))
    })??;
    let receipt = ports::opengauss::launched_receipt(preview, launch.owner);
    let mut privileged = state.privileged.lock().map_err(|_| state_error())?;
    if privileged.opengauss_generation == selection_generation {
        privileged.opengauss_receipt = Some(receipt.clone());
    }
    Ok(Some(receipt))
}

fn opengauss_evidence(item: &EvidenceItemDto) -> OpenGaussSelectedEvidenceDto {
    let source = match &item.source {
        EvidenceSourceDto::LocalFile {
            repository_relative_path,
            ..
        } => repository_relative_path.clone(),
        EvidenceSourceDto::CommandOutput { run_id, stream } => format!("{run_id}:{stream}"),
    };
    OpenGaussSelectedEvidenceDto {
        display_name: item.display_name.clone(),
        sha256: item.sha256.clone(),
        size: item.size,
        media_type: item.media_type.clone(),
        kind_hint: item.kind_hint.clone(),
        source_commit: item.source_commit.clone(),
        source_tree: item.source_tree.clone(),
        source,
    }
}

fn opengauss_check(run: &CompletedRun) -> OpenGaussSelectedCheckDto {
    let result = &run.result;
    let preview = &run.preview;
    OpenGaussSelectedCheckDto {
        run_id: result.run_id.clone(),
        repository_path: preview.repository_path.clone(),
        profile: result.profile,
        state: result.state.clone(),
        exit_code: result.exit_code,
        source_commit: result.source_commit.clone(),
        source_tree: result.source_tree.clone(),
        executable_path: preview.executable.path.clone(),
        executable_sha256: result.executable_sha256.clone(),
        argv: preview.argv.clone(),
        working_directory: preview.working_directory.clone(),
        environment: preview.environment.clone(),
        timeout_ms: preview.timeout_ms,
        max_stdout_bytes: preview.max_stdout_bytes,
        max_stderr_bytes: preview.max_stderr_bytes,
        stdout_sha256: result.stdout.sha256.clone(),
        stderr_sha256: result.stderr.sha256.clone(),
        producer_check_method: result.producer_check_method.clone(),
        producer_check_outcome: result.producer_check_outcome.clone(),
    }
}

fn validate_opengauss_result_bindings(
    repository: &Path,
    git_after: &OpenGaussGitIdentityDto,
    evidence: &[OpenGaussSelectedEvidenceDto],
    checks: &[OpenGaussSelectedCheckDto],
) -> Result<(), CommandErrorDto> {
    let repository = repository.display().to_string();
    if evidence
        .iter()
        .any(|item| item.source_commit != git_after.commit || item.source_tree != git_after.tree)
    {
        return Err(CommandErrorDto::new(
            "stale",
            "selected OpenGauss evidence belongs to a different source revision; capture it again",
        ));
    }
    if checks.iter().any(|item| {
        item.repository_path != repository
            || item.working_directory != repository
            || item.source_commit != git_after.commit
            || item.source_tree != git_after.tree
    }) {
        return Err(CommandErrorDto::new(
            "stale",
            "selected OpenGauss check belongs to a different Repository or source revision; run it again",
        ));
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn refresh_opengauss_handoff(
    receipt: OpenGaussHandoffReceiptDto,
    evidence_sources: Vec<EvidenceSourceDto>,
    check_run_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<OpenGaussHandoffReceiptDto, CommandErrorDto> {
    if evidence_sources.len() > 16 || check_run_ids.len() > 4 {
        return Err(CommandErrorDto::new(
            "invalid_input",
            "OpenGauss receipt accepts at most 16 explicit evidence items and 4 checks",
        ));
    }
    let mut evidence_keys = evidence_sources
        .iter()
        .map(evidence_key)
        .collect::<Vec<_>>();
    evidence_keys.sort();
    if evidence_keys.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(CommandErrorDto::new(
            "invalid_input",
            "duplicate evidence selection",
        ));
    }
    for run_id in &check_run_ids {
        validate_run_id(run_id)?;
    }
    let mut unique_runs = check_run_ids.clone();
    unique_runs.sort();
    if unique_runs.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(CommandErrorDto::new(
            "invalid_input",
            "duplicate check selection",
        ));
    }
    let (canonical, _) = selected_repository(&receipt.preview.repository_path, &state)?;
    if state
        .privileged
        .lock()
        .map_err(|_| state_error())?
        .opengauss_receipt
        .as_ref()
        != Some(&receipt)
    {
        return Err(CommandErrorDto::new(
            "stale",
            "OpenGauss receipt is not the exact current host-owned handoff",
        ));
    }
    let baseline = receipt.clone();
    let mut refreshed =
        tauri::async_runtime::spawn_blocking(move || ports::opengauss::refresh_receipt(&baseline))
            .await
            .map_err(|error| {
                CommandErrorDto::new("internal", format!("OpenGauss refresh failed: {error}"))
            })??;
    let selected_evidence = evidence_sources
        .iter()
        .map(|source| {
            resolve_evidence(&canonical, source, &state).map(|item| opengauss_evidence(&item.dto))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let selected_checks = {
        let privileged = state.privileged.lock().map_err(|_| state_error())?;
        check_run_ids
            .iter()
            .map(|run_id| {
                privileged
                    .completed_runs
                    .get(run_id)
                    .map(opengauss_check)
                    .ok_or_else(|| {
                        CommandErrorDto::new(
                            "run_not_available",
                            format!("completed check {run_id} is no longer in bounded memory"),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let git_after = refreshed
        .git_after
        .as_ref()
        .ok_or_else(|| CommandErrorDto::new("internal", "OpenGauss refresh omitted Git-after"))?;
    validate_opengauss_result_bindings(
        &canonical,
        git_after,
        &selected_evidence,
        &selected_checks,
    )?;
    let confirmed_git = ports::git::inspect(&canonical).map_err(CommandErrorDto::from)?;
    if refreshed.git_after.as_ref() != Some(&ports::opengauss::git_identity(&confirmed_git)) {
        return Err(CommandErrorDto::new(
            "stale",
            "Repository changed while binding selected OpenGauss result evidence",
        ));
    }
    refreshed.selected_evidence = selected_evidence;
    refreshed.selected_checks = selected_checks;
    let mut privileged = state.privileged.lock().map_err(|_| state_error())?;
    if privileged.opengauss_receipt.as_ref() != Some(&receipt) {
        return Err(CommandErrorDto::new(
            "stale",
            "OpenGauss receipt expired during refresh",
        ));
    }
    privileged.opengauss_receipt = Some(refreshed.clone());
    Ok(refreshed)
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
    let preview_for_storage = preview.clone();
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
        privileged.remember_run(result.clone(), preview_for_storage);
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
            let run = state
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
            if run.preview.repository_path != root.display().to_string()
                || run.preview.working_directory != root.display().to_string()
            {
                return Err(CommandErrorDto::new(
                    "stale",
                    "completed command output belongs to a different Repository",
                ));
            }
            ports::evidence::capture_output(&run.result, stream).map_err(Into::into)
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
            let run = privileged.completed_runs.get(run_id).ok_or_else(|| {
                CommandErrorDto::new(
                    "run_not_available",
                    format!("producer check run {run_id} is no longer in bounded memory"),
                )
            })?;
            let result = &run.result;
            if run.preview.repository_path != git.root || run.preview.working_directory != git.root
            {
                return Err(CommandErrorDto::new(
                    "stale",
                    format!("producer check run {run_id} belongs to a different Repository"),
                ));
            }
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
            "select the pinned signed Vela v0.977.3 runtime before reviewing a Submission",
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
            "select the exact signed Vela v0.977.3 runtime before using Vela Repository actions",
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

fn remember_inspection_recovery(
    snapshot: &RepositorySnapshotDto,
    state: &State<'_, AppState>,
) -> Result<(), CommandErrorDto> {
    let mut privileged = state.privileged.lock().map_err(|_| state_error())?;
    if let Some(operation_id) = &snapshot.vela.recovery_operation_id {
        ports::tranche_three::validate_operation_id(operation_id)?;
    }
    privileged.replace_inspection_recovery(
        &snapshot.path,
        snapshot.vela.recovery_operation_id.as_deref(),
    );
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
    let outcome = dialog_value(&preview.outcome)?;
    let description = format!(
        "Import one signed scoped Verification?\n\nVerifier: {verifier}\nProposal: {proposal}\nEnvelope: {digest}\nOutcome: {outcome}\n\nAuthority effect: none. The signed Vela CLI verifies the exact signature and current bindings."
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
    let (reason, session) = decision_dialog_intent(&preview.request)?;
    let description = format!(
        "Execute one attributed {:?} Decision?\n\nPerformer: {performer} ({})\nRepository authority principal: {}\nAuthentication: {}\nTransaction signer: {}\nProposal: {proposal}\nEntry root: {entry}\nReason: {reason}\nSession reference: {session}\nExpected successor repository: {successor}\nVerification records: {}\n\nThis changes Repository scientific state if Vela authenticates, authorizes, and commits it. Do not retry after a post-commit receipt failure.",
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
            "recovery operation was not surfaced by a structured current-operation refusal or signed recovery inspection in this process",
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
    let repository_path = dialog_value(&preview.repository_path)?;
    let description = format!(
        "Recover one exact Vela transaction?\n\nOperation: {operation}\nRepository: {repository_path}\n\nThis never retries or chooses a Decision. Vela applies only the exact signed recovery journal."
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

    use super::{
        PrivilegedState, decision_dialog_intent, dialog_value, inspect_path,
        preflight_submission_draft, validate_opengauss_result_bindings,
    };
    use crate::contracts::{
        DecisionActionDto, DecisionRequestDto, NativeExecProfileDto, NativeExecResultDto,
        NativeExecStateDto, NativeOutputDto, OpenGaussGitIdentityDto, OpenGaussSelectedCheckDto,
        OpenGaussSelectedEvidenceDto, SubmissionDraftDto,
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
        let first_open_gauss_generation = state.begin_opengauss_selection();
        let cancellation = Arc::new(AtomicBool::new(false));
        state.active_run = Some(("run-clear-race".into(), Arc::clone(&cancellation)));
        state.clear();
        assert!(cancellation.load(Ordering::SeqCst));
        assert_ne!(state.opengauss_generation, first_open_gauss_generation);
        assert!(state.selected_opengauss.is_none());
        assert!(state.opengauss_receipt.is_none());
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
        let _ = result;
        assert!(!may_store);
        assert!(state.completed_runs.is_empty());
    }

    #[test]
    fn fresh_inspection_replaces_and_revokes_recovery_capability() {
        let repository = "/private/tmp/recovery-repository";
        let first = format!("vop_{}", "a".repeat(64));
        let second = format!("vop_{}", "b".repeat(64));
        let mut state = PrivilegedState::default();
        state.replace_inspection_recovery(repository, Some(&first));
        assert_eq!(
            state.recovery_operations.get(repository).map(String::as_str),
            Some(first.as_str())
        );
        state.replace_inspection_recovery(repository, Some(&second));
        assert_eq!(
            state.recovery_operations.get(repository).map(String::as_str),
            Some(second.as_str())
        );
        state.replace_inspection_recovery(repository, None);
        assert!(!state.recovery_operations.contains_key(repository));
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
    fn opengauss_result_bindings_require_exact_repository_and_git_after() {
        let repository = Path::new("/private/tmp/disposable-open-gauss");
        let after = OpenGaussGitIdentityDto {
            branch: Some("main".into()),
            commit: "1".repeat(40),
            tree: "2".repeat(40),
            dirty: false,
            changed_paths: 0,
        };
        let evidence = OpenGaussSelectedEvidenceDto {
            display_name: "result.lean".into(),
            sha256: format!("sha256:{}", "3".repeat(64)),
            size: 12,
            media_type: "text/plain".into(),
            kind_hint: "lean-source".into(),
            source_commit: after.commit.clone(),
            source_tree: after.tree.clone(),
            source: "Result.lean".into(),
        };
        let check = OpenGaussSelectedCheckDto {
            run_id: "run-opengauss-binding".into(),
            repository_path: repository.display().to_string(),
            profile: NativeExecProfileDto::LeanBuild,
            state: NativeExecStateDto::Completed,
            exit_code: Some(0),
            source_commit: after.commit.clone(),
            source_tree: after.tree.clone(),
            executable_path: "/usr/local/bin/lake".into(),
            executable_sha256: format!("sha256:{}", "4".repeat(64)),
            argv: vec!["build".into()],
            working_directory: repository.display().to_string(),
            environment: Vec::new(),
            timeout_ms: 120_000,
            max_stdout_bytes: 1_048_576,
            max_stderr_bytes: 1_048_576,
            stdout_sha256: format!("sha256:{}", "5".repeat(64)),
            stderr_sha256: format!("sha256:{}", "6".repeat(64)),
            producer_check_method: "vela-workbench-lean-build".into(),
            producer_check_outcome: "pass".into(),
        };
        validate_opengauss_result_bindings(
            repository,
            &after,
            std::slice::from_ref(&evidence),
            std::slice::from_ref(&check),
        )
        .expect("exact result binding");

        let mut stale_evidence = evidence;
        stale_evidence.source_tree = "7".repeat(40);
        assert!(
            validate_opengauss_result_bindings(repository, &after, &[stale_evidence], &[]).is_err()
        );
        let mut foreign_check = check;
        foreign_check.repository_path = "/private/tmp/other-repository".into();
        assert!(
            validate_opengauss_result_bindings(repository, &after, &[], &[foreign_check]).is_err()
        );
    }

    #[test]
    fn native_dialog_values_escape_untrusted_control_text() {
        let encoded = dialog_value("claim\nDigest: fake\u{1b}").expect("JSON string");
        assert!(!encoded.contains('\n'));
        assert!(encoded.contains("\\n"));
        assert!(encoded.contains("\\u001b"));
    }

    #[test]
    fn decision_dialog_includes_escaped_reason_and_session_intent() {
        let request = DecisionRequestDto {
            proposal_id: "vpr_fixture".into(),
            entry_root: "sha256:fixture".into(),
            action: DecisionActionDto::Reject,
            reason: "bounded reason\nSigner: fake".into(),
            performer: "agent:fixture".into(),
            session_ref: Some("session\u{1b}fake".into()),
        };
        let (reason, session) = decision_dialog_intent(&request).expect("dialog values");
        assert_eq!(reason, "\"bounded reason\\nSigner: fake\"");
        assert_eq!(session, "\"session\\u001bfake\"");
    }
}
