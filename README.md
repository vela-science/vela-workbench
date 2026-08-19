# Vela Workbench

Vela Workbench is the local continuation of a scientific Problem. It lets a researcher choose the exact source checkout, work with explicit local tools, capture evidence, prepare a Result for Submission, record a scoped Check, and ask a Repository authority for an attributed Decision through signed Vela. Git owns source, Entire owns sessions/checkpoints, problems.science owns shared discovery and coordination, and Vela Core owns scientific-state semantics and Repository authority.

## Boundary

Workbench does not own Git refs, Entire sessions, agent runtimes, Vela protocol objects, public discovery, hosted collaboration, or authority. It has no server, database, generic shell/filesystem/HTTP command, upload surface, remote WebView, provider app, or bundled Vela sidecar. The only persistent data is one Rust-owned preferences file containing clearable repository paths and selected tool paths; command output and selected evidence remain process-local and clear on exit or Clear recents.

The renderer invokes a closed set of compiled private IPC commands. Rust canonicalizes paths, fixes command-profile arguments, bounds environment/output/lifetime/process-tree capture, verifies the selected Vela binary, parses supported JSON schemas, and generates the renderer DTO contract one-way from Rust. Native execution is not a sandbox: repository-controlled build scripts and plugins run with the current user's privileges, only after an exact preview and explicit initiation. The OpenGauss probe has the same current-user warning; the later Terminal session is owned by Terminal and OpenGauss and is neither observed nor constrained by Workbench.

## Frozen foundation

- Tauri Rust crate `2.11.5`, Tauri build crate `2.6.3`
- Tauri CLI `2.11.4`, Tauri API `2.11.1`
- Rust `1.97.1`, edition 2024
- Bun lockfile with React `19.2.8`, TypeScript `5.8.3`, Vite `7.3.6`, Tailwind `4.3.3`, Base UI `1.7.0`
- Signed Core release `v0.977.3`, commit `1c1abe8f365f16803fea889bf9280877992a6d02`, tree `66bb4cb5173ff50beeef45c03fa11060e1e9e377`, Protocol 1 manifest root `sha256:534120b2dfcfb8357d7f20be3e3a0e0dfbe21d0cd51082edb5dea686ca82ec85`
- macOS arm64 runtime SHA-256 `3a1173918bdcb887155bab681411bf5e9ff64d925fe1b50369ac37ab020b94ad`; release manifest SHA-256 `b0da2d3e9a5d896fdfad96fe096c9caee5c66e54ed713bd0a0c408e14a1b65a0`
- Linux x86_64 runtime SHA-256 `89e5f366db5480a011c722bdc7d3c7f09e07fe78c0cd2855d2e53d3a419520a0`; release manifest SHA-256 `3308d8867575f3703070b93f9a664a2e35f98bc1aac5a060efa4d3a5d1ae9b72`
- WB-OPENGAUSS-01 observed against `math-inc/OpenGauss` version `0.2.2`, commit `f87633900ae185b8037bf451a914fe7eeae1eb08`, tree `aa3768f7cf5dd06d01a972bc8ed789f7b43246fb`; OpenGauss is never bundled

The interface fixtures and executable gating name the same immutable signed v0.977.3 release. Each supported build accepts exactly one platform binary hash and one runtime version; earlier releases are not alternate paths.

## Continue a Problem locally

An installed Workbench accepts one browser-safe custom URL:

```text
vela-workbench://continue?v=1&problem=https%3A%2F%2Fproblems.science%2Fproblems%2Ferdos-problems%2F94&source=https%3A%2F%2Fgithub.com%2Fvela-science%2Flean-proofs&ref=0123456789012345678901234567890123456789&repository=https%3A%2F%2Fgithub.com%2Fvela-science%2Fmath&artifact=Erdos%2FProblem94.lean
```

The handoff carries only an exact `https://problems.science/problems/...` URL, an HTTPS source repository, a full Git object ID, an HTTPS authority Repository, and up to 32 explicit source-relative artifact references. Rust rejects unknown fields, credentials, query-bearing repository locators, symbolic refs, traversal, duplicates, and oversized input. Workbench requires the researcher to select the source checkout and separately matches its fetch remote and HEAD. It then requires a second explicit selection for the authority Repository, matching its fetch remote and current Vela Repository state without inferring Decision authority. A mismatch is never marked ready and Workbench never automatically clones, switches, uploads, executes, authenticates, or moves evidence. The requested ref is prefilled only for the source detached-worktree preview.

Result text may remain in process memory while the researcher continues from the selected source to the separate Repository. Exact source bytes do not follow it: the researcher must use the existing reviewed export to create an explicit copy inside the Repository and select that copy there before Submission. The source and Repository paths remain separately visible throughout the handoff. After a committed Decision, Workbench hands ordinary Git publication back to the researcher and returns to the exact Problem for public Results/History verification; it never pushes or treats local state as a fresh projection.

## Development

Prerequisites are Bun and the checked-in Rust toolchain. From the repository root:

```sh
bun install --frozen-lockfile
bun run check
bun run tauri build
```

`bun run contracts` regenerates `src/contracts/generated/ipc.ts` from Rust. The Rust test suite refuses drift between the generated file and its source. Packages are qualified on macOS only; the open GLib dependency alert prevents claiming a Linux/BSD distribution until the supported Tauri stack can move to GLib 0.20 or later and is fully requalified.

For a researcher install, first run, signed/notarized macOS release, clean-account check, rollback, and reconstruction, use [INSTALL.md](INSTALL.md). The bounded four-role external run and content-free evidence checklist are in [PILOT.md](PILOT.md). `bun run tauri build` alone produces a development artifact; `bun run release:macos` is the fail-closed distribution entry point.

## Security and deletion

The bundled renderer is covered by one local capability containing only the enumerated commands and a restrictive bundled-local CSP. Native command profiles expose their executable identity, fixed arguments, working directory, bounded environment, limits, and current-user privilege warning before Start. Cancellation and timeout bound lifetime and captured output; they are not security isolation. External handoffs accept canonical local roots or sanitized HTTPS remotes only.

Deleting Workbench and its application-data preferences changes no repository bytes, Git refs, Entire checkpoints, Vela objects, Decisions, Events, Standing, or problems.science activity.

See [ARCHITECTURE.md](ARCHITECTURE.md), [SECURITY.md](SECURITY.md), [PRODUCT.md](PRODUCT.md), and [DESIGN.md](DESIGN.md).
