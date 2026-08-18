# Vela Workbench

Vela Workbench is a thin, local-first desktop shell for exact Git and Vela state. Tranche 2 preserves the read-only Orient surface and adds four bounded local actions: detached worktree creation, app-reviewed source-native command profiles, explicit evidence capture/export, and ordinary Submission v3 author/import through the signed Vela CLI. It remains non-authoritative: Git owns source, Entire owns sessions/checkpoints, Problems owns shared coordination/presentation, and Vela Core owns scientific-state semantics.

## Boundary

Workbench does not own Git refs, Entire sessions, agent runtimes, Vela protocol objects, public discovery, hosted collaboration, or authority. It has no server, database, generic shell/filesystem/HTTP command, upload surface, remote WebView, provider app, or bundled Vela sidecar. The only persistent data is one Rust-owned preferences file containing clearable repository paths and selected tool paths; command output and selected evidence remain process-local and clear on exit or Clear recents.

The renderer invokes a closed set of compiled private IPC commands. Rust canonicalizes paths, fixes command-profile arguments, bounds environment/output/lifetime/process-tree capture, verifies the selected Vela binary, parses supported JSON schemas, and generates the renderer DTO contract one-way from Rust. Native execution is not a sandbox: repository-controlled build scripts and plugins run with the current user's privileges, only after an exact preview and explicit initiation.

## Frozen foundation

- Tauri Rust crate `2.11.5`, Tauri build crate `2.6.3`
- Tauri CLI `2.11.4`, Tauri API `2.11.1`
- Rust `1.97.1`, edition 2024
- Bun lockfile with React `19.2.8`, TypeScript `5.8.3`, Vite `7.3.6`, Tailwind `4.3.3`, Base UI `1.7.0`
- Signed Core release `v0.977.1`, commit `0e057c0debcff775a3deb56150ceaccfd4707b41`, tree `55768612b82b93a4a01bb5aeddeb937dff678e4a`, Protocol 1 manifest root `sha256:448e7e80ac1ead40045d87df51f4352e80091c09ceae6e5acea250795b5ff9ed`
- macOS arm64 runtime SHA-256 `a4f5594b2777b265f6d58296cc8e9efd85d0a72c82b49c0fce4805438ed46948`; release manifest SHA-256 `6942a9f215909a55e849c7722a49ea961ef6259294c9f8ca36f944d6fd88d884`
- Linux x86_64 runtime SHA-256 `3c25344f2a636a803d82fd7cf663e5638778d1121198301f478ff3dcc18f0270`; release manifest SHA-256 `6cd034646b57b1c5e8c3d85a95a882a3221f7277c543f6bdde69fc9757b423e4`

The prior merged-interface/runtime split is closed: interface fixtures and executable gating now name the same immutable signed v0.977.1 release. Each supported build accepts exactly one platform binary hash and one runtime version; v0.977.0 is not an alternate path.

## Development

Prerequisites are Bun and the checked-in Rust toolchain. From the repository root:

```sh
bun install --frozen-lockfile
bun run check
bun run tauri build
```

`bun run contracts` regenerates `src/contracts/generated/ipc.ts` from Rust. The Rust test suite refuses drift between the generated file and its source. Tranche 2 packages are qualified on macOS only; the open GLib dependency alert prevents claiming a Linux/BSD distribution until the supported Tauri stack can move to GLib 0.20 or later and is fully requalified.

## Security and deletion

The bundled renderer is covered by one local capability containing only the enumerated commands and a restrictive bundled-local CSP. Native command profiles expose their executable identity, fixed arguments, working directory, bounded environment, limits, and current-user privilege warning before Start. Cancellation and timeout bound lifetime and captured output; they are not security isolation. External handoffs accept canonical local roots or sanitized HTTPS remotes only.

Deleting Workbench and its application-data preferences changes no repository bytes, Git refs, Entire checkpoints, Vela objects, Decisions, Events, Standing, or problems.science activity.

See [ARCHITECTURE.md](ARCHITECTURE.md), [SECURITY.md](SECURITY.md), [PRODUCT.md](PRODUCT.md), and [DESIGN.md](DESIGN.md).
