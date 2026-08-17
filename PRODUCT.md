# Vela Workbench product context

register: product

## Product purpose

Vela Workbench is a local desktop shell for inspecting exact Git and Vela state without absorbing the systems that own that state. It lets a researcher open several native repositories, orient to current accepted contributions, inspect worktrees and Entire checkpoint references, and hand exact source to an editor, terminal, or forge. Tranche 1 is strictly read-only.

The application is useful when a researcher has private local files, multiple source repositories, and a signed Vela executable that must remain under local custody. It should make the boundaries among Git source state, Vela scientific state, Entire provenance, and external tools visible at every handoff.

## Users

- Researchers who work across one authority repository and one or more source-native repositories.
- Human and agent performers who need the same attributed, policy-governed view of current state.
- Reviewers who need exact repository, commit, tree, worktree, binary, schema, and projection facts before acting elsewhere.

Users are technically fluent. They value precision, compact density, legible refusal states, and exact provenance more than onboarding theater.

## Strategic principles

1. Native repositories stay sovereign. The Workbench reads and hands off; it does not copy branches, checkpoints, transcripts, or scientific state.
2. Core semantics stay in `vela --json`. Rust validates supported schemas and returns typed private IPC data. The renderer never parses `.vela` files or raw subprocess JSON.
3. Public discovery and shared coordination stay in Problems/Web. The Workbench shows only an exact reviewed Problem locator when one is supplied.
4. Local privilege stays narrow. Tranche 2 adds only reviewed native execution, explicit evidence selection/export, detached worktree creation, and producer-authenticated Submission intake. It still has no generic shell, filesystem, HTTP, upload, provider mutation, Verification, Decision, or authority action.
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

## Tranche 2 surfaces

- Repository switcher: user-selected Git roots, clearable recents, local classification, branch and dirty state.
- Orient: exact Git/Vela facts, accepted/current Claim rows, integration non-authority labels, reviewed Problem handoff only.
- Execute / Source: worktrees, remotes, Entire checkpoint references, explicit local or HTTPS handoffs, detached worktree preview/create, and four app-reviewed command profiles. There is no arbitrary argv surface.
- Capture: explicit local files or one completed command stream, with exact bytes, digest, size, kind, source revision, exclusions, redaction confirmation, and one-shot local export.
- Review draft: preview one ordinary Submission v3 author/import operation through the pinned signed Vela CLI. Producer attribution is displayed separately from Repository authority. Verification and Decision are absent.

The renderer journey is Orient → Execute → Capture → Review draft. Every privilege-changing step has a preview and a fresh Rust-side revalidation; native confirmation is required immediately before a worktree, export, or Submission mutation.
