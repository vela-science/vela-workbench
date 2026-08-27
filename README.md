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
- Signed Core release `v0.977.6`, commit `9ac8e7730bfb63a3b8eb1d2e1d91081c3e703c59`, tree `1332713f627ac73c235e4f9a7afe206499717154`, signed tag object `4a562d4529f6a329d938fc427bc73c4cbff90767`, Protocol 1 manifest root `sha256:bf1ef68165bccbc4d2e8a854f78c70448cc7de771bac23329f7a8ca115303f56`
- macOS arm64 archive SHA-256 `62ea9006e086b40f0431b2ce2cf74827518f37dc58e329353920083f50dad874`; runtime SHA-256 `5b21415c98503b20518c0e68714b0b4f4b3c371525ea110563b89a53a0d3dbb3`; release manifest SHA-256 `596273b718661899ad10cb65d82c8c0d92240939899e72042180ef4912acfa2c`; manifest signature SHA-256 `f4bbfe43dd3528b9a3a2de6f5efd00a7e1585aa1d813cbe09841bf35a42d123b`
- Linux x86_64 archive SHA-256 `a8cb120a01211fbb40b5da6d697b0fc8e4a84b0d76e62cfa574a2518bdebb83e`; runtime SHA-256 `e476ece52cb5f356519f890533f06c918fb10f3dd00268092d490701f7fd1b65`; release manifest SHA-256 `495626ddf9ca286ffbc173df268edef73ae44b0a87f7f2b46d8fdbcdf38d8a25`; manifest signature SHA-256 `8e0481f1e7aab844584cc7db414ca567e2c7b7c03f8e9aabf98a38e164565272`
- Both release manifests verify as `release@vela.space`, namespace `vela-release`, key fingerprint `SHA256:MX3Eo1o9S5pLnx2kiNyAy2aME7PAWDtvqtUBljJst1M`, using the namespace-scoped `allowed_signers` at the exact Core release commit; `allowed_signers` SHA-256 `dc471fc1ff1960879f39cc52cbe46b87142e1ccfb3b4d567eaae9ac4d26d0d10`
- WB-OPENGAUSS-01 observed against `math-inc/OpenGauss` version `0.2.2`, commit `f87633900ae185b8037bf451a914fe7eeae1eb08`, tree `aa3768f7cf5dd06d01a972bc8ed789f7b43246fb`; OpenGauss is never bundled

Executable gating and the current release fixtures name the same immutable signed v0.977.6 release. Each supported build accepts exactly one platform binary hash and one runtime version; earlier releases are not alternate runtime paths. The frozen v0.977.3 interface fixtures remain compatibility and historical evidence and are never rewritten by a repin.

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
