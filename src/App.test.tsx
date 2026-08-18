import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { BootstrapDto, RepositorySnapshotDto } from "./contracts/generated/ipc";

const bootstrap: BootstrapDto = {
  preferences: { recent_repositories: [], vela_binary_path: "/usr/local/bin/vela" },
  runtime: {
    interface_commit: "0e057c0debcff775a3deb56150ceaccfd4707b41",
    interface_tree: "55768612b82b93a4a01bb5aeddeb937dff678e4a",
    runtime_version: "vela 0.977.1",
    runtime_commit: "0e057c0debcff775a3deb56150ceaccfd4707b41",
    runtime_sha256: "a4f5594b2777b265f6d58296cc8e9efd85d0a72c82b49c0fce4805438ed46948",
    read_only: false,
    tranche: "2",
    mutation_scope: "detached_worktree_and_submission_intake_only",
    tranche_three_enabled: false,
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
    binary: { path: "/usr/local/bin/vela", version: "vela 0.977.1", sha256: bootstrap.runtime.runtime_sha256, state: "signed_runtime_baseline" },
    status: {
      repository_id: "vela-math", repository_name: "Vela Math", repository_profile_root: "profile-root",
      repository_root: "repository-root", origin_root: "origin-root", authority_keyset_root: "keys-root",
      authority_policy_root: "policy-root", repository_commit: "0123456789012345678901234567890123456789",
      repository_tree: "abcdefabcdefabcdefabcdefabcdefabcdefabcd", replay: "valid", strict: "valid", blocker_count: 0,
      counts: { claims: 1, accepted_claims: 1, pending_claims: 0, pending_review: 0, accepted_review: 1, rejected_review: 0, withdrawn_review: 0, submissions: 1, verifications: 1, artifacts: 0 },
      inbox_pending: 0, inbox_projection_root: null, work_mode: "orient", work_command: "vela status --repo . --json", work_note: "Inspect exact current state.",
    },
    claims: [{ claim_id: "claim-1", claim_root: "claim-root", standing: "accepted", origin_era: "current", readable: true, assertion_kind: "statement", assertion: "A bounded accepted result.", unreadable_reason: null, created_at: null, revision: 1 }],
    integration: null, refusal: null,
  },
  entire: { cli_available: false, checkpoint_reference_count: 0, note: "No substitute store." },
  reviewed_problem_url: null,
};

const calls = vi.hoisted(() => ({
  bootstrap: vi.fn(), selectRepository: vi.fn(), inspectRepository: vi.fn(),
  selectVelaBinary: vi.fn(), clearRecents: vi.fn(), launchRepository: vi.fn(),
  previewWorktree: vi.fn(), createWorktree: vi.fn(), selectNativeTool: vi.fn(),
  previewNativeExec: vi.fn(), runNativeExec: vi.fn(), cancelNativeExec: vi.fn(),
  selectEvidenceFile: vi.fn(), previewEvidenceExport: vi.fn(), exportEvidence: vi.fn(),
  previewSubmissionDraft: vi.fn(), submitSubmissionDraft: vi.fn(),
  selectSubmissionImport: vi.fn(), importSubmission: vi.fn(),
}));

vi.mock("./lib/workbench", () => ({ workbench: calls }));
import App from "./App";

describe("Vela Workbench Tranche 2", () => {
  afterEach(cleanup);
  beforeEach(() => {
    vi.clearAllMocks();
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
  });

  it("starts with a local-only repository choice and runtime boundary", async () => {
    render(<App />);
    expect(await screen.findByText("Start from sovereign source")).toBeVisible();
    expect(screen.getByText("Private files, credentials, and evidence stay local.")).toBeVisible();
    expect(await screen.findByText("vela 0.977.1")).toBeVisible();
  });

  it("orients from a selected repository and exposes only exact handoffs", async () => {
    const user = userEvent.setup();
    render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    expect(await screen.findByText("Vela Math")).toBeVisible();
    expect(screen.getByText("A bounded accepted result.")).toBeVisible();
    expect(screen.getByText("No reviewed Problem locator")).toBeVisible();
    await user.click(screen.getByRole("tab", { name: "Execute" }));
    expect(await screen.findByText("Open exact source elsewhere")).toBeVisible();
    await user.click(screen.getByRole("button", { name: "Terminal" }));
    expect(calls.launchRepository).toHaveBeenCalledWith(snapshot.path, "terminal");
  });

  it("requires explicit native execution review and says the controls are not a sandbox", async () => {
    const user = userEvent.setup(); render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    await user.click(screen.getByRole("tab", { name: "Execute" }));
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
});
