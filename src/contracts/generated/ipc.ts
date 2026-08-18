// Generated from src-tauri/src/contracts/mod.rs. Do not edit by hand.

export type PreferencesDto = { recent_repositories: Array<string>, vela_binary_path: string | null, };

export type RuntimePolicyDto = { interface_commit: string, interface_tree: string, runtime_version: string, runtime_commit: string, runtime_sha256: string, read_only: boolean, tranche: string, mutation_scope: string, tranche_three_enabled: boolean, };

export type BootstrapDto = { preferences: PreferencesDto, runtime: RuntimePolicyDto, };

export type RepositoryClassificationDto = "vela_repository" | "native_integration" | "git_only";

export type VelaBinaryStateDto = "signed_runtime_baseline" | "unsupported";

export type VelaBinaryDto = { path: string, version: string, sha256: string, state: VelaBinaryStateDto, };

export type GitRemoteDto = { name: string, url: string, operation: string, };

export type GitWorktreeDto = { path: string, head: string | null, branch: string | null, detached: boolean, locked: boolean, prunable: boolean, };

export type EntireCheckpointDto = { commit: string, checkpoint_id: string, };

export type GitSnapshotDto = { root: string, branch: string | null, detached: boolean, head_commit: string, head_tree: string, upstream: string | null, ahead: number, behind: number, dirty: boolean, conflicted: boolean, changed_paths: number, worktrees: Array<GitWorktreeDto>, remotes: Array<GitRemoteDto>, entire_checkpoints: Array<EntireCheckpointDto>, };

export type StatusCountsDto = { claims: number, accepted_claims: number, pending_claims: number, pending_review: number, accepted_review: number, rejected_review: number, withdrawn_review: number, submissions: number, verifications: number, artifacts: number, };

export type VelaStatusDto = { repository_id: string, repository_name: string, repository_profile_root: string, repository_root: string | null, origin_root: string | null, authority_keyset_root: string | null, authority_policy_root: string | null, repository_commit: string | null, repository_tree: string | null, replay: string, strict: string, blocker_count: number, counts: StatusCountsDto, inbox_pending: number, inbox_projection_root: string | null, work_mode: string, work_command: string, work_note: string, };

export type ClaimDto = { claim_id: string, claim_root: string, standing: string, origin_era: string, readable: boolean, assertion_kind: string | null, assertion: string | null, unreadable_reason: string | null, created_at: string | null, revision: number | null, };

export type IntegrationItemDto = { kind: string, id: string, path: string, root: string, };

export type IntegrationDto = { authority_effect: string, manifest_root: string, documents_checked: number, repository: string, revision: string, profiles: Array<IntegrationItemDto>, bindings: Array<IntegrationItemDto>, methods: Array<IntegrationItemDto>, does_not_establish: Array<string>, };

export type RefusalDto = { area: string, kind: string, code: string | null, message: string, hint: string | null, command: string, };

export type VelaInspectionDto = { binary: VelaBinaryDto | null, status: VelaStatusDto | null, claims: Array<ClaimDto>, integration: IntegrationDto | null, refusal: RefusalDto | null, };

export type EntireAvailabilityDto = { cli_available: boolean, checkpoint_reference_count: number, note: string, };

export type RepositorySnapshotDto = { path: string, name: string, observed_at_unix_ms: number, classification: RepositoryClassificationDto, classification_basis: string, git: GitSnapshotDto, vela: VelaInspectionDto, entire: EntireAvailabilityDto, reviewed_problem_url: string | null, };

export type LaunchKindDto = "terminal" | "cursor" | "visual_studio_code" | "forge";

export type LaunchResultDto = { target: string, owner: string, };

export type WorktreePreviewDto = { repository_path: string, source_head: string, source_tree: string, target_ref: string, target_commit: string, destination: string, command: Array<string>, rollback: Array<string>, warning: string, };

export type WorktreeResultDto = { destination: string, target_commit: string, rollback: Array<string>, repository: RepositorySnapshotDto, };

export type NativeExecProfileDto = "git_diff_check" | "lean_build" | "cargo_test" | "bun_test";

export type NativeToolDto = { profile: NativeExecProfileDto, path: string, sha256: string, size: number, };

export type EnvironmentEntryDto = { name: string, value: string, };

export type NativeExecPreviewDto = { profile: NativeExecProfileDto, label: string, repository_path: string, source_commit: string, source_tree: string, executable: NativeToolDto, argv: Array<string>, working_directory: string, environment: Array<EnvironmentEntryDto>, timeout_ms: number, max_stdout_bytes: number, max_stderr_bytes: number, trust_warning: string, sandboxed: boolean, };

export type NativeExecStateDto = "completed" | "failed" | "cancelled" | "timed_out";

export type NativeOutputDto = { stream: string, sha256: string, size: number, content_base64: string, content_utf8: string | null, truncated: boolean, };

export type NativeExecResultDto = { run_id: string, profile: NativeExecProfileDto, state: NativeExecStateDto, exit_code: number | null, started_at_unix_ms: number, completed_at_unix_ms: number, source_commit: string, source_tree: string, executable_sha256: string, stdout: NativeOutputDto, stderr: NativeOutputDto, producer_check_method: string, producer_check_outcome: string, };

export type CancelResultDto = { run_id: string, cancellation_requested: boolean, };

export type EvidenceSourceDto = { "source": "local_file", path: string, repository_relative_path: string, } | { "source": "command_output", run_id: string, stream: string, };

export type EvidenceItemDto = { source: EvidenceSourceDto, display_name: string, sha256: string, size: number, media_type: string, kind_hint: string, source_commit: string, source_tree: string, source_dirty: boolean, content_base64: string, content_utf8: string | null, private: boolean, };

export type EvidenceExportRequestDto = { source: EvidenceSourceDto, expected_sha256: string, exclusions: Array<string>, redaction_confirmed: boolean, derived_utf8: string | null, };

export type EvidenceExportPreviewDto = { request: EvidenceExportRequestDto, destination: string, source_sha256: string, source_size: number, output_sha256: string, output_size: number, derived: boolean, exclusions: Array<string>, redaction_confirmed: boolean, output_base64: string, output_utf8: string | null, warning: string, };

export type EvidenceExportResultDto = { destination: string, sha256: string, size: number, derived: boolean, source_unchanged: boolean, };

export type SubmissionArtifactDraftDto = { path: string, kind: string, sha256: string, size: number, };

export type SubmissionDraftDto = { assertion: string, claim_type: string, conditions: Array<string>, replayability: string, artifacts: Array<SubmissionArtifactDraftDto>, caveats: Array<string>, producer_check_run_ids: Array<string>, verification_requirements: Array<string>, source_run: string | null, producer: string, };

export type SubmissionPreviewDto = { draft: SubmissionDraftDto, repository_path: string, source_commit: string, source_tree: string, vela_binary_sha256: string, argv: Array<string>, artifact_total_bytes: number, producer_checks: Array<string>, authority_boundary: string, warning: string, };

export type SubmissionImportPreviewDto = { envelope_path: string, envelope_sha256: string, envelope_size: number, envelope_base64: string, payload_type: string, producer: string, assertion: string, claim_type: string, artifacts: Array<SubmissionArtifactDraftDto>, repository_path: string, source_commit: string, source_tree: string, vela_binary_sha256: string, authority_boundary: string, warning: string, };

export type SubmissionResultDto = { operation_id: string, submission_id: string, submission_root: string, proposal_id: string, proposal_root: string, claim_id: string, route: string, accepted_event_delta: number, accepted_state_changed: boolean, publication_state: string, publication_commit: string | null, };

export type CommandErrorDto = { kind: string, message: string, detail: string | null, };
