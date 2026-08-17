# Security boundary

## Trust model

Workbench is a privileged local desktop app for a single macOS user. Selected repositories and their Git metadata are untrusted input. The bundled renderer is less trusted than the Rust host and receives only validated DTOs. The selected Vela executable is executable input and therefore runs only when its version and exact platform hash match the pinned signed baseline.

## Controls

- One local Tauri capability grants only six app commands to the `main` window. No generic Tauri opener, shell, filesystem, HTTP, process, dialog, or upload plugin is enabled.
- CSP permits bundled self content and Tauri IPC only. Frames, objects, forms, remote scripts, and remote WebView navigation are denied.
- Repository and executable paths are canonicalized. Later commands accept only canonical roots already recorded by an explicit native selection.
- Process arguments use OS argv values, never a shell. Git uses a fixed system executable on macOS and disables optional locks, hooks, and filesystem monitors for inspection.
- Child environments are cleared. Only required locale/temp/home values and a minimal system PATH are restored; credential and signer environment variables are not forwarded.
- Output is captured with fixed byte ceilings. Every subprocess has a timeout; the isolated process group is killed so descendants cannot retain pipes.
- Vela is hashed before every JSON command and again afterward. Unsupported versions, hashes, schemas, shapes, envelope tags, or semantic invariants fail closed.
- External forge locators require HTTPS without embedded credentials; query and fragment data are removed. Local handoffs receive a canonical absolute repository path.
- Preferences deny unknown fields, contain only recents/tool choice, are atomically replaced with mode 0600, and are clearable in the UI.

## Residual assumptions

An attacker who already controls the user account can replace application memory, change repositories, or race a selected executable and is outside the desktop app's isolation guarantee. Hash-before/hash-after detection narrows executable substitution but is not a kernel-backed immutable execution handle. Git and the signed Vela runtime remain trusted dependencies within their fixed command surfaces.

Report security issues privately to the Vela maintainers. Do not include private repository contents, credentials, signer material, or scientific evidence in a report.
