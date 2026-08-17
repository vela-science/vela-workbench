# Vela Workbench

Vela Workbench is a thin, local-first desktop shell for exact Git and Vela state. Tranche 1 selects native repositories, orients to supported Vela state, shows Git worktrees and Entire checkpoint references, and hands exact source to a terminal, editor, or forge. It is non-authoritative and read-only with respect to source and scientific state.

## Boundary

Workbench does not own Git refs, Entire sessions, agent runtimes, Vela protocol objects, public discovery, hosted collaboration, or authority. It has no server, database, generic shell/filesystem/HTTP command, upload surface, remote WebView, provider app, or bundled Vela sidecar. The only persistent data is one Rust-owned preferences file containing clearable repository paths and the selected Vela executable path.

The renderer invokes six compiled private IPC commands. Rust canonicalizes paths, executes a fixed Git read set, verifies the selected Vela binary, parses supported JSON schemas, and generates the renderer DTO contract one-way from Rust.

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

`bun run contracts` regenerates `src/contracts/generated/ipc.ts` from Rust. The Rust test suite refuses drift between the generated file and its source.

## Security and deletion

The bundled renderer is covered by one local capability containing only the six enumerated commands and a restrictive bundled-local CSP. Child commands receive a cleared environment plus a minimal allowlist, bounded output, timeouts, and process-group termination. External handoffs accept canonical local roots or sanitized HTTPS remotes only.

Deleting Workbench and its application-data preferences changes no repository bytes, Git refs, Entire checkpoints, Vela objects, Decisions, Events, Standing, or problems.science activity.

See [ARCHITECTURE.md](ARCHITECTURE.md), [SECURITY.md](SECURITY.md), [PRODUCT.md](PRODUCT.md), and [DESIGN.md](DESIGN.md).
