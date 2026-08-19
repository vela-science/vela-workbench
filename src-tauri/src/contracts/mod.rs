mod wire;

pub(crate) use wire::{
    ClaimsV1Wire, ErrorEnvelopeWire, IntegrationCheckV1Wire, IntegrationInspectionV1Wire,
    IntegrationItemWire, PublicationWire, RecoveryInspectionV1Wire, StatusV4Wire,
    SubmitResultV1Wire,
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
pub struct ProblemHandoffDto {
    pub schema: String,
    pub handoff_url: String,
    pub problem_url: String,
    pub source_repository_url: String,
    pub source_revision: String,
    pub authority_repository_url: String,
    pub artifact_paths: Vec<String>,
    pub authority_effect: String,
    pub boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct ProblemHandoffSourceDto {
    pub repository_path: String,
    pub source_repository_url: String,
    pub source_revision: String,
    pub selected_head: String,
    pub remote_matches: bool,
    pub revision_matches: bool,
    pub ready: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct ProblemHandoffAuthorityDto {
    pub repository_path: String,
    pub authority_repository_url: String,
    pub repository_id: Option<String>,
    pub remote_matches: bool,
    pub vela_repository: bool,
    pub ready: bool,
    pub note: String,
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
    pub operation_id: Option<String>,
    pub changed: Option<bool>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct VelaInspectionDto {
    pub binary: Option<VelaBinaryDto>,
    pub status: Option<VelaStatusDto>,
    pub claims: Vec<ClaimDto>,
    pub integration: Option<IntegrationDto>,
    pub refusal: Option<RefusalDto>,
    pub recovery_operation_id: Option<String>,
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
pub struct OpenGaussToolDto {
    pub path: String,
    pub version: String,
    pub sha256: String,
    pub size: u64,
    pub probe_argv: Vec<String>,
    pub probe_environment: Vec<EnvironmentEntryDto>,
    pub trust_warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct OpenGaussProjectDto {
    pub manifest_path: String,
    pub manifest_sha256: String,
    pub manifest_size: u64,
    pub schema_version: u64,
    pub name: String,
    pub kind: String,
    pub project_root: String,
    pub lean_root: String,
    pub source_mode: String,
    pub template_source_declared: bool,
    pub blueprint_markers: Vec<String>,
    pub configured_paths_validated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct OpenGaussGitIdentityDto {
    pub branch: Option<String>,
    pub commit: String,
    pub tree: String,
    pub dirty: bool,
    pub changed_paths: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct OpenGaussHandoffPreviewDto {
    pub repository_path: String,
    pub tool: OpenGaussToolDto,
    pub project: OpenGaussProjectDto,
    pub git_before: OpenGaussGitIdentityDto,
    pub cwd: String,
    pub interactive_argv: Vec<String>,
    pub launcher_environment: Vec<EnvironmentEntryDto>,
    pub documented_workflows: Vec<String>,
    pub documented_entrypoint: String,
    pub backend_identity: String,
    pub hidden_transport_visible: bool,
    pub upstream_source_commit: String,
    pub upstream_source_tree: String,
    pub authority_effect: String,
    pub boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct OpenGaussSelectedEvidenceDto {
    pub display_name: String,
    pub sha256: String,
    pub size: u64,
    pub media_type: String,
    pub kind_hint: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct OpenGaussSelectedCheckDto {
    pub run_id: String,
    pub repository_path: String,
    pub profile: NativeExecProfileDto,
    pub state: NativeExecStateDto,
    pub exit_code: Option<i32>,
    pub source_commit: String,
    pub source_tree: String,
    pub executable_path: String,
    pub executable_sha256: String,
    pub argv: Vec<String>,
    pub working_directory: String,
    pub environment: Vec<EnvironmentEntryDto>,
    pub timeout_ms: u64,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub producer_check_method: String,
    pub producer_check_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct OpenGaussHandoffReceiptDto {
    pub preview: OpenGaussHandoffPreviewDto,
    pub terminal_owner: String,
    pub launched_at_unix_ms: u64,
    pub git_after: Option<OpenGaussGitIdentityDto>,
    pub selected_evidence: Vec<OpenGaussSelectedEvidenceDto>,
    pub selected_checks: Vec<OpenGaussSelectedCheckDto>,
    pub result_boundary: String,
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
pub struct VerificationMethodDto {
    pub path: String,
    pub repository_relative_path: String,
    pub sha256: String,
    pub size: u64,
    pub profile: String,
    pub property: String,
    pub reviewer_kind: Option<String>,
    pub reviewer_display_name: Option<String>,
    pub reviewer_identifier: Option<String>,
    pub provider: Option<String>,
    pub version: Option<String>,
    pub attested_by_actor_id: Option<String>,
    pub procedure: Vec<String>,
    pub required_output: Vec<String>,
    pub does_not_establish: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct VerificationDraftDto {
    pub proposal_id: String,
    pub profile: String,
    pub method: VerificationMethodDto,
    pub property: Option<String>,
    pub complementary: bool,
    pub outcome: String,
    pub does_not_establish: Vec<String>,
    pub independent_of: Vec<String>,
    pub shared_dependencies: Vec<String>,
    pub output_paths: Vec<String>,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct VerificationFacetDto {
    pub verification_record_id: String,
    pub verification_record_root: String,
    pub verifier: String,
    pub performer_kind: Option<String>,
    pub performer_identifier: Option<String>,
    pub provider: Option<String>,
    pub version: Option<String>,
    pub method_metadata_status: String,
    pub method_profile: String,
    pub method_path: String,
    pub environment_root: String,
    pub property: String,
    pub outcome: String,
    pub declared_independent_of: Vec<String>,
    pub shared_dependencies: Vec<String>,
    pub evidence_artifact_ids: Vec<String>,
    pub output_artifact_ids: Vec<String>,
    pub does_not_establish: Vec<String>,
    pub protocol_evidence_role: Option<String>,
    pub satisfies_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct StandingStateDto {
    pub repository_root: String,
    pub accepted_claim_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct StandingDeltaDto {
    pub transition: String,
    pub affected_claim_ids: Vec<String>,
    pub before: StandingStateDto,
    pub if_accept: StandingStateDto,
    pub if_reject: StandingStateDto,
    pub global_accepted_before: u64,
    pub global_accepted_if_accept: u64,
    pub global_accepted_if_reject: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct DecisionBlockerDto {
    pub code: String,
    pub detail: String,
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct DecisionEntryDto {
    pub proposal_id: String,
    pub proposal_root: String,
    pub submission_root: String,
    pub claim_id: String,
    pub claim_root: String,
    pub repository_root: String,
    pub verification_set_root: String,
    pub entry_root: String,
    pub assertion: String,
    pub proposal_actor: String,
    pub proposal_action: String,
    pub proposal_reason: String,
    pub created_at: String,
    pub protocol_gate: String,
    pub blockers: Vec<DecisionBlockerDto>,
    pub rejection_available: bool,
    pub verification_requirements: Vec<String>,
    pub verifications: Vec<VerificationFacetDto>,
    pub limits: Vec<String>,
    pub standing_delta: StandingDeltaDto,
    pub authority_keyset_root: String,
    pub policy_bundle_root: String,
    pub authority_record_root: String,
    pub authority_event_log_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct DecisionInboxDto {
    pub repository_id: String,
    pub repository_root: String,
    pub projection_root: String,
    pub entries: Vec<DecisionEntryDto>,
    pub observed_at_unix_ms: u64,
    pub task: String,
    pub included_records: Vec<String>,
    pub omissions: Vec<String>,
    pub stale: bool,
    pub refusal: Option<StructuredVelaRefusalDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct VerificationPreviewDto {
    pub draft: VerificationDraftDto,
    pub repository_path: String,
    pub source_commit: String,
    pub source_tree: String,
    pub repository_root: String,
    pub proposal_root: String,
    pub submission_root: String,
    pub claim_root: String,
    pub vela_binary_sha256: String,
    pub argv: Vec<String>,
    pub selected_output_roots: Vec<String>,
    pub authority_effect: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct VerificationImportPreviewDto {
    pub envelope_path: String,
    pub envelope_sha256: String,
    pub envelope_size: u64,
    pub envelope_base64: String,
    pub verification_record_id: String,
    pub verification_record_root: String,
    pub verifier: String,
    pub proposal_id: String,
    pub proposal_root: String,
    pub submission_id: String,
    pub submission_root: String,
    pub claim_id: String,
    pub method_profile: String,
    pub method_path: String,
    pub environment_root: String,
    pub property: String,
    pub outcome: String,
    pub declared_independent_of: Vec<String>,
    pub shared_dependencies: Vec<String>,
    pub output_artifact_ids: Vec<String>,
    pub does_not_establish: Vec<String>,
    pub repository_path: String,
    pub current_repository_root: String,
    pub source_commit: String,
    pub source_tree: String,
    pub vela_binary_sha256: String,
    pub argv: Vec<String>,
    pub authority_effect: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct StructuredVelaRefusalDto {
    pub kind: String,
    pub code: Option<String>,
    pub message: String,
    pub hint: Option<String>,
    pub command: String,
    pub operation_id: Option<String>,
    pub changed: Option<bool>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct VerificationResultDto {
    pub operation_id: Option<String>,
    pub verification_record_id: Option<String>,
    pub verification_record_root: Option<String>,
    pub proposal_id: String,
    pub claim_id: Option<String>,
    pub outcome: Option<String>,
    pub idempotent: Option<bool>,
    pub accepted_event_delta: Option<u64>,
    pub publication_state: Option<String>,
    pub publication_commit: Option<String>,
    pub refusal: Option<StructuredVelaRefusalDto>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionActionDto {
    Accept,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct DecisionRequestDto {
    pub proposal_id: String,
    pub entry_root: String,
    pub action: DecisionActionDto,
    pub reason: String,
    pub performer: String,
    pub session_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct DecisionPreviewDto {
    pub request: DecisionRequestDto,
    pub repository_path: String,
    pub source_commit: String,
    pub source_tree: String,
    pub vela_binary_sha256: String,
    pub entry: DecisionEntryDto,
    pub performer_kind: String,
    pub repository_authority_principal: String,
    pub authentication: String,
    pub transaction_signer: String,
    pub ssh_agent_forwarded: bool,
    pub argv: Vec<String>,
    pub expected_successor: StandingStateDto,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct DecisionReadbackDto {
    pub status: String,
    pub standing: Option<String>,
    pub decision_actor: Option<String>,
    pub decision_actor_class: Option<String>,
    pub authority_principal_id: Option<String>,
    pub decision_event_id: Option<String>,
    pub applied_event_id: Option<String>,
    pub event_root: Option<String>,
    pub repository_root: String,
    pub replay_accepted_claims: u64,
    pub replay_pending_claims: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct DecisionExecutionDto {
    pub command_succeeded: bool,
    pub decision_committed: bool,
    pub successor_matches_preview: bool,
    pub events_match_receipt: Option<bool>,
    pub action: DecisionActionDto,
    pub proposal_id: String,
    pub entry_root: String,
    pub decision_plan_root: Option<String>,
    pub event_ids: Vec<String>,
    pub authority_record_id: Option<String>,
    pub actual_performer: Option<String>,
    pub actual_performer_kind: Option<String>,
    pub actual_authority_principal: Option<String>,
    pub authentication: Option<String>,
    pub transaction_signer: Option<String>,
    pub scientific_state_changed: Option<bool>,
    pub refusal: Option<StructuredVelaRefusalDto>,
    pub readback: DecisionReadbackDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct RecoveryPreviewDto {
    pub repository_path: String,
    pub operation_id: String,
    pub source_commit: String,
    pub source_tree: String,
    pub vela_binary_sha256: String,
    pub argv: Vec<String>,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct RecoveryResultDto {
    pub operation_id: String,
    pub outcome: Option<String>,
    pub repository_blocked_after: Option<bool>,
    pub continuation_status: Option<String>,
    pub next_command: Option<String>,
    pub refusal: Option<StructuredVelaRefusalDto>,
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
        ProblemHandoffDto::decl(&config),
        ProblemHandoffSourceDto::decl(&config),
        ProblemHandoffAuthorityDto::decl(&config),
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
        OpenGaussToolDto::decl(&config),
        OpenGaussProjectDto::decl(&config),
        OpenGaussGitIdentityDto::decl(&config),
        OpenGaussHandoffPreviewDto::decl(&config),
        OpenGaussSelectedEvidenceDto::decl(&config),
        OpenGaussSelectedCheckDto::decl(&config),
        OpenGaussHandoffReceiptDto::decl(&config),
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
        VerificationMethodDto::decl(&config),
        VerificationDraftDto::decl(&config),
        VerificationFacetDto::decl(&config),
        StandingStateDto::decl(&config),
        StandingDeltaDto::decl(&config),
        DecisionBlockerDto::decl(&config),
        DecisionEntryDto::decl(&config),
        DecisionInboxDto::decl(&config),
        VerificationPreviewDto::decl(&config),
        VerificationImportPreviewDto::decl(&config),
        StructuredVelaRefusalDto::decl(&config),
        VerificationResultDto::decl(&config),
        DecisionActionDto::decl(&config),
        DecisionRequestDto::decl(&config),
        DecisionPreviewDto::decl(&config),
        DecisionReadbackDto::decl(&config),
        DecisionExecutionDto::decl(&config),
        RecoveryPreviewDto::decl(&config),
        RecoveryResultDto::decl(&config),
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
    fn product_capability_is_closed_and_contains_only_reviewed_authority_paths() {
        let permission = include_str!("../../permissions/workbench.toml");
        for forbidden in ["shell", "http", "upload", "provider", "generic", "session"] {
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
        for required in [
            "review_problem_handoff",
            "review_problem_handoff_source",
            "review_problem_handoff_authority",
            "open_problem_handoff",
            "refresh_decision_inbox",
            "preview_verification_record",
            "record_verification",
            "preview_decision",
            "execute_decision",
            "preview_recovery",
            "recover_transaction",
            "select_opengauss",
            "launch_opengauss_handoff",
            "refresh_opengauss_handoff",
        ] {
            assert!(permission.contains(required), "missing {required}");
        }
        assert_eq!(
            permission
                .lines()
                .filter(|line| line.trim_start().starts_with('"'))
                .count(),
            36
        );
    }
}
