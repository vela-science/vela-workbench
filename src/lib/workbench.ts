import { invoke } from "@tauri-apps/api/core";
import type {
  BootstrapDto,
  CancelResultDto,
  EvidenceExportPreviewDto,
  EvidenceExportRequestDto,
  EvidenceExportResultDto,
  EvidenceItemDto,
  LaunchKindDto,
  LaunchResultDto,
  NativeExecPreviewDto,
  NativeExecProfileDto,
  NativeExecResultDto,
  NativeToolDto,
  PreferencesDto,
  RepositorySnapshotDto,
  SubmissionDraftDto,
  SubmissionImportPreviewDto,
  SubmissionPreviewDto,
  SubmissionResultDto,
  VelaBinaryDto,
  WorktreePreviewDto,
  WorktreeResultDto,
} from "../contracts/generated/ipc";

export const workbench = {
  bootstrap: () => invoke<BootstrapDto>("bootstrap"),
  selectRepository: () => invoke<RepositorySnapshotDto | null>("select_repository"),
  inspectRepository: (path: string) => invoke<RepositorySnapshotDto>("inspect_repository", { path }),
  selectVelaBinary: () => invoke<VelaBinaryDto | null>("select_vela_binary"),
  clearRecents: () => invoke<PreferencesDto>("clear_recents"),
  launchRepository: (path: string, kind: LaunchKindDto) =>
    invoke<LaunchResultDto>("launch_repository", { path, kind }),
  previewWorktree: (path: string, targetRef: string) =>
    invoke<WorktreePreviewDto | null>("preview_worktree", { path, targetRef }),
  createWorktree: (preview: WorktreePreviewDto) =>
    invoke<WorktreeResultDto | null>("create_worktree", { preview }),
  selectNativeTool: (profile: NativeExecProfileDto) =>
    invoke<NativeToolDto | null>("select_native_tool", { profile }),
  previewNativeExec: (path: string, profile: NativeExecProfileDto) =>
    invoke<NativeExecPreviewDto>("preview_native_exec", { path, profile }),
  runNativeExec: (runId: string, preview: NativeExecPreviewDto) =>
    invoke<NativeExecResultDto>("run_native_exec", { runId, preview }),
  cancelNativeExec: (runId: string) =>
    invoke<CancelResultDto>("cancel_native_exec", { runId }),
  selectEvidenceFile: (path: string) =>
    invoke<EvidenceItemDto | null>("select_evidence_file", { path }),
  previewEvidenceExport: (repositoryPath: string, request: EvidenceExportRequestDto) =>
    invoke<EvidenceExportPreviewDto | null>("preview_evidence_export", { repositoryPath, request }),
  exportEvidence: (repositoryPath: string, preview: EvidenceExportPreviewDto) =>
    invoke<EvidenceExportResultDto | null>("export_evidence", { repositoryPath, preview }),
  previewSubmissionDraft: (path: string, draft: SubmissionDraftDto) =>
    invoke<SubmissionPreviewDto>("preview_submission_draft", { path, draft }),
  submitSubmissionDraft: (preview: SubmissionPreviewDto) =>
    invoke<SubmissionResultDto | null>("submit_submission_draft", { preview }),
  selectSubmissionImport: (path: string) =>
    invoke<SubmissionImportPreviewDto | null>("select_submission_import", { path }),
  importSubmission: (preview: SubmissionImportPreviewDto) =>
    invoke<SubmissionResultDto | null>("import_submission", { preview }),
};
