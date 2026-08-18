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

export type OpenGaussToolDto = { path: string, version: string, sha256: string, size: number, probe_argv: Array<string>, probe_environment: Array<EnvironmentEntryDto>, trust_warning: string, };

export type OpenGaussProjectDto = { manifest_path: string, manifest_sha256: string, manifest_size: number, schema_version: number, name: string, kind: string, project_root: string, lean_root: string, source_mode: string, template_source_declared: boolean, blueprint_markers: Array<string>, configured_paths_validated: boolean, };

export type OpenGaussGitIdentityDto = { branch: string | null, commit: string, tree: string, dirty: boolean, changed_paths: number, };

export type OpenGaussHandoffPreviewDto = { repository_path: string, tool: OpenGaussToolDto, project: OpenGaussProjectDto, git_before: OpenGaussGitIdentityDto, cwd: string, interactive_argv: Array<string>, launcher_environment: Array<EnvironmentEntryDto>, documented_workflows: Array<string>, documented_entrypoint: string, backend_identity: string, hidden_transport_visible: boolean, upstream_source_commit: string, upstream_source_tree: string, authority_effect: string, boundary: string, };

export type OpenGaussSelectedEvidenceDto = { display_name: string, sha256: string, size: number, media_type: string, kind_hint: string, source_commit: string, source_tree: string, source: string, };

export type OpenGaussSelectedCheckDto = { run_id: string, profile: NativeExecProfileDto, state: NativeExecStateDto, exit_code: number | null, source_commit: string, source_tree: string, executable_sha256: string, stdout_sha256: string, stderr_sha256: string, producer_check_method: string, producer_check_outcome: string, };

export type OpenGaussHandoffReceiptDto = { preview: OpenGaussHandoffPreviewDto, terminal_owner: string, launched_at_unix_ms: number, git_after: OpenGaussGitIdentityDto | null, selected_evidence: Array<OpenGaussSelectedEvidenceDto>, selected_checks: Array<OpenGaussSelectedCheckDto>, result_boundary: string, };

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

export type VerificationMethodDto = { path: string, repository_relative_path: string, sha256: string, size: number, profile: string, property: string, reviewer_kind: string | null, reviewer_display_name: string | null, reviewer_identifier: string | null, provider: string | null, version: string | null, attested_by_actor_id: string | null, procedure: Array<string>, required_output: Array<string>, does_not_establish: Array<string>, };

export type VerificationDraftDto = { proposal_id: string, profile: string, method: VerificationMethodDto, property: string | null, complementary: boolean, outcome: string, does_not_establish: Array<string>, independent_of: Array<string>, shared_dependencies: Array<string>, output_paths: Array<string>, actor: string, };

export type VerificationFacetDto = { verification_record_id: string, verification_record_root: string, verifier: string, performer_kind: string | null, performer_identifier: string | null, provider: string | null, version: string | null, method_metadata_status: string, method_profile: string, method_path: string, environment_root: string, property: string, outcome: string, declared_independent_of: Array<string>, shared_dependencies: Array<string>, evidence_artifact_ids: Array<string>, output_artifact_ids: Array<string>, does_not_establish: Array<string>, protocol_evidence_role: string | null, satisfies_requirements: Array<string>, };

export type StandingStateDto = { repository_root: string, accepted_claim_ids: Array<string>, };

export type StandingDeltaDto = { transition: string, affected_claim_ids: Array<string>, before: StandingStateDto, if_accept: StandingStateDto, if_reject: StandingStateDto, global_accepted_before: number, global_accepted_if_accept: number, global_accepted_if_reject: number, };

export type DecisionBlockerDto = { code: string, detail: string, subject: string | null, };

export type DecisionEntryDto = { proposal_id: string, proposal_root: string, submission_root: string, claim_id: string, claim_root: string, repository_root: string, verification_set_root: string, entry_root: string, assertion: string, proposal_actor: string, proposal_action: string, proposal_reason: string, created_at: string, protocol_gate: string, blockers: Array<DecisionBlockerDto>, rejection_available: boolean, verification_requirements: Array<string>, verifications: Array<VerificationFacetDto>, limits: Array<string>, standing_delta: StandingDeltaDto, authority_keyset_root: string, policy_bundle_root: string, authority_record_root: string, authority_event_log_root: string, };

export type DecisionInboxDto = { repository_id: string, repository_root: string, projection_root: string, entries: Array<DecisionEntryDto>, observed_at_unix_ms: number, task: string, included_records: Array<string>, omissions: Array<string>, stale: boolean, refusal: StructuredVelaRefusalDto | null, };

export type VerificationPreviewDto = { draft: VerificationDraftDto, repository_path: string, source_commit: string, source_tree: string, repository_root: string, proposal_root: string, submission_root: string, claim_root: string, vela_binary_sha256: string, argv: Array<string>, selected_output_roots: Array<string>, authority_effect: string, warning: string, };

export type VerificationImportPreviewDto = { envelope_path: string, envelope_sha256: string, envelope_size: number, envelope_base64: string, verification_record_id: string, verification_record_root: string, verifier: string, proposal_id: string, proposal_root: string, submission_id: string, submission_root: string, claim_id: string, method_profile: string, method_path: string, environment_root: string, property: string, outcome: string, declared_independent_of: Array<string>, shared_dependencies: Array<string>, output_artifact_ids: Array<string>, does_not_establish: Array<string>, repository_path: string, current_repository_root: string, source_commit: string, source_tree: string, vela_binary_sha256: string, argv: Array<string>, authority_effect: string, warning: string, };

export type StructuredVelaRefusalDto = { kind: string, code: string | null, message: string, hint: string | null, command: string, operation_id: string | null, changed: boolean | null, next: string | null, };

export type VerificationResultDto = { operation_id: string | null, verification_record_id: string | null, verification_record_root: string | null, proposal_id: string, claim_id: string | null, outcome: string | null, idempotent: boolean | null, accepted_event_delta: number | null, publication_state: string | null, publication_commit: string | null, refusal: StructuredVelaRefusalDto | null, };

export type DecisionActionDto = "accept" | "reject";

export type DecisionRequestDto = { proposal_id: string, entry_root: string, action: DecisionActionDto, reason: string, performer: string, session_ref: string | null, };

export type DecisionPreviewDto = { request: DecisionRequestDto, repository_path: string, source_commit: string, source_tree: string, vela_binary_sha256: string, entry: DecisionEntryDto, performer_kind: string, repository_authority_principal: string, authentication: string, transaction_signer: string, ssh_agent_forwarded: boolean, argv: Array<string>, expected_successor: StandingStateDto, warning: string, };

export type DecisionReadbackDto = { status: string, standing: string | null, decision_actor: string | null, decision_actor_class: string | null, authority_principal_id: string | null, decision_event_id: string | null, applied_event_id: string | null, event_root: string | null, repository_root: string, replay_accepted_claims: number, replay_pending_claims: number, };

export type DecisionExecutionDto = { command_succeeded: boolean, decision_committed: boolean, successor_matches_preview: boolean, events_match_receipt: boolean | null, action: DecisionActionDto, proposal_id: string, entry_root: string, decision_plan_root: string | null, event_ids: Array<string>, authority_record_id: string | null, actual_performer: string | null, actual_performer_kind: string | null, actual_authority_principal: string | null, authentication: string | null, transaction_signer: string | null, scientific_state_changed: boolean | null, refusal: StructuredVelaRefusalDto | null, readback: DecisionReadbackDto, };

export type RecoveryPreviewDto = { repository_path: string, operation_id: string, source_commit: string, source_tree: string, vela_binary_sha256: string, argv: Array<string>, warning: string, };

export type RecoveryResultDto = { operation_id: string, outcome: string | null, repository_blocked_after: boolean | null, continuation_status: string | null, next_command: string | null, refusal: StructuredVelaRefusalDto | null, };

export type CommandErrorDto = { kind: string, message: string, detail: string | null, };
