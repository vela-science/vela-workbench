import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { BootstrapDto, ProblemHandoffDto, RepositorySnapshotDto } from "./contracts/generated/ipc";

const bootstrap: BootstrapDto = {
  preferences: { recent_repositories: [], vela_binary_path: "/usr/local/bin/vela" },
  runtime: {
    interface_commit: "1c1abe8f365f16803fea889bf9280877992a6d02",
    interface_tree: "66bb4cb5173ff50beeef45c03fa11060e1e9e377",
    runtime_version: "vela 0.977.3",
    runtime_commit: "1c1abe8f365f16803fea889bf9280877992a6d02",
    runtime_sha256: "3a1173918bdcb887155bab681411bf5e9ff64d925fe1b50369ac37ab020b94ad",
    read_only: false,
    tranche: "3",
    mutation_scope: "explicit_verification_and_attributed_repository_decision",
    tranche_three_enabled: true,
  },
};

const snapshot: RepositorySnapshotDto = {
  path: "/private/research/math",
  name: "math",
  observed_at_unix_ms: 1_787_000_000_000,
  classification: "vela_repository",
  classification_basis: "Validated vela.status.v4 from the selected signed runtime.",
  git: {
    root: "/private/research/math", branch: "main", detached: false,
    head_commit: "0123456789012345678901234567890123456789",
    head_tree: "abcdefabcdefabcdefabcdefabcdefabcdefabcd",
    upstream: "origin/main", ahead: 0, behind: 0, dirty: false, conflicted: false,
    changed_paths: 0,
    worktrees: [{ path: "/private/research/math", head: "0123456789012345678901234567890123456789", branch: "main", detached: false, locked: false, prunable: false }],
    remotes: [{ name: "origin", url: "git@github.com:vela-science/math.git", operation: "fetch" }],
    entire_checkpoints: [],
  },
  vela: {
    binary: { path: "/usr/local/bin/vela", version: "vela 0.977.3", sha256: bootstrap.runtime.runtime_sha256, state: "signed_runtime_baseline" },
    status: {
      repository_id: "vela-math", repository_name: "Vela Math", repository_profile_root: "profile-root",
      repository_root: "repository-root", origin_root: "origin-root", authority_keyset_root: "keys-root",
      authority_policy_root: "policy-root", repository_commit: "0123456789012345678901234567890123456789",
      repository_tree: "abcdefabcdefabcdefabcdefabcdefabcdefabcd", replay: "valid", strict: "valid", blocker_count: 0,
      counts: { claims: 1, accepted_claims: 1, pending_claims: 0, pending_review: 0, accepted_review: 1, rejected_review: 0, withdrawn_review: 0, submissions: 1, verifications: 1, artifacts: 0 },
      inbox_pending: 0, inbox_projection_root: null, work_mode: "orient", work_command: "vela status --repo . --json", work_note: "Inspect exact current state.",
    },
    claims: [{ claim_id: "claim-1", claim_root: "claim-root", standing: "accepted", origin_era: "current", readable: true, assertion_kind: "statement", assertion: "A bounded accepted result.", unreadable_reason: null, created_at: null, revision: 1 }],
    integration: null, refusal: null, recovery_operation_id: null,
  },
  entire: { cli_available: false, checkpoint_reference_count: 0, note: "No substitute store." },
};

const sourceSnapshot: RepositorySnapshotDto = {
  ...snapshot,
  path: "/private/research/lean-proofs",
  name: "lean-proofs",
  classification: "git_only",
  classification_basis: "No supported Vela state or native integration was found.",
  git: {
    ...snapshot.git,
    root: "/private/research/lean-proofs",
    worktrees: [{ ...snapshot.git.worktrees[0], path: "/private/research/lean-proofs" }],
    remotes: [{ name: "origin", url: "git@github.com:vela-science/lean-proofs.git", operation: "fetch" }],
  },
  vela: { ...snapshot.vela, status: null, claims: [], binary: snapshot.vela.binary },
};

const calls = vi.hoisted(() => ({
  bootstrap: vi.fn(), selectRepository: vi.fn(), inspectRepository: vi.fn(),
  reviewProblemHandoff: vi.fn(), openProblemHandoff: vi.fn(), reviewProblemHandoffSource: vi.fn(), reviewProblemHandoffAuthority: vi.fn(),
  selectVelaBinary: vi.fn(), clearRecents: vi.fn(), launchRepository: vi.fn(),
  previewWorktree: vi.fn(), createWorktree: vi.fn(), selectNativeTool: vi.fn(),
  previewNativeExec: vi.fn(), runNativeExec: vi.fn(), cancelNativeExec: vi.fn(),
  selectEvidenceFile: vi.fn(), previewEvidenceExport: vi.fn(), exportEvidence: vi.fn(),
  previewSubmissionDraft: vi.fn(), submitSubmissionDraft: vi.fn(),
  selectSubmissionImport: vi.fn(), importSubmission: vi.fn(),
  refreshDecisionInbox: vi.fn(), selectVerificationMethod: vi.fn(),
  previewVerificationRecord: vi.fn(), recordVerification: vi.fn(),
  selectVerificationImport: vi.fn(), importVerification: vi.fn(),
  previewDecision: vi.fn(), executeDecision: vi.fn(),
  previewRecovery: vi.fn(), recoverTransaction: vi.fn(),
  selectOpenGauss: vi.fn(), launchOpenGaussHandoff: vi.fn(), refreshOpenGaussHandoff: vi.fn(),
}));
const deepLinks = vi.hoisted(() => ({ observe: vi.fn() }));

vi.mock("./lib/workbench", () => ({ workbench: calls }));
vi.mock("./lib/problem-handoff", () => ({ observeProblemHandoffUrls: deepLinks.observe }));
import App from "./App";
import { TrancheThree } from "./TrancheThree";

const problemHandoff: ProblemHandoffDto = {
  schema: "vela.workbench.problem-handoff.v1",
  handoff_url: `vela-workbench://continue?v=1&problem=https%3A%2F%2Fproblems.science%2Fproblems%2Ferdos-problems%2F94&source=https%3A%2F%2Fgithub.com%2Fvela-science%2Flean-proofs.git&ref=${snapshot.git.head_commit}&repository=https%3A%2F%2Fgithub.com%2Fvela-science%2Fmath.git&artifact=result.txt`,
  problem_url: "https://problems.science/problems/erdos-problems/94",
  source_repository_url: "https://github.com/vela-science/lean-proofs",
  source_revision: snapshot.git.head_commit,
  authority_repository_url: "https://github.com/vela-science/math",
  artifact_paths: ["result.txt"],
  authority_effect: "none",
  boundary: "Browser-safe locators only. No authority is inferred.",
};

const nextProblemHandoff: ProblemHandoffDto = {
  ...problemHandoff,
  handoff_url: problemHandoff.handoff_url
    .replace("erdos-problems%2F94", "formal-conjectures%2FIMO-1959-1")
    .replace(snapshot.git.head_commit, "f".repeat(40)),
  problem_url: "https://problems.science/problems/formal-conjectures/IMO-1959-1",
  source_revision: "f".repeat(40),
};

const verification = {
  verification_record_id: "vvr_review", verification_record_root: "sha256:verification",
  verifier: "agent:reviewer-b", performer_kind: "agent", performer_identifier: null,
  provider: null, version: null, method_metadata_status: "unavailable_or_mismatched", method_profile: "lean-proof-review",
  method_path: ".vela/methods/review.json", environment_root: "sha256:environment",
  property: "formal-correctness", outcome: "pass", declared_independent_of: [],
  shared_dependencies: ["same provider and source tree"], evidence_artifact_ids: ["var_input"],
  output_artifact_ids: ["var_output"], does_not_establish: ["scientific acceptance"],
  protocol_evidence_role: "independent", satisfies_requirements: ["formal review"],
};

const entry = {
  proposal_id: "vpr_current", proposal_root: "sha256:proposal", submission_root: "sha256:submission",
  claim_id: "vcl_current", claim_root: "sha256:claim", repository_root: "sha256:repository-before",
  verification_set_root: "sha256:set", entry_root: "sha256:entry", assertion: "A scoped result.",
  proposal_actor: "agent:producer", proposal_action: "new", proposal_reason: "bounded result",
  created_at: "2026-08-18T00:00:00Z", protocol_gate: "ready", blockers: [], rejection_available: true,
  verification_requirements: ["formal review"], verifications: [verification], limits: ["fixture only"],
  standing_delta: { transition: "pending to accepted", current: { claim_id: null, claim_root: null, standing: null, revision: null, repository_root: "sha256:repository-before" }, accept: { claim_id: "vcl_current", claim_root: "sha256:claim", standing: "accepted", revision: 1, repository_root: "sha256:repository-after" }, reject: { claim_id: null, claim_root: null, standing: "rejected", revision: null, repository_root: "sha256:repository-rejected" } },
  authority_keyset_root: "sha256:keys", policy_bundle_root: "sha256:policy",
  authority_record_root: "sha256:authority", authority_event_log_root: "sha256:event-log",
};

const opengaussPreview = {
  repository_path: snapshot.path,
  tool: {
    path: "/opt/gauss/bin/gauss", version: "Gauss v0.2.2 (2026.4.5)", sha256: "sha256:gauss", size: 8123,
    probe_argv: ["--version"], probe_environment: [{ name: "PATH", value: "/opt/gauss/bin:/usr/bin:/bin" }],
    trust_warning: "The selected OpenGauss executable runs with your current local user privileges. Bounds are not a sandbox or security isolation.",
  },
  project: {
    manifest_path: `${snapshot.path}/.gauss/project.yaml`, manifest_sha256: "sha256:project", manifest_size: 214,
    schema_version: 1, name: "Disposable pilot", kind: "lean4", project_root: snapshot.path, lean_root: snapshot.path,
    source_mode: "init", template_source_declared: false, blueprint_markers: [], configured_paths_validated: true,
  },
  git_before: { branch: "main", commit: snapshot.git.head_commit, tree: snapshot.git.head_tree, dirty: false, changed_paths: 0 },
  cwd: snapshot.path, interactive_argv: ["/opt/gauss/bin/gauss"],
  launcher_environment: [{ name: "PATH", value: "/usr/bin:/bin" }],
  documented_workflows: ["/prove", "/draft", "/review", "/checkpoint", "/refactor", "/golf", "/formalize"],
  documented_entrypoint: "Interactive OpenGauss slash command selected by the user after handoff",
  backend_identity: "Not exposed by project.yaml or the Workbench handoff; OpenGauss owns backend selection",
  hidden_transport_visible: false,
  upstream_source_commit: "f87633900ae185b8037bf451a914fe7eeae1eb08",
  upstream_source_tree: "aa3768f7cf5dd06d01a972bc8ed789f7b43246fb",
  authority_effect: "none",
  boundary: "Workbench opens Terminal at the exact project root. It does not start OpenGauss, type a slash command, observe hidden model transport, or ingest OpenGauss session state.",
} as const;

describe("Vela Workbench product loop", () => {
  afterEach(cleanup);
  beforeEach(() => {
    vi.clearAllMocks();
    deepLinks.observe.mockResolvedValue(() => {});
    calls.bootstrap.mockResolvedValue(bootstrap);
    calls.selectRepository.mockResolvedValue(snapshot);
    calls.inspectRepository.mockResolvedValue(snapshot);
    calls.launchRepository.mockResolvedValue({ target: snapshot.path, owner: "Terminal" });
    calls.selectNativeTool.mockResolvedValue({ profile: "git_diff_check", path: "/usr/bin/git", sha256: "sha256:git", size: 100 });
    calls.previewNativeExec.mockResolvedValue({
      profile: "git_diff_check", label: "Git diff check", repository_path: snapshot.path,
      source_commit: snapshot.git.head_commit, source_tree: snapshot.git.head_tree,
      executable: { profile: "git_diff_check", path: "/usr/bin/git", sha256: "sha256:git", size: 100 },
      argv: ["diff", "--no-ext-diff", "--no-textconv", "--check"], working_directory: snapshot.path,
      environment: [{ name: "PATH", value: "/usr/bin:/bin" }], timeout_ms: 30000,
      max_stdout_bytes: 2097152, max_stderr_bytes: 1048576,
      trust_warning: "This profile can execute repository-controlled build scripts or plugins with your current local user privileges. Output, environment, lifetime, and process-tree controls are bounds only; this is not a sandbox or security isolation.",
      sandboxed: false,
    });
    calls.runNativeExec.mockResolvedValue({
      run_id: "run-explicit-1234", profile: "git_diff_check", state: "completed", exit_code: 0,
      started_at_unix_ms: 1, completed_at_unix_ms: 2,
      source_commit: snapshot.git.head_commit, source_tree: snapshot.git.head_tree,
      executable_sha256: "sha256:git",
      stdout: { stream: "stdout", sha256: "sha256:empty", size: 0, content_base64: "", content_utf8: "", truncated: false },
      stderr: { stream: "stderr", sha256: "sha256:empty", size: 0, content_base64: "", content_utf8: "", truncated: false },
      producer_check_method: "vela-workbench-git-diff-check", producer_check_outcome: "pass",
    });
    calls.selectEvidenceFile.mockResolvedValue({
      source: { source: "local_file", path: `${snapshot.path}/result.txt`, repository_relative_path: "result.txt" },
      display_name: "result.txt", sha256: "sha256:evidence", size: 7, media_type: "text/plain",
      kind_hint: "output", source_commit: snapshot.git.head_commit, source_tree: snapshot.git.head_tree,
      source_dirty: false, content_base64: "cmVzdWx0Cg==", content_utf8: "result\n", private: true,
    });
    calls.reviewProblemHandoff.mockResolvedValue(problemHandoff);
    calls.openProblemHandoff.mockResolvedValue({ target: problemHandoff.problem_url, owner: "default HTTPS browser" });
    calls.reviewProblemHandoffSource.mockResolvedValue({
      repository_path: sourceSnapshot.path,
      source_repository_url: problemHandoff.source_repository_url,
      source_revision: problemHandoff.source_revision,
      selected_head: snapshot.git.head_commit,
      remote_matches: true,
      revision_matches: true,
      ready: true,
      note: "The selected checkout matches the handoff source remote and exact revision.",
    });
    calls.reviewProblemHandoffAuthority.mockResolvedValue({
      repository_path: snapshot.path,
      authority_repository_url: problemHandoff.authority_repository_url,
      repository_id: "vela-math",
      remote_matches: true,
      vela_repository: true,
      ready: true,
      note: "The selected checkout matches the authority Repository locator and contains current Vela Repository state. Vela still authenticates and authorizes every Decision separately.",
    });
    calls.previewRecovery.mockResolvedValue({
      repository_path: snapshot.path,
      operation_id: `vop_${"a".repeat(64)}`,
      source_commit: snapshot.git.head_commit,
      source_tree: snapshot.git.head_tree,
      vela_binary_sha256: bootstrap.runtime.runtime_sha256,
      argv: ["recover", "--repo", snapshot.path, `vop_${"a".repeat(64)}`, "--json"],
      warning: "Recovery applies only the signed Vela transaction journal for this exact operation. It never retries or chooses a Decision.",
    });
    calls.previewSubmissionDraft.mockResolvedValue(null);
    calls.refreshDecisionInbox.mockResolvedValue({ repository_id: "vela-math", repository_root: entry.repository_root, projection_root: "sha256:projection", entries: [entry], observed_at_unix_ms: 1_787_000_000_000, task: "Review exact pending Proposals", included_records: ["Proposal", "Submission", "Verification", "Standing"], omissions: ["No hidden session or provider state is included."], stale: false, refusal: null });
    calls.previewDecision.mockResolvedValue({ request: { proposal_id: entry.proposal_id, entry_root: entry.entry_root, action: "accept", reason: "Evidence supports the bounded claim.", performer: "agent:reviewer", session_ref: null }, repository_path: snapshot.path, source_commit: snapshot.git.head_commit, source_tree: snapshot.git.head_tree, vela_binary_sha256: bootstrap.runtime.runtime_sha256, entry, performer_kind: "agent", repository_authority_principal: "Resolved by signed Vela during execution; performer does not grant authority.", authentication: "Local OS and repository policy", transaction_signer: "Repository authority signer selected by signed Vela", ssh_agent_forwarded: true, argv: ["review", "accept", "--if-entry-root", entry.entry_root], expected_successor: entry.standing_delta.accept, warning: "Authority changes only after native confirmation." });
    calls.executeDecision.mockResolvedValue({ command_succeeded: true, decision_committed: true, successor_matches_preview: true, events_match_receipt: true, action: "accept", proposal_id: entry.proposal_id, entry_root: entry.entry_root, decision_plan_root: "sha256:plan", event_ids: ["vev_decision", "vev_applied"], authority_record_id: "var_authority", actual_performer: "agent:reviewer", actual_performer_kind: "agent", actual_authority_principal: "local:fixture", authentication: "local", transaction_signer: "fixture signer", scientific_state_changed: true, refusal: null, readback: { status: "accepted", decision_event_id: "vev_decision", applied_event_id: "vev_applied", standing: "accepted", claim_id: entry.claim_id, claim_root: entry.claim_root, repository_root: "sha256:repository-after", pending_inbox_count: 0, accepted_event_count: 1 } });
    calls.selectOpenGauss.mockResolvedValue(opengaussPreview);
    calls.launchOpenGaussHandoff.mockResolvedValue({ preview: opengaussPreview, terminal_owner: "Terminal", launched_at_unix_ms: 1_787_000_000_100, git_after: null, selected_evidence: [], selected_checks: [], result_boundary: "No external result is inferred. Select exact evidence." });
    calls.refreshOpenGaussHandoff.mockImplementation(async (receipt) => ({ ...receipt, git_after: opengaussPreview.git_before }));
  });

  it("starts with a local-only repository choice and runtime boundary", async () => {
    render(<App />);
    expect(await screen.findByText("Continue local scientific work")).toBeVisible();
    expect(screen.getByText("Private files, credentials, and evidence stay local.")).toBeVisible();
    expect(await screen.findByText("vela 0.977.3")).toBeVisible();
  });

  it("reviews a browser handoff before binding the exact local source", async () => {
    calls.selectRepository.mockResolvedValueOnce(sourceSnapshot);
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    expect(await screen.findByText("Continue this Problem locally")).toBeVisible();
    expect(calls.reviewProblemHandoff).toHaveBeenCalledWith(problemHandoff.handoff_url);
    expect(screen.getByText(problemHandoff.problem_url)).toBeVisible();
    expect(screen.getByText(problemHandoff.authority_repository_url)).toBeVisible();
    expect(screen.getByText("result.txt")).toBeVisible();
    await user.click(screen.getByRole("button", { name: "Choose local source" }));
    expect(await screen.findByText(/exact source selected/)).toBeVisible();
    expect(calls.reviewProblemHandoffSource).toHaveBeenCalledWith(sourceSnapshot.path, problemHandoff);
    await user.click(screen.getByRole("tab", { name: "Work" }));
    expect(screen.getByLabelText("Exact target ref")).toHaveValue(problemHandoff.source_revision);
    await user.click(screen.getByRole("button", { name: "Open exact Problem" }));
    expect(calls.openProblemHandoff).toHaveBeenCalledWith(problemHandoff);
  });

  it("keeps the Result draft while explicitly continuing from source to the separate Repository", async () => {
    calls.selectRepository
      .mockResolvedValueOnce(sourceSnapshot)
      .mockResolvedValueOnce(snapshot);
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    await screen.findByText(problemHandoff.problem_url);
    await user.click(screen.getByRole("button", { name: "Choose local source" }));
    expect(await screen.findByText(/exact source selected/)).toBeVisible();
    await user.click(screen.getByRole("tab", { name: "Submit" }));
    await user.selectOptions(screen.getByLabelText("Result type"), "theoretical");
    fireEvent.change(screen.getByLabelText("Bounded result"), { target: { value: "The exact source proves one bounded identity." } });
    fireEvent.change(screen.getByLabelText("Required caveat"), { target: { value: "This does not establish Repository acceptance." } });
    fireEvent.change(screen.getByLabelText("Required independent Check"), { target: { value: "Replay the exact proof source." } });
    await user.click(screen.getByRole("button", { name: "Choose local Repository" }));
    await waitFor(() => expect(calls.reviewProblemHandoffAuthority).toHaveBeenCalledWith(snapshot.path, problemHandoff));
    expect(await screen.findByRole("button", { name: "Continue to Repository" })).toBeVisible();
    expect(screen.getByText("Selected authority Repository")).toBeVisible();
    expect(screen.getByLabelText("Bounded result")).toHaveValue("The exact source proves one bounded identity.");
    expect(screen.getByLabelText("Result type")).toHaveValue("theoretical");
    expect(screen.getByLabelText("Required caveat")).toHaveValue("This does not establish Repository acceptance.");
    expect(screen.getByLabelText("Required independent Check")).toHaveValue("Replay the exact proof source.");
    expect(screen.getByText(/Source bytes never move automatically/)).toBeVisible();
    expect(screen.getByText(/Export each exact selected Artifact into the authority Repository/)).toBeVisible();
    calls.inspectRepository.mockResolvedValueOnce(sourceSnapshot);
    await user.click(screen.getByRole("button", { name: "Return to source" }));
    expect(await screen.findByText("Selected Problem source")).toBeVisible();
    expect(screen.getByLabelText("Bounded result")).toHaveValue("The exact source proves one bounded identity.");
  });

  it("keeps the newest Problem ref when an older repository picker finishes", async () => {
    let finishPicker!: (value: RepositorySnapshotDto | null) => void;
    calls.selectRepository.mockReturnValue(new Promise((resolve) => { finishPicker = resolve; }));
    calls.reviewProblemHandoff.mockImplementation(async (url: string) => url === nextProblemHandoff.handoff_url ? nextProblemHandoff : problemHandoff);
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    await screen.findByText(problemHandoff.problem_url);
    await user.click(screen.getByRole("button", { name: "Choose local source" }));
    await waitFor(() => expect(calls.selectRepository).toHaveBeenCalledTimes(1));
    accept(nextProblemHandoff.handoff_url);
    await screen.findByText(nextProblemHandoff.problem_url);
    finishPicker(sourceSnapshot);
    await screen.findByRole("heading", { name: "lean-proofs" }, { timeout: 5_000 });
    await user.click(screen.getByRole("tab", { name: "Work" }));
    expect(screen.getByLabelText("Exact target ref")).toHaveValue(nextProblemHandoff.source_revision);
  });

  it("does not bind a delayed source review to a newer Problem", async () => {
    let finishReview!: (value: Awaited<ReturnType<typeof calls.reviewProblemHandoffSource>>) => void;
    calls.selectRepository.mockResolvedValueOnce(sourceSnapshot);
    calls.reviewProblemHandoff.mockImplementation(async (url: string) => url === nextProblemHandoff.handoff_url ? nextProblemHandoff : problemHandoff);
    calls.reviewProblemHandoffSource.mockReturnValue(new Promise((resolve) => { finishReview = resolve; }));
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    await screen.findByText(problemHandoff.problem_url);
    await user.click(screen.getByRole("button", { name: "Choose local source" }));
    await waitFor(() => expect(calls.reviewProblemHandoffSource).toHaveBeenCalledTimes(1));
    accept(nextProblemHandoff.handoff_url);
    await screen.findByText(nextProblemHandoff.problem_url);
    finishReview({
      repository_path: sourceSnapshot.path,
      source_repository_url: problemHandoff.source_repository_url,
      source_revision: problemHandoff.source_revision,
      selected_head: sourceSnapshot.git.head_commit,
      remote_matches: true,
      revision_matches: true,
      ready: true,
      note: "Old source review.",
    });
    await waitFor(() => expect(screen.queryByRole("button", { name: "Return to source" })).not.toBeInTheDocument());
    expect(screen.getByText(/source selection required/)).toBeVisible();
  });

  it("does not bind a delayed authority review to a newer Problem", async () => {
    let finishReview!: (value: Awaited<ReturnType<typeof calls.reviewProblemHandoffAuthority>>) => void;
    calls.selectRepository.mockResolvedValueOnce(snapshot);
    calls.reviewProblemHandoff.mockImplementation(async (url: string) => url === nextProblemHandoff.handoff_url ? nextProblemHandoff : problemHandoff);
    calls.reviewProblemHandoffAuthority.mockReturnValue(new Promise((resolve) => { finishReview = resolve; }));
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    await screen.findByText(problemHandoff.problem_url);
    await user.click(screen.getByRole("button", { name: "Choose local Repository" }));
    await waitFor(() => expect(calls.reviewProblemHandoffAuthority).toHaveBeenCalledTimes(1));
    accept(nextProblemHandoff.handoff_url);
    await screen.findByText(nextProblemHandoff.problem_url);
    finishReview({
      repository_path: snapshot.path,
      authority_repository_url: problemHandoff.authority_repository_url,
      repository_id: "vela-math",
      remote_matches: true,
      vela_repository: true,
      ready: true,
      note: "Old authority review.",
    });
    await waitFor(() => expect(screen.queryByRole("button", { name: "Continue to Repository" })).not.toBeInTheDocument());
    expect(screen.getByText(/Repository selection required/)).toBeVisible();
  });

  it("revokes source readiness when refresh finds checkout drift", async () => {
    calls.selectRepository.mockResolvedValueOnce(sourceSnapshot);
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    await screen.findByText(problemHandoff.problem_url);
    await user.click(screen.getByRole("button", { name: "Choose local source" }));
    expect(await screen.findByText(/exact source selected/)).toBeVisible();
    const drifted = { ...sourceSnapshot, git: { ...sourceSnapshot.git, head_commit: "e".repeat(40) } };
    calls.inspectRepository.mockResolvedValueOnce(drifted);
    calls.reviewProblemHandoffSource.mockResolvedValueOnce({
      repository_path: sourceSnapshot.path,
      source_repository_url: problemHandoff.source_repository_url,
      source_revision: problemHandoff.source_revision,
      selected_head: drifted.git.head_commit,
      remote_matches: true,
      revision_matches: false,
      ready: false,
      note: "The source remote matches, but the selected checkout is at a different revision.",
    });
    await user.click(screen.getByRole("button", { name: "Refresh" }));
    expect(await screen.findByText(/source mismatch/)).toBeVisible();
    expect(screen.queryByRole("button", { name: "Return to source" })).not.toBeInTheDocument();
    expect(screen.getByText("Selected local repository")).toBeVisible();
  });

  it("keeps the Problem draft but clears repository evidence on a bound refresh", async () => {
    calls.selectRepository.mockResolvedValueOnce(sourceSnapshot);
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    await screen.findByText(problemHandoff.problem_url);
    await user.click(screen.getByRole("button", { name: "Choose local source" }));
    expect(await screen.findByText(/exact source selected/)).toBeVisible();
    await user.click(screen.getByRole("tab", { name: "Capture" }));
    await user.click(screen.getByRole("button", { name: "Choose one file" }));
    expect((await screen.findAllByText("result.txt")).length).toBeGreaterThan(1);
    await user.click(screen.getByRole("tab", { name: "Submit" }));
    await user.selectOptions(screen.getByLabelText("Result type"), "theoretical");
    fireEvent.change(screen.getByLabelText("Bounded result"), { target: { value: "The exact source proves one bounded identity." } });
    fireEvent.change(screen.getByLabelText("Required caveat"), { target: { value: "This does not establish Repository acceptance." } });
    fireEvent.change(screen.getByLabelText("Required independent Check"), { target: { value: "Replay the exact proof source." } });
    calls.inspectRepository.mockResolvedValueOnce(sourceSnapshot);
    await user.click(screen.getByRole("button", { name: "Refresh" }));
    await waitFor(() => expect(calls.reviewProblemHandoffSource).toHaveBeenCalledTimes(2));
    expect(screen.getByLabelText("Bounded result")).toHaveValue("The exact source proves one bounded identity.");
    expect(screen.getByLabelText("Result type")).toHaveValue("theoretical");
    expect(screen.getByLabelText("Required caveat")).toHaveValue("This does not establish Repository acceptance.");
    expect(screen.getByLabelText("Required independent Check")).toHaveValue("Replay the exact proof source.");
    expect(screen.getByText("No repository file evidence is selected.")).toBeVisible();
  });

  it("revokes Repository readiness when refresh finds remote or Vela drift", async () => {
    calls.selectRepository.mockResolvedValueOnce(snapshot);
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    await screen.findByText(problemHandoff.problem_url);
    await user.click(screen.getByRole("button", { name: "Choose local Repository" }));
    expect(await screen.findByRole("button", { name: "Continue to Repository" })).toBeVisible();
    const drifted = { ...snapshot, classification: "git_only" as const, git: { ...snapshot.git, remotes: [] } };
    calls.inspectRepository.mockResolvedValueOnce(drifted);
    calls.reviewProblemHandoffAuthority.mockResolvedValueOnce({
      repository_path: snapshot.path,
      authority_repository_url: problemHandoff.authority_repository_url,
      repository_id: null,
      remote_matches: false,
      vela_repository: false,
      ready: false,
      note: "The selected checkout does not match the handoff authority Repository.",
    });
    await user.click(screen.getByRole("button", { name: "Refresh" }));
    expect(await screen.findByText(/Repository mismatch/)).toBeVisible();
    expect(screen.queryByRole("button", { name: "Continue to Repository" })).not.toBeInTheDocument();
    expect(screen.getByText("Selected local repository")).toBeVisible();
  });

  it("clears prior Problem draft and evidence when a new handoff is accepted", async () => {
    calls.selectRepository.mockResolvedValueOnce(sourceSnapshot);
    calls.reviewProblemHandoff.mockImplementation(async (url: string) => url === nextProblemHandoff.handoff_url ? nextProblemHandoff : problemHandoff);
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    await screen.findByText(problemHandoff.problem_url);
    await user.click(screen.getByRole("button", { name: "Choose local source" }));
    await user.click(screen.getByRole("tab", { name: "Capture" }));
    await user.click(screen.getByRole("button", { name: "Choose one file" }));
    expect((await screen.findAllByText("result.txt")).length).toBeGreaterThan(1);
    await user.click(screen.getByRole("tab", { name: "Submit" }));
    await user.selectOptions(screen.getByLabelText("Result type"), "theoretical");
    fireEvent.change(screen.getByLabelText("Bounded result"), { target: { value: "Old Problem Result." } });
    fireEvent.change(screen.getByLabelText("Required caveat"), { target: { value: "Old caveat." } });
    accept(nextProblemHandoff.handoff_url);
    await screen.findByText(nextProblemHandoff.problem_url);
    expect(screen.getByLabelText("Bounded result")).toHaveValue("");
    expect(screen.getByLabelText("Result type")).toHaveValue("computational");
    expect(screen.getByLabelText("Required caveat")).toHaveValue("");
    expect(screen.getByText("No repository file evidence is selected.")).toBeVisible();
  });

  it("orients from a selected repository and exposes only exact handoffs", async () => {
    const user = userEvent.setup();
    render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    expect(await screen.findByText("Vela Math")).toBeVisible();
    expect(screen.getByText("A bounded accepted result.")).toBeVisible();
    await user.click(screen.getByRole("tab", { name: "Work" }));
    expect(await screen.findByText("Open exact source elsewhere")).toBeVisible();
    await user.click(screen.getByRole("button", { name: "Terminal" }));
    expect(calls.launchRepository).toHaveBeenCalledWith(snapshot.path, "terminal");
  });

  it("surfaces exact restart recovery from fresh signed repository inspection", async () => {
    const operationId = `vop_${"a".repeat(64)}`;
    const incomplete: RepositorySnapshotDto = {
      ...snapshot,
      classification_basis: "Signed Vela found one exact interrupted Repository operation; explicit recovery is required before other authority work.",
      vela: {
        ...snapshot.vela,
        status: null,
        claims: [],
        recovery_operation_id: operationId,
        refusal: null,
      },
    };
    calls.selectRepository.mockResolvedValue(incomplete);
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    expect(await screen.findByText("Vela repository")).toBeVisible();
    await user.click(screen.getByRole("tab", { name: "Check & Decide" }));
    expect(await screen.findByText(operationId)).toBeVisible();
    expect(screen.getByText(/fresh signed Vela recovery inspection found this unfinished Repository transaction/)).toBeVisible();
    await user.click(screen.getByRole("button", { name: "Review recovery" }));
    expect(calls.previewRecovery).toHaveBeenCalledWith(snapshot.path, operationId);
    expect(await screen.findByText(/Recovery applies only the signed Vela transaction journal/)).toBeVisible();
  });

  it("clears a recovery preview when fresh inspection replaces the operation", async () => {
    const user = userEvent.setup();
    const operationA = `vop_${"a".repeat(64)}`;
    const operationB = `vop_${"b".repeat(64)}`;
    const withOperation = (operationId: string | null): RepositorySnapshotDto => ({
      ...snapshot,
      vela: {
        ...snapshot.vela,
        status: operationId ? null : snapshot.vela.status,
        claims: operationId ? [] : snapshot.vela.claims,
        recovery_operation_id: operationId,
      },
    });
    const onRepositoryChanged = vi.fn(async () => {});
    const rendered = render(
      <TrancheThree snapshot={withOperation(operationA)} evidence={[]} onRepositoryChanged={onRepositoryChanged} />,
    );
    await user.click(screen.getByRole("button", { name: "Review recovery" }));
    expect(await screen.findByText(/Recovery applies only the signed Vela transaction journal/)).toBeVisible();
    rendered.rerender(
      <TrancheThree snapshot={withOperation(operationB)} evidence={[]} onRepositoryChanged={onRepositoryChanged} />,
    );
    expect(await screen.findByText(operationB)).toBeVisible();
    await waitFor(() => expect(screen.queryByText(/Recovery applies only the signed Vela transaction journal/)).not.toBeInTheDocument());
    expect(screen.getByText("Exact transaction recovery")).toBeVisible();
    rendered.rerender(
      <TrancheThree snapshot={withOperation(null)} evidence={[]} onRepositoryChanged={onRepositoryChanged} />,
    );
    await waitFor(() => expect(screen.queryByText("Exact transaction recovery")).not.toBeInTheDocument());
  });

  it("requires explicit native execution review and says the controls are not a sandbox", async () => {
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    await user.click(screen.getByRole("tab", { name: "Work" }));
    await user.click(screen.getByRole("button", { name: "Select tool" }));
    await user.click(await screen.findByRole("button", { name: "Review command" }));
    expect(await screen.findByText(/current local user privileges/)).toBeVisible();
    expect(screen.getByText("not sandboxed")).toBeVisible();
    expect(calls.runNativeExec).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "Start explicitly" }));
    await waitFor(() => expect(calls.runNativeExec).toHaveBeenCalledTimes(1));
  });

  it("states that redaction creates a derived file and never edits selected evidence", async () => {
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    await user.click(screen.getByRole("tab", { name: "Capture" }));
    await user.click(screen.getByRole("button", { name: "Choose one file" }));
    expect(await screen.findByText("Redaction creates a new derived file. The selected source evidence is never edited.")).toBeVisible();
    expect(screen.getByLabelText("Exact selected evidence bytes")).toHaveValue("result\n");
    await user.click(screen.getByRole("checkbox", { name: "Create a derived redacted text output" }));
    await user.type(screen.getByLabelText("Exclusions / redactions (one per line)"), "private line");
    const derived = screen.getByLabelText("Exact derived UTF-8 output");
    await user.clear(derived);
    await user.type(derived, "public\n");
    calls.previewEvidenceExport.mockResolvedValue(null);
    await user.click(screen.getByRole("button", { name: "Choose destination and review export" }));
    await waitFor(() => expect(calls.previewEvidenceExport).toHaveBeenCalledWith(snapshot.path, expect.objectContaining({
      expected_sha256: "sha256:evidence",
      exclusions: ["private line"],
      redaction_confirmed: true,
      derived_utf8: "public\n",
    })));
    expect(screen.getByLabelText("Exact selected evidence bytes")).toHaveValue("result\n");
  });

  it("carries the selected Result type into the exact Submission preview", async () => {
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    await user.click(screen.getByRole("tab", { name: "Capture" }));
    await user.click(screen.getByRole("button", { name: "Choose one file" }));
    await user.click(screen.getByRole("tab", { name: "Submit" }));
    expect(screen.getByLabelText("Result type").querySelectorAll("option")).toHaveLength(5);
    expect(Array.from(screen.getByLabelText("Result type").querySelectorAll("option"), (option) => option.value)).toEqual([
      "theoretical", "computational", "empirical", "negative", "contradiction",
    ]);
    await user.selectOptions(screen.getByLabelText("Result type"), "theoretical");
    fireEvent.change(screen.getByLabelText("Bounded result"), { target: { value: "A reusable theorem." } });
    fireEvent.change(screen.getByLabelText("Required caveat"), { target: { value: "This does not establish Repository acceptance." } });
    await user.click(screen.getByRole("button", { name: "Review exact CLI operation" }));
    await waitFor(() => expect(calls.previewSubmissionDraft).toHaveBeenCalledWith(
      snapshot.path,
      expect.objectContaining({
        assertion: "A reusable theorem.",
        claim_type: "theoretical",
        artifacts: [expect.objectContaining({ path: "result.txt", sha256: "sha256:evidence" })],
      }),
    ));
  });

  it("clears the complete source-bound Result draft on repository refresh", async () => {
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    await user.click(screen.getByRole("tab", { name: "Submit" }));
    await user.selectOptions(screen.getByLabelText("Result type"), "negative");
    fireEvent.change(screen.getByLabelText("Bounded result"), { target: { value: "No witness was found within the exact bound." } });
    fireEvent.change(screen.getByLabelText("Producer attribution"), { target: { value: "agent:other" } });
    fireEvent.change(screen.getByLabelText("Required caveat"), { target: { value: "The unbounded case remains open." } });
    fireEvent.change(screen.getByLabelText("Required independent Check"), { target: { value: "Repeat the bounded search." } });
    await user.click(screen.getByRole("button", { name: "Refresh" }));
    await waitFor(() => expect(calls.inspectRepository).toHaveBeenCalledWith(snapshot.path));
    expect(screen.getByLabelText("Bounded result")).toHaveValue("");
    expect(screen.getByLabelText("Result type")).toHaveValue("computational");
    expect(screen.getByLabelText("Producer attribution")).toHaveValue("agent:researcher");
    expect(screen.getByLabelText("Required caveat")).toHaveValue("");
    expect(screen.getByLabelText("Required independent Check")).toHaveValue("");
  });

  it("does not infer independence from actor identity and keeps task orientation explicit", async () => {
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    await user.click(screen.getByRole("tab", { name: "Check & Decide" }));
    expect(await screen.findByText("Inbox not read")).toBeVisible();
    expect(calls.refreshDecisionInbox).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "Refresh exact roots" }));
    expect(await screen.findByText("Review exact pending Proposals")).toBeVisible();
    expect(screen.getByText(/No hidden session or provider state/)).toBeVisible();
    expect(screen.getByText(/Actor difference is never treated as independence/)).toBeVisible();
  });

  it("previews authority identity and separates Verification from actual Decision Standing", async () => {
    const user = userEvent.setup(); render(<App />);
    await waitFor(() => expect(deepLinks.observe).toHaveBeenCalledTimes(1));
    const accept = deepLinks.observe.mock.calls[0][0] as (url: string) => void;
    accept(problemHandoff.handoff_url);
    await screen.findByText(problemHandoff.problem_url);
    await user.click(screen.getByRole("button", { name: "Choose local Repository" }));
    expect(await screen.findByRole("button", { name: "Continue to Repository" })).toBeVisible();
    await user.click(screen.getByRole("tab", { name: "Check & Decide" }));
    await user.click(screen.getByRole("button", { name: "Refresh exact roots" }));
    fireEvent.change(screen.getByLabelText("Bounded scientific reason"), { target: { value: "Evidence supports the bounded claim." } });
    await user.click(screen.getByRole("button", { name: "Review exact attributed Decision" }));
    expect(await screen.findByText(/Resolved by signed Vela during execution/)).toBeVisible();
    expect(screen.getByText(/same provider and source tree/)).toBeVisible();
    expect(screen.getByText(/does not match the signed environment root/)).toBeVisible();
    expect(calls.executeDecision).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "Execute after native confirmation" }));
    expect(await screen.findByText("Actual Decision / Event / Standing readback")).toBeVisible();
    expect(screen.getByText(/Verification outcome is not this Standing/)).toBeVisible();
    expect(screen.getByText(/Repository state is committed locally until Git publishes this checkout/)).toBeVisible();
    expect(screen.getAllByText(problemHandoff.problem_url)).toHaveLength(2);
    await user.click(screen.getByRole("button", { name: "Open Repository Terminal" }));
    expect(calls.launchRepository).toHaveBeenCalledWith(snapshot.path, "terminal");
    await user.click(screen.getByRole("button", { name: "Return to exact Problem" }));
    expect(calls.openProblemHandoff).toHaveBeenCalledWith(problemHandoff);
    calls.reviewProblemHandoff.mockImplementation(async (url: string) => url === nextProblemHandoff.handoff_url ? nextProblemHandoff : problemHandoff);
    accept(nextProblemHandoff.handoff_url);
    expect(await screen.findByText(nextProblemHandoff.problem_url)).toBeVisible();
    expect(screen.queryByText("Actual Decision / Event / Standing readback")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Return to exact Problem" })).not.toBeInTheDocument();
  });

  it("keeps OpenGauss workflows behind an explicit interactive-only handoff", async () => {
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    await user.click(screen.getByRole("tab", { name: "Work" }));
    expect(await screen.findByText("OpenGauss handoff pilot")).toBeVisible();
    await user.click(screen.getByRole("button", { name: "Select OpenGauss" }));
    expect(await screen.findByText(/current local user privileges/)).toBeVisible();
    expect(screen.getByText("/prove")).toBeVisible();
    expect(screen.getByText(/slash commands, not stable shell workflow argv/)).toBeVisible();
    expect(screen.getByText(/Workbench does not type or automate them/)).toBeVisible();
    expect(screen.getByText(/Hidden model transport visible: false/)).toBeVisible();
    expect(calls.launchOpenGaussHandoff).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "Open explicit Terminal handoff" }));
    await waitFor(() => expect(calls.launchOpenGaussHandoff).toHaveBeenCalledWith(opengaussPreview));
    expect(await screen.findByText(/Workbench did not start OpenGauss or type a workflow command/)).toBeVisible();
  });

  it("binds only explicit Workbench evidence to the OpenGauss receipt", async () => {
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    await user.click(screen.getByRole("tab", { name: "Capture" }));
    await user.click(screen.getByRole("button", { name: "Choose one file" }));
    await user.click(screen.getByRole("tab", { name: "Work" }));
    await user.click(screen.getByRole("button", { name: "Select OpenGauss" }));
    await user.click(screen.getByRole("button", { name: "Open explicit Terminal handoff" }));
    await user.click(await screen.findByRole("checkbox", { name: /result.txt/ }));
    await user.click(screen.getByRole("button", { name: "Refresh Git and bind selected evidence" }));
    await waitFor(() => expect(calls.refreshOpenGaussHandoff).toHaveBeenCalledWith(
      expect.objectContaining({ terminal_owner: "Terminal" }),
      [expect.objectContaining({ source: "local_file", repository_relative_path: "result.txt" })],
      [],
    ));
    expect(screen.getByText(/OpenGauss provenance remains external-tool provenance/)).toBeVisible();
  });

  it("renders the exact reviewed command path bound to an OpenGauss receipt", async () => {
    calls.refreshOpenGaussHandoff.mockImplementation(async (receipt, _evidence, runIds) => ({
      ...receipt,
      git_after: opengaussPreview.git_before,
      selected_checks: runIds.map((runId: string) => ({
        run_id: runId,
        repository_path: snapshot.path,
        profile: "git_diff_check" as const,
        state: "completed" as const,
        exit_code: 0,
        source_commit: snapshot.git.head_commit,
        source_tree: snapshot.git.head_tree,
        executable_path: "/usr/bin/git",
        executable_sha256: "sha256:git",
        argv: ["diff", "--no-ext-diff", "--no-textconv", "--check"],
        working_directory: snapshot.path,
        environment: [{ name: "PATH", value: "/usr/bin:/bin" }],
        timeout_ms: 30000,
        max_stdout_bytes: 2097152,
        max_stderr_bytes: 1048576,
        stdout_sha256: "sha256:empty",
        stderr_sha256: "sha256:empty",
        producer_check_method: "vela-workbench-git-diff-check",
        producer_check_outcome: "pass",
      })),
    }));
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    await user.click(screen.getByRole("tab", { name: "Work" }));
    await user.click(screen.getByRole("button", { name: "Select tool" }));
    await user.click(await screen.findByRole("button", { name: "Review command" }));
    await user.click(screen.getByRole("button", { name: "Start explicitly" }));
    await user.click(screen.getByRole("button", { name: "Select OpenGauss" }));
    await user.click(screen.getByRole("button", { name: "Open explicit Terminal handoff" }));
    await user.click(await screen.findByRole("checkbox", { name: /run-explicit-1234/ }));
    await user.click(screen.getByRole("button", { name: "Refresh Git and bind selected evidence" }));
    expect(await screen.findByText(/\/usr\/bin\/git.*"diff".*cwd \/private\/research\/math/)).toBeVisible();
    expect(screen.getByText(new RegExp(`source ${snapshot.git.head_commit}/${snapshot.git.head_tree}`))).toBeVisible();
  });
});
