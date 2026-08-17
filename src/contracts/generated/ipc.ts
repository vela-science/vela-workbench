// Generated from src-tauri/src/contracts/mod.rs. Do not edit by hand.

export type PreferencesDto = { recent_repositories: Array<string>, vela_binary_path: string | null, };

export type RuntimePolicyDto = { interface_commit: string, interface_tree: string, runtime_version: string, runtime_commit: string, runtime_sha256: string, read_only: boolean, };

export type BootstrapDto = { preferences: PreferencesDto, runtime: RuntimePolicyDto, };

export type RepositoryClassificationDto = "vela_repository" | "native_integration" | "git_only";

export type VelaBinaryStateDto = "signed_runtime_baseline" | "version_match_hash_unrecognized" | "unsupported";

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

export type CommandErrorDto = { kind: string, message: string, detail: string | null, };
