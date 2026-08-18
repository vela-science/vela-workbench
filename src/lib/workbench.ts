import { invoke } from "@tauri-apps/api/core";
import type {
  BootstrapDto,
  CancelResultDto,
  DecisionExecutionDto,
  DecisionInboxDto,
  DecisionPreviewDto,
  DecisionRequestDto,
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
  OpenGaussHandoffPreviewDto,
  OpenGaussHandoffReceiptDto,
  PreferencesDto,
  RepositorySnapshotDto,
  SubmissionDraftDto,
  SubmissionImportPreviewDto,
  SubmissionPreviewDto,
  SubmissionResultDto,
  VerificationDraftDto,
  VerificationImportPreviewDto,
  VerificationMethodDto,
  VerificationPreviewDto,
  VerificationResultDto,
  RecoveryPreviewDto,
  RecoveryResultDto,
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
  refreshDecisionInbox: (path: string) =>
    invoke<DecisionInboxDto>("refresh_decision_inbox", { path }),
  selectVerificationMethod: (path: string) =>
    invoke<VerificationMethodDto | null>("select_verification_method", { path }),
  previewVerificationRecord: (path: string, draft: VerificationDraftDto) =>
    invoke<VerificationPreviewDto>("preview_verification_record", { path, draft }),
  recordVerification: (preview: VerificationPreviewDto) =>
    invoke<VerificationResultDto | null>("record_verification", { preview }),
  selectVerificationImport: (path: string) =>
    invoke<VerificationImportPreviewDto | null>("select_verification_import", { path }),
  importVerification: (preview: VerificationImportPreviewDto) =>
    invoke<VerificationResultDto | null>("import_verification", { preview }),
  previewDecision: (path: string, request: DecisionRequestDto) =>
    invoke<DecisionPreviewDto>("preview_decision", { path, request }),
  executeDecision: (preview: DecisionPreviewDto) =>
    invoke<DecisionExecutionDto | null>("execute_decision", { preview }),
  previewRecovery: (path: string, operationId: string) =>
    invoke<RecoveryPreviewDto>("preview_recovery", { path, operationId }),
  recoverTransaction: (preview: RecoveryPreviewDto) =>
    invoke<RecoveryResultDto | null>("recover_transaction", { preview }),
  selectOpenGauss: (path: string) =>
    invoke<OpenGaussHandoffPreviewDto | null>("select_opengauss", { path }),
  launchOpenGaussHandoff: (preview: OpenGaussHandoffPreviewDto) =>
    invoke<OpenGaussHandoffReceiptDto | null>("launch_opengauss_handoff", { preview }),
  refreshOpenGaussHandoff: (
    receipt: OpenGaussHandoffReceiptDto,
    evidenceSources: EvidenceItemDto["source"][],
    checkRunIds: string[],
  ) => invoke<OpenGaussHandoffReceiptDto>("refresh_opengauss_handoff", {
    receipt,
    evidenceSources,
    checkRunIds,
  }),
};
