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
- Merged Core interface target `3bfcf23f12fb6a38a924a257ba25ad3d8594dc78`, tree `ab85ef6ec7f6cd7c49fc4664bbbbd4f597e71816`
- Executable runtime baseline `vela 0.977.0`, commit `00d567c879138733ba22949efc985b54578c148b`, macOS arm64 SHA-256 `4332427789bf3dac83ebad9843670047b448f6ba370661f48a0100cbb61bc00c`

The runtime-interface gap is explicit: UI and fixtures target merged Core main, but subprocess execution remains pinned to the signed v0.977.0 binary. A later signed compatible Core release must carry the reviewed merged interface before Workbench can update its runtime version, commit, binary hash, and frozen fixtures together.

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
