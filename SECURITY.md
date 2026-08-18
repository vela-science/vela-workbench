# Security boundary

## Trust model

Workbench is a privileged local desktop app for a single macOS user. Selected repositories, Git metadata, evidence files, command output, native tools, and signed envelopes are untrusted input. The bundled renderer is less trusted than the Rust host and receives only validated DTOs. The selected Vela executable is executable input and therefore runs only when its version and exact platform hash match the pinned signed baseline.

## Controls

- One local Tauri capability grants only explicitly enumerated app commands to the `main` window. No generic Tauri opener, shell, filesystem, HTTP, process, dialog, or upload plugin is enabled.
- CSP permits bundled self content and Tauri IPC only. Frames, objects, forms, remote scripts, and remote WebView navigation are denied.
- Repository and executable paths are canonicalized. Later commands accept only canonical roots already recorded by an explicit native selection.
- Process arguments use OS argv values, never a shell. Git uses a fixed system executable on macOS and disables optional locks, hooks, and filesystem monitors for inspection.
- Child environments are cleared. Only required locale/temp/home values and a minimal system PATH are restored; credential and signer environment variables are not forwarded.
- Output is captured with fixed byte ceilings. Every subprocess has a timeout; the isolated process group is killed so descendants cannot retain pipes.
- Vela is hashed before every JSON command and again afterward. Unsupported versions, hashes, schemas, shapes, envelope tags, or semantic invariants fail closed.
- Worktree creation is limited to a native-selected empty destination outside every existing checkout and one resolved commit through fixed `git worktree add --detach` argv. Rust enumerates the target tree's effective attributes and refuses any assigned `filter`, including drivers supplied through repository config or info attributes, before Git can run repository-controlled smudge/process code. Source/ref/destination preconditions are repeated after the OS-native dialog confirms the exact change and rollback command. Workbench never switches or resets the selected checkout.
- Native execution is limited to four app-reviewed profiles with fixed argv and manifest markers. A user-selected tool is canonicalized, hashed before and after use, and never receives ambient credential/signer variables. The preview and confirmation state that repository-controlled build scripts/plugins run with the current user's privileges. One process may run at a time; cancellation and timeout kill its process group. These controls limit process lifetime and capture only and are not a sandbox or security boundary. Run records are bounded and memory-only.
- Evidence selection reads one regular non-symlink repository-contained file at a time. Exact bytes and SHA-256 are bounded; command output can cross into evidence only through an explicit one-shot export. Export uses a native destination, refuses overwrite, revalidates the source digest, and requires OS-native confirmation. Redaction/exclusion creates a distinct derived output and never mutates the selected source evidence.
- Submission author/import runs only pinned `vela submit ... --json`. Rust bounds vector and string shapes before filesystem work, revalidates source revision, Artifact bytes, envelope, and Vela identity again after OS-native confirmation, and shows exact argv or envelope bytes/root. Unverified envelope text is JSON-escaped in the native dialog. The returned result must report `accepted_event_delta: 0` and `accepted_state_changed: false`; Verification and Decision commands are not compiled into IPC.
- External forge locators require HTTPS without embedded credentials; query and fragment data are removed. Local handoffs receive a canonical absolute repository path.
- Preferences deny unknown fields, contain only recents/tool choice, are atomically replaced with mode 0600, and are clearable in the UI.

## Residual assumptions

An attacker who already controls the user account can replace application memory, approve native confirmations, change repositories, or race selected files and executables and is outside the desktop app's isolation guarantee. Hash-before/hash-after detection narrows substitution but is not a kernel-backed immutable handle. Git, selected native tools, and signed Vela remain trusted only within their fixed command surfaces. A source-native command may intentionally mutate its working tree; the preview names this boundary and Workbench never describes execution as no-mutation. The open GLib advisory remains excluded from the qualified macOS dependency graph and blocks any future Linux/BSD package until the supported Tauri stack reaches GLib 0.20 or later and is requalified.

Report security issues privately to the Vela maintainers. Do not include private repository contents, credentials, signer material, or scientific evidence in a report.
