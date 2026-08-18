mod wire;

pub(crate) use wire::{
    ClaimsV1Wire, ErrorEnvelopeWire, IntegrationCheckV1Wire, IntegrationInspectionV1Wire,
    IntegrationItemWire, PublicationWire, StatusV4Wire, SubmitResultV1Wire,
};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct PreferencesDto {
    pub recent_repositories: Vec<String>,
    pub vela_binary_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct RuntimePolicyDto {
    pub interface_commit: String,
    pub interface_tree: String,
    pub runtime_version: String,
    pub runtime_commit: String,
    pub runtime_sha256: String,
    pub read_only: bool,
    pub tranche: String,
    pub mutation_scope: String,
    pub tranche_three_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct BootstrapDto {
    pub preferences: PreferencesDto,
    pub runtime: RuntimePolicyDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryClassificationDto {
    VelaRepository,
    NativeIntegration,
    GitOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VelaBinaryStateDto {
    SignedRuntimeBaseline,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct VelaBinaryDto {
    pub path: String,
    pub version: String,
    pub sha256: String,
    pub state: VelaBinaryStateDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct GitRemoteDto {
    pub name: String,
    pub url: String,
    pub operation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct GitWorktreeDto {
    pub path: String,
    pub head: Option<String>,
    pub branch: Option<String>,
    pub detached: bool,
    pub locked: bool,
    pub prunable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct EntireCheckpointDto {
    pub commit: String,
    pub checkpoint_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct GitSnapshotDto {
    pub root: String,
    pub branch: Option<String>,
    pub detached: bool,
    pub head_commit: String,
    pub head_tree: String,
    pub upstream: Option<String>,
    pub ahead: u64,
    pub behind: u64,
    pub dirty: bool,
    pub conflicted: bool,
    pub changed_paths: u64,
    pub worktrees: Vec<GitWorktreeDto>,
    pub remotes: Vec<GitRemoteDto>,
    pub entire_checkpoints: Vec<EntireCheckpointDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct StatusCountsDto {
    pub claims: u64,
    pub accepted_claims: u64,
    pub pending_claims: u64,
    pub pending_review: u64,
    pub accepted_review: u64,
    pub rejected_review: u64,
    pub withdrawn_review: u64,
    pub submissions: u64,
    pub verifications: u64,
    pub artifacts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct VelaStatusDto {
    pub repository_id: String,
    pub repository_name: String,
    pub repository_profile_root: String,
    pub repository_root: Option<String>,
    pub origin_root: Option<String>,
    pub authority_keyset_root: Option<String>,
    pub authority_policy_root: Option<String>,
    pub repository_commit: Option<String>,
    pub repository_tree: Option<String>,
    pub replay: String,
    pub strict: String,
    pub blocker_count: u64,
    pub counts: StatusCountsDto,
    pub inbox_pending: u64,
    pub inbox_projection_root: Option<String>,
    pub work_mode: String,
    pub work_command: String,
    pub work_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct ClaimDto {
    pub claim_id: String,
    pub claim_root: String,
    pub standing: String,
    pub origin_era: String,
    pub readable: bool,
    pub assertion_kind: Option<String>,
    pub assertion: Option<String>,
    pub unreadable_reason: Option<String>,
    pub created_at: Option<String>,
    pub revision: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct IntegrationItemDto {
    pub kind: String,
    pub id: String,
    pub path: String,
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct IntegrationDto {
    pub authority_effect: String,
    pub manifest_root: String,
    pub documents_checked: u64,
    pub repository: String,
    pub revision: String,
    pub profiles: Vec<IntegrationItemDto>,
    pub bindings: Vec<IntegrationItemDto>,
    pub methods: Vec<IntegrationItemDto>,
    pub does_not_establish: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct RefusalDto {
    pub area: String,
    pub kind: String,
    pub code: Option<String>,
    pub message: String,
    pub hint: Option<String>,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct VelaInspectionDto {
    pub binary: Option<VelaBinaryDto>,
    pub status: Option<VelaStatusDto>,
    pub claims: Vec<ClaimDto>,
    pub integration: Option<IntegrationDto>,
    pub refusal: Option<RefusalDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct EntireAvailabilityDto {
    pub cli_available: bool,
    pub checkpoint_reference_count: u64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct RepositorySnapshotDto {
    pub path: String,
    pub name: String,
    pub observed_at_unix_ms: u64,
    pub classification: RepositoryClassificationDto,
    pub classification_basis: String,
    pub git: GitSnapshotDto,
    pub vela: VelaInspectionDto,
    pub entire: EntireAvailabilityDto,
    pub reviewed_problem_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchKindDto {
    Terminal,
    Cursor,
    VisualStudioCode,
    Forge,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct LaunchResultDto {
    pub target: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct WorktreePreviewDto {
    pub repository_path: String,
    pub source_head: String,
    pub source_tree: String,
    pub target_ref: String,
    pub target_commit: String,
    pub destination: String,
    pub command: Vec<String>,
    pub rollback: Vec<String>,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct WorktreeResultDto {
    pub destination: String,
    pub target_commit: String,
    pub rollback: Vec<String>,
    pub repository: RepositorySnapshotDto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NativeExecProfileDto {
    GitDiffCheck,
    LeanBuild,
    CargoTest,
    BunTest,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct NativeToolDto {
    pub profile: NativeExecProfileDto,
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct EnvironmentEntryDto {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct NativeExecPreviewDto {
    pub profile: NativeExecProfileDto,
    pub label: String,
    pub repository_path: String,
    pub source_commit: String,
    pub source_tree: String,
    pub executable: NativeToolDto,
    pub argv: Vec<String>,
    pub working_directory: String,
    pub environment: Vec<EnvironmentEntryDto>,
    pub timeout_ms: u64,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
    pub trust_warning: String,
    pub sandboxed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativeExecStateDto {
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct NativeOutputDto {
    pub stream: String,
    pub sha256: String,
    pub size: u64,
    pub content_base64: String,
    pub content_utf8: Option<String>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct NativeExecResultDto {
    pub run_id: String,
    pub profile: NativeExecProfileDto,
    pub state: NativeExecStateDto,
    pub exit_code: Option<i32>,
    pub started_at_unix_ms: u64,
    pub completed_at_unix_ms: u64,
    pub source_commit: String,
    pub source_tree: String,
    pub executable_sha256: String,
    pub stdout: NativeOutputDto,
    pub stderr: NativeOutputDto,
    pub producer_check_method: String,
    pub producer_check_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct CancelResultDto {
    pub run_id: String,
    pub cancellation_requested: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum EvidenceSourceDto {
    LocalFile {
        path: String,
        repository_relative_path: String,
    },
    CommandOutput {
        run_id: String,
        stream: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct EvidenceItemDto {
    pub source: EvidenceSourceDto,
    pub display_name: String,
    pub sha256: String,
    pub size: u64,
    pub media_type: String,
    pub kind_hint: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_dirty: bool,
    pub content_base64: String,
    pub content_utf8: Option<String>,
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct EvidenceExportRequestDto {
    pub source: EvidenceSourceDto,
    pub expected_sha256: String,
    pub exclusions: Vec<String>,
    pub redaction_confirmed: bool,
    pub derived_utf8: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct EvidenceExportPreviewDto {
    pub request: EvidenceExportRequestDto,
    pub destination: String,
    pub source_sha256: String,
    pub source_size: u64,
    pub output_sha256: String,
    pub output_size: u64,
    pub derived: bool,
    pub exclusions: Vec<String>,
    pub redaction_confirmed: bool,
    pub output_base64: String,
    pub output_utf8: Option<String>,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct EvidenceExportResultDto {
    pub destination: String,
    pub sha256: String,
    pub size: u64,
    pub derived: bool,
    pub source_unchanged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct SubmissionArtifactDraftDto {
    pub path: String,
    pub kind: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct SubmissionDraftDto {
    pub assertion: String,
    pub claim_type: String,
    pub conditions: Vec<String>,
    pub replayability: String,
    pub artifacts: Vec<SubmissionArtifactDraftDto>,
    pub caveats: Vec<String>,
    pub producer_check_run_ids: Vec<String>,
    pub verification_requirements: Vec<String>,
    pub source_run: Option<String>,
    pub producer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct SubmissionPreviewDto {
    pub draft: SubmissionDraftDto,
    pub repository_path: String,
    pub source_commit: String,
    pub source_tree: String,
    pub vela_binary_sha256: String,
    pub argv: Vec<String>,
    pub artifact_total_bytes: u64,
    pub producer_checks: Vec<String>,
    pub authority_boundary: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct SubmissionImportPreviewDto {
    pub envelope_path: String,
    pub envelope_sha256: String,
    pub envelope_size: u64,
    pub envelope_base64: String,
    pub payload_type: String,
    pub producer: String,
    pub assertion: String,
    pub claim_type: String,
    pub artifacts: Vec<SubmissionArtifactDraftDto>,
    pub repository_path: String,
    pub source_commit: String,
    pub source_tree: String,
    pub vela_binary_sha256: String,
    pub authority_boundary: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct SubmissionResultDto {
    pub operation_id: String,
    pub submission_id: String,
    pub submission_root: String,
    pub proposal_id: String,
    pub proposal_root: String,
    pub claim_id: String,
    pub route: String,
    pub accepted_event_delta: u64,
    pub accepted_state_changed: bool,
    pub publication_state: String,
    pub publication_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct CommandErrorDto {
    pub kind: String,
    pub message: String,
    pub detail: Option<String>,
}

impl CommandErrorDto {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Generate the only renderer-facing contract file from the Rust DTO source.
pub fn typescript_bindings() -> String {
    // Tauri serializes these bounded counters as JSON numbers, not JavaScript
    // bigint values. Keep the generated transport contract faithful to IPC.
    let config = ts_rs::Config::default().with_large_int("number");
    let declarations = [
        PreferencesDto::decl(&config),
        RuntimePolicyDto::decl(&config),
        BootstrapDto::decl(&config),
        RepositoryClassificationDto::decl(&config),
        VelaBinaryStateDto::decl(&config),
        VelaBinaryDto::decl(&config),
        GitRemoteDto::decl(&config),
        GitWorktreeDto::decl(&config),
        EntireCheckpointDto::decl(&config),
        GitSnapshotDto::decl(&config),
        StatusCountsDto::decl(&config),
        VelaStatusDto::decl(&config),
        ClaimDto::decl(&config),
        IntegrationItemDto::decl(&config),
        IntegrationDto::decl(&config),
        RefusalDto::decl(&config),
        VelaInspectionDto::decl(&config),
        EntireAvailabilityDto::decl(&config),
        RepositorySnapshotDto::decl(&config),
        LaunchKindDto::decl(&config),
        LaunchResultDto::decl(&config),
        WorktreePreviewDto::decl(&config),
        WorktreeResultDto::decl(&config),
        NativeExecProfileDto::decl(&config),
        NativeToolDto::decl(&config),
        EnvironmentEntryDto::decl(&config),
        NativeExecPreviewDto::decl(&config),
        NativeExecStateDto::decl(&config),
        NativeOutputDto::decl(&config),
        NativeExecResultDto::decl(&config),
        CancelResultDto::decl(&config),
        EvidenceSourceDto::decl(&config),
        EvidenceItemDto::decl(&config),
        EvidenceExportRequestDto::decl(&config),
        EvidenceExportPreviewDto::decl(&config),
        EvidenceExportResultDto::decl(&config),
        SubmissionArtifactDraftDto::decl(&config),
        SubmissionDraftDto::decl(&config),
        SubmissionPreviewDto::decl(&config),
        SubmissionImportPreviewDto::decl(&config),
        SubmissionResultDto::decl(&config),
        CommandErrorDto::decl(&config),
    ];
    let mut output =
        String::from("// Generated from src-tauri/src/contracts/mod.rs. Do not edit by hand.\n\n");
    for declaration in declarations {
        output.push_str("export ");
        output.push_str(&declaration);
        output.push_str("\n\n");
    }
    output.pop();
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn checked_in_renderer_contract_matches_rust_source() {
        let checked_in = include_str!("../../../src/contracts/generated/ipc.ts");
        assert_eq!(checked_in, super::typescript_bindings());
    }

    #[test]
    fn tranche_two_capability_exposes_no_generic_or_authority_command() {
        let permission = include_str!("../../permissions/workbench.toml");
        for forbidden in [
            "shell",
            "http",
            "upload",
            "verification",
            "accept",
            "reject",
            "decision",
            "generic",
        ] {
            assert!(
                !permission.lines().any(|line| {
                    line.trim_start().starts_with('"')
                        && line.to_ascii_lowercase().contains(forbidden)
                }),
                "forbidden capability command: {forbidden}"
            );
        }
        assert!(permission.contains("preview_submission_draft"));
        assert!(permission.contains("import_submission"));
    }
}
