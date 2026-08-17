# Architecture

## Runtime shape

The application is a single Tauri 2 desktop binary. A React renderer presents three surfaces: repository switcher, Orient, and Execute / Source. The renderer has no filesystem or shell APIs. Six private Tauri commands cross into the Rust host:

| Command | Inputs | Output | Effect |
| --- | --- | --- | --- |
| `bootstrap` | none | preferences and pinned runtime policy | reads preferences |
| `select_repository` | native folder choice | repository snapshot or cancel | reads source; records canonical recent |
| `inspect_repository` | previously selected path | repository snapshot | reads source |
| `select_vela_binary` | native file choice | verified binary identity or cancel | hashes/version-checks; records tool choice |
| `clear_recents` | none | empty preferences | clears repository and tool choices |
| `launch_repository` | selected path and fixed launch enum | handoff receipt | explicitly opens terminal, editor, or HTTPS forge |

There is no public IPC protocol. `src-tauri/src/contracts/mod.rs` is the source of truth for private DTOs; `src-tauri/src/bin/export-bindings.rs` generates the TypeScript types.

## Ports

- Git: a fixed read-only porcelain set observes root, status, worktrees, remotes, commits, trees, and `Entire-Checkpoint` trailers. Git owns every returned source fact.
- Vela: a user-selected executable must match the pinned signed version and platform hash. Rust parses typed supported envelopes and validates semantic invariants. Vela owns scientific state.
- Entire: Tranche 1 only reports CLI availability and checkpoint references already present in Git. Entire owns session and checkpoint content.
- Handoff: fixed launch enums map to macOS `open` or a sanitized HTTPS URL derived from a fetch remote. The receiving app owns all later actions.
- Problems: no catalogue, search, or inferred mapping exists. Only an exact reviewed URL may be handed off in a future compatible surface.

## Offline and degraded operation

Git-only inspection works without Vela, Entire, or network access. Missing or unrecognized Vela produces an explicit refusal and no scientific interpretation. A valid native integration is displayed with `authority_effect: none`. Missing Entire produces no replacement store. Forge handoff can open offline but freshness and access belong to the browser/provider.

## Explicitly absent

No Core write command, Submission preparation, Verification, Decision, signer use, source-native execution, upload, HTTP client, agent coordination, generic command runner, work queue, session store, server, database, provider SDK, updater, release pipeline, or telemetry exists in Tranche 1.
