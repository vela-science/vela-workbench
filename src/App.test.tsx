import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { BootstrapDto, RepositorySnapshotDto } from "./contracts/generated/ipc";

const bootstrap: BootstrapDto = {
  preferences: { recent_repositories: [], vela_binary_path: "/usr/local/bin/vela" },
  runtime: {
    interface_commit: "3bfcf23f12fb6a38a924a257ba25ad3d8594dc78",
    interface_tree: "ab85ef6ec7f6cd7c49fc4664bbbbd4f597e71816",
    runtime_version: "vela 0.977.0",
    runtime_commit: "00d567c879138733ba22949efc985b54578c148b",
    runtime_sha256: "4332427789bf3dac83ebad9843670047b448f6ba370661f48a0100cbb61bc00c",
    read_only: true,
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
    binary: { path: "/usr/local/bin/vela", version: "vela 0.977.0", sha256: bootstrap.runtime.runtime_sha256, state: "signed_runtime_baseline" },
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
}));

vi.mock("./lib/workbench", () => ({ workbench: calls }));
import App from "./App";

describe("Vela Workbench Tranche 1", () => {
  afterEach(cleanup);
  beforeEach(() => {
    vi.clearAllMocks();
    calls.bootstrap.mockResolvedValue(bootstrap);
    calls.selectRepository.mockResolvedValue(snapshot);
    calls.launchRepository.mockResolvedValue({ target: snapshot.path, owner: "Terminal" });
  });

  it("starts with a local-only repository choice and runtime boundary", async () => {
    render(<App />);
    expect(await screen.findByText("Start from sovereign source")).toBeVisible();
    expect(screen.getByText("Private files and credentials stay local.")).toBeVisible();
    expect(await screen.findByText("vela 0.977.0")).toBeVisible();
  });

  it("orients from a selected repository and exposes only exact handoffs", async () => {
    const user = userEvent.setup();
    render(<App />);
    const choices = await screen.findAllByRole("button", { name: "Choose repository" });
    await user.click(choices[choices.length - 1]);
    expect(await screen.findByText("Vela Math")).toBeVisible();
    expect(screen.getByText("A bounded accepted result.")).toBeVisible();
    expect(screen.getByText("No reviewed Problem locator")).toBeVisible();
    await user.click(screen.getByRole("tab", { name: "Execute / Source" }));
    expect(await screen.findByText("Open exact source elsewhere")).toBeVisible();
    await user.click(screen.getByRole("button", { name: "Terminal" }));
    expect(calls.launchRepository).toHaveBeenCalledWith(snapshot.path, "terminal");
  });
});
