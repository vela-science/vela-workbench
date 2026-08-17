# Architecture

## Runtime shape

The application is a single Tauri 2 desktop binary. A React renderer presents a repository switcher and the Orient → Execute → Capture → Review draft journey. The renderer has no filesystem, process, network, signer, or provider API. Private Tauri commands cross into four Rust-owned privilege families in addition to the Tranche 1 observation and handoff commands:

| Family | Commands | Contract | Effect |
| --- | --- | --- | --- |
| Observation | `bootstrap`, `select_repository`, `inspect_repository`, `select_vela_binary`, `clear_recents` | canonical selected recents and one pinned signed Vela runtime | reads source/preferences; clears choices |
| Handoff | `launch_repository` | fixed receiving-app enum and sanitized canonical path/HTTPS locator | explicit external open |
| Worktree | `preview_worktree`, `create_worktree` | selected repository, native-selected empty destination, validated ref resolved to one commit, repeated source preconditions | creates one detached Git worktree after native confirmation |
| Native execution | `select_native_tool`, `preview_native_exec`, `run_native_exec`, `cancel_native_exec` | one of `git_diff_check`, `lean_build`, `cargo_test`, or `bun_test`; fixed argv; exact executable digest; one active bounded process | runs or cancels one reviewed source-native command |
| Evidence | `select_evidence_file`, `preview_evidence_export`, `export_evidence` | explicit regular file or completed command stream; bounded exact bytes; digest/size/source revision; native destination | reads one item; creates one new local export after native confirmation |
| Submission | `preview_submission_draft`, `submit_submission_draft`, `select_submission_import`, `import_submission` | exact v3 fields or exact signed envelope; current file digests; exact CLI argv; pinned Vela hash | producer-authenticated pending Submission/Proposal mutation after native confirmation |

There is no public IPC protocol. `src-tauri/src/contracts/mod.rs` is the source of truth for private DTOs; `src-tauri/src/bin/export-bindings.rs` generates the TypeScript types. Preview DTOs are explanatory, not bearer authority: every mutating command rebuilds the plan from typed inputs, repeats executable/file/source checks, and displays an OS-native confirmation.

## Ports

- Git: fixed observation porcelain plus one `git worktree add --detach <destination> <resolved-commit>` mutation. Ref resolution, destination selection, preview, repeated preconditions, and rollback guidance are Rust-owned. The selected current checkout is never switched or reset.
- NativeExec: app-owned profiles map to an exact executable and fixed argv. Selected tools are regular executable files whose canonical path, SHA-256, size, and platform identity are shown. Every preview also shows cwd, the bounded environment, and a warning that repository-controlled build scripts/plugins execute with the current user's local privileges. Environments are cleared and rebuilt from a documented non-secret allowlist; stdout/stderr, time, concurrency, and the full process group are bounded. These are lifetime and capture controls, not a sandbox or security isolation. Completed output is ephemeral memory only.
- Evidence: regular non-symlink files must be explicitly selected inside the selected repository; command streams come only from the ephemeral completed-run map. Exact bytes are bounded and returned as UTF-8 plus base64 identity where applicable. Export uses a native destination, create-new semantics, digest recheck, and native confirmation. Any exclusion/redaction operation creates a distinct derived output and never edits the selected source evidence. No directory traversal, repository scan, transcript copy, or background transfer exists.
- Vela: a user-selected executable must match signed v0.977.1 and its exact macOS arm64 hash. Draft authoring and signed-envelope import use only `vela submit ... --json`. Rust parses and validates `vela.submit-result.v1`, including zero accepted-event delta and no accepted-state change. The CLI retains signer custody and Repository policy remains separate from producer attribution.
- Entire: Tranche 1 only reports CLI availability and checkpoint references already present in Git. Entire owns session and checkpoint content.
- Handoff: fixed launch enums map to macOS `open` or a sanitized HTTPS URL derived from a fetch remote. The receiving app owns all later actions.
- Problems: no catalogue, search, or inferred mapping exists. Only an exact reviewed URL may be handed off in a future compatible surface.

## Offline and degraded operation

Git-only orientation, worktree preview, native execution, capture, and local export work without network access. Missing or unrecognized Vela disables Submission preview/import and produces an explicit refusal. Missing a selected native tool degrades only its profile. A stale source/ref/file/tool digest invalidates its preview. Cancellation terminates the isolated process group and records no durable run. Missing Entire produces no replacement store. Forge handoff can open offline but freshness and access belong to the browser/provider.

## Explicitly absent

No Verification, accept/reject Decision, Repository-authority action, current-checkout switch/reset, arbitrary command/argv/environment, generic filesystem or HTTP API, upload, agent coordination, runner service, queue, durable session/run store, server, database, provider SDK, updater, release pipeline, telemetry, Linux/BSD distribution, or new Protocol object exists in Tranche 2.
