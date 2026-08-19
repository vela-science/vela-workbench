# Vela Workbench product context

register: product

## Product purpose

Vela Workbench is a local desktop shell for inspecting exact Git and Vela state without absorbing the systems that own that state. It lets a researcher open native repositories, orient to current contributions, inspect worktrees and Entire checkpoint references, run one explicitly reviewed source-native profile, capture explicit evidence, prepare an ordinary Submission v3, record or import a scoped Verification, and explicitly request one attributed Repository Decision through signed Vela. Hosted authority and generic execution remain absent.

The application is useful when a researcher has private local files, multiple source repositories, and a signed Vela executable that must remain under local custody. It should make the boundaries among Git source state, Vela scientific state, Entire provenance, and external tools visible at every handoff.

## Users

- Researchers who work across one authority repository and one or more source-native repositories.
- Human and agent performers who need the same attributed, policy-governed view of current state.
- Reviewers who need exact repository, commit, tree, worktree, binary, schema, and projection facts before acting elsewhere.

Users are technically fluent. They value precision, compact density, legible refusal states, and exact provenance more than onboarding theater.

## Strategic principles

1. Native repositories stay sovereign. The Workbench reads and hands off; it does not copy branches, checkpoints, transcripts, or scientific state.
2. Core semantics stay in `vela --json`. Rust validates supported schemas and returns typed private IPC data. The renderer never parses `.vela` files or raw subprocess JSON.
3. Public discovery and shared coordination stay in problems.science. Workbench accepts only a browser-safe exact Problem continuation and never infers a Problem from local source.
4. Local privilege stays narrow. Workbench exposes only scoped Verification author/import, exact Inbox/show reads, accept/reject with the current entry root, immediate replay/readback, and exact structured recovery. After restart, recovery becomes available only when the signed `vela.recovery-inspection.v1` read of an explicitly selected recent returns one exact operation ID; Workbench never scans or chooses journals. The bounded OpenGauss pilot adds three typed operations for detection, explicit Terminal handoff, and receipt refresh. It still has no generic shell, filesystem, HTTP, upload, provider mutation, hosted signer, background authority, queue, or session store.
5. Fail closed and explain why. Missing binaries, unsupported schemas, invalid integrations, stale locators, and unavailable Entire references remain visible degraded states.
6. Deletion must be harmless. Removing the app and its clearable preferences changes no Git ref, repository byte, Vela object, Event, Decision, Standing, or Web activity record.

## Tone

Calm, exact, and operational. Labels name the owning system and the observation time. Copy avoids promotional language and never turns local facts into scientific claims. Empty states teach the next safe action. Errors state what was refused and which boundary owns recovery.

## Anti-references

- A cloud dashboard that implies freshness it did not fetch.
- A generic AI cockpit with chat, tasks, queues, agents, or session history.
- A file browser that grants the renderer broad local access.
- A public Problems catalogue or evidence feed copied into a desktop shell.
- A decorative science interface that uses graphs, gradients, or status colors without semantic ownership.

## Product surfaces

- Repository switcher: user-selected Git roots, clearable recents, local classification, branch and dirty state.
- Problem continuation: one browser-safe, versioned URL carries separate Problem, source repository/ref, authority Repository, and explicit artifact references. Rust validates it before display; a user-selected source checkout must match both remote and revision, and a separately selected authority checkout must match its remote and current Vela Repository state. Result text may remain process-local across that transition, while exact source bytes move only through a reviewed create-new export and explicit reselection in the Repository.
- Overview: exact Git/Vela facts, accepted/current Result rows, and integration non-authority labels.
- Work: worktrees, remotes, Entire checkpoint references, explicit local or HTTPS handoffs, detached worktree preview/create, and four app-reviewed command profiles. Execution is explicit and warns that repository-controlled build scripts/plugins run with the current user's privileges. The bounds are not a sandbox. There is no arbitrary argv surface.
- OpenGauss pilot within Work: choose one exact `gauss` executable, confirm the fixed `--version` probe, validate one `.gauss/project.yaml` without calling OpenGauss's mutating loader, and open Terminal at the project root. Documented workflows are shown as interactive slash commands only. Workbench never types them, observes hidden backend/model transport, or ingests OpenGauss session, swarm, prompt, transcript, memory, SQLite, or trajectory state.
- Capture: explicit local files or one completed command stream, with exact bytes, digest, size, kind, source revision, exclusions, redaction confirmation, and one-shot local export. Redacted output is always a new derived file; selected source evidence is immutable to Workbench.
- Submit: preview one ordinary Submission v3 author/import operation through the pinned signed Vela CLI. Producer attribution is displayed separately from Repository authority.
- Check & Decide: refresh one exact current Inbox; preserve Method, evidence, toolchain/environment, performer/model/provider and declared shared-dependency facets; author/import one non-authoritative Verification; then preview and explicitly confirm one attributed accept or reject with the exact `entry_root`. Actual Decision, Event and Standing readback remains visually separate from Verification outcome.

The renderer journey is Overview → Work → Capture → Submit → Check & Decide. Every mutation has an exact preview and fresh Rust-side revalidation; native confirmation is required immediately before worktree, export, Submission, Verification, Decision, or recovery mutation. A compact task orientation is derived and memory-only; it lists its included records, omissions, roots and observation time and never becomes scientific state.

After external OpenGauss work, the user explicitly selects files and checks through the existing Capture and NativeExec surfaces. The pilot receipt binds only those selections plus Git before/after. OpenGauss progress is not evidence by default, and selected external-tool provenance has `authority_effect:none`; an ordinary Submission v3, later scoped Verification, and attributed Decision remain separate.
