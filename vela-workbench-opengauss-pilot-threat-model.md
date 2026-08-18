# WB-OPENGAUSS-01 threat and capability contract

Status: frozen before privileged implementation
Base: `44fb8d26b989b81948150b185add7f1c2df98b8a`
OpenGauss source observation: `math-inc/OpenGauss` commit
`f87633900ae185b8037bf451a914fe7eeae1eb08`, tree
`aa3768f7cf5dd06d01a972bc8ed789f7b43246fb`, version `0.2.2`

## Product boundary

This pilot is one explicit local handoff to an external OpenGauss installation.
Workbench verifies one user-selected executable and one selected Repository's
`.gauss/project.yaml`, opens a terminal at the exact project root, and later
compares exact Git identity. It does not run a Lean workflow, speak ACP or MCP,
read OpenGauss session databases, inspect swarm state, mirror prompts or
transcripts, or turn progress into Vela Verification or Standing.

OpenGauss, Codex, or Claude executes work. Entire owns generic session and Git
checkpoint provenance. Workbench retains no durable execution session and only
captures files or command output that the user later selects through the
existing evidence ports. An ordinary Submission v3 is the first Vela object;
OpenGauss provenance has `authority_effect: none`.

## Observed upstream interface and design inference

Observed in the exact primary source above:

- `gauss --version` / `gauss version` reports the installed version.
- `.gauss/project.yaml` schema version 1 identifies one Lean 4 project. The
  manifest contains `name`, `kind`, `lean_root`, optional source/blueprint
  metadata, and runtime/cache/workflow paths.
- OpenGauss discovers the manifest upward and exposes `/project` plus
  `/prove`, `/draft`, `/review`, `/checkpoint`, `/refactor`, `/golf`,
  `/autoprove`, `/formalize`, and `/autoformalize` inside its interactive CLI.
- Current public shell subcommands expose generic `gauss` / `gauss chat`, not a
  stable non-interactive machine contract for those lifted workflow commands.
- OpenGauss owns its backend selection, child sessions, swarm state, recovery,
  SQLite/session state, tools, and hidden model transport.

Design inference: Workbench may list the documented interactive workflow names
and open the exact project in Terminal, but must not synthesize slash-command
input or call internal Python APIs. The exact handoff command is only the
selected executable with no arguments. The user chooses and initiates every
workflow inside OpenGauss. Backend/model/transport remain “not exposed at this
boundary” unless OpenGauss itself shows them after launch.

Primary sources:

- <https://github.com/math-inc/OpenGauss>
- <https://github.com/math-inc/OpenGauss/blob/f87633900ae185b8037bf451a914fe7eeae1eb08/README.md>
- <https://github.com/math-inc/OpenGauss/blob/f87633900ae185b8037bf451a914fe7eeae1eb08/website/docs/reference/cli-commands.md>
- <https://github.com/math-inc/OpenGauss/blob/f87633900ae185b8037bf451a914fe7eeae1eb08/website/docs/reference/slash-commands.md>
- <https://github.com/math-inc/OpenGauss/blob/f87633900ae185b8037bf451a914fe7eeae1eb08/gauss_cli/project.py>

## Exact private capability

Add only three typed Tauri commands to the existing local `main` capability:

- `select_opengauss`
- `launch_opengauss_handoff`
- `refresh_opengauss_handoff`

There is no generic executable, argv, shell, PTY, terminal-input, filesystem,
HTTP, ACP, MCP, upload, session, swarm, database, provider, or Vela command.
The existing evidence, NativeExec, Submission v3, and terminal handoff ports are
reused without broadening their contracts.

## Executable inspection

The file dialog is explicit selection, but selecting a path is not enough to
authorize execution. Rust first canonicalizes the selected regular executable,
requires the basename `gauss`, and hashes its exact bytes. Before running the
fixed `--version` probe, an OS-native confirmation shows:

- canonical path, pre-execution SHA-256 and size;
- fixed argv `--version`, selected Repository cwd, cleared bounded environment,
  output/time/process-tree bounds;
- a warning that the selected executable runs with the current user's
  privileges, may read local OpenGauss configuration/auth and perform its own
  network/update checks, and is not sandboxed or isolated.

The probe accepts only a bounded first line matching `Gauss v0.2.2 (...)`, then
rehashes the executable and requires exact identity. Probe output is bounded and
does not become scientific evidence automatically. A fresh selection and native
confirmation are required for every handoff; no successful settings are replayed.

## Project inspection

Rust reads only the exact selected Repository's `.gauss/project.yaml`; it never
calls OpenGauss's Python loader because that loader creates runtime/cache/workflow
directories. The manifest must be one contained, regular, non-symlink file under
the canonical Git root, no larger than 64 KiB, valid YAML mapping, and schema
version 1. `name` and optional metadata are bounded control-free text; `kind`
must be `lean4`; `lean_root` and configured paths must be normalized relative
paths resolving inside the Repository. The declared Lean root must contain a
regular `lakefile.lean` or `lakefile.toml`.

The renderer receives the manifest path, SHA-256, size, schema/name/kind,
canonical Lean root, bounded source/blueprint summaries, and exact Git identity.
It does not receive the manifest bytes, OpenGauss runtime/cache/workflow contents,
global config, secrets, sessions, prompts, transcripts, memories, or trajectories.

## Handoff and receipt

Immediately before launch, Rust rehashes the selected executable, reparses the
manifest, and takes two identical Git snapshots. A one-shot selected-tool token
must match the preview. Workbench then opens Terminal at the exact project root
through its existing fixed `/usr/bin/open` port. It does not execute `gauss` or
type a slash command. The visible receipt binds:

- executable path, `Gauss v0.2.2` identity, digest, and size;
- project manifest path/root, project and Lean roots, upstream source commit;
- cwd, displayed interactive command boundary `[<selected gauss path>]`, and the
  cleared bounded `/usr/bin/open` launcher environment; the later Terminal shell
  environment is owned by Terminal and is unobserved/unconstrained;
- documented interactive workflow entrypoints and the explicit boundary that
  no workflow argv was automated;
- backend/model/transport as unobserved at this boundary;
- Git commit/tree/dirty state before external work.

Refresh consumes no OpenGauss state. It revalidates Repository/project/executable
identity, records Git commit/tree/dirty state after the external handoff, and may
bind only files or check receipts the user explicitly selected through the
existing Capture and NativeExec surfaces. OpenGauss progress/checkpoints are
never rendered as Verification, Decision, Event, or Standing.

## Falsifiers and gates

- arbitrary executable basename, symlink substitution, hash change, unsupported
  version, cancelled probe, timeout, or unbounded output fails closed;
- manifest symlink, oversized YAML, duplicate/unknown structure that changes the
  closed parse, absolute/traversing/symlink-escaping path, missing Lean marker, or
  Repository/Git change during inspection fails closed;
- forged renderer preview without the current one-shot Rust selection cannot
  open a terminal;
- launch never runs OpenGauss, injects slash commands, inherits ambient secrets,
  or reads OpenGauss state directories;
- refresh records Git identity only and cannot ingest a transcript/session DB;
- capability/CSP snapshots prove exactly three added handlers and no generic IPC;
- existing Math/lean-proofs no-mutation, deletion, fixtures, Rust/UI/build, and
  macOS packaged gates remain green;
- one disposable non-authoritative Lean worktree proves manifest inspection and
  terminal handoff without a Vela Decision or claim of external adoption.

## Residuals and later evaluation

Same-user executable/filesystem TOCTOU cannot be eliminated completely. The
selected OpenGauss process, once the user starts it in Terminal, has the user's
full privileges and may access files, credentials, and network; Workbench's
probe and handoff controls are bounds and provenance disclosure, never a sandbox.
Workbench cannot observe hidden backend/model transport or prove what occurred
outside its selected Git/files/check outputs.

Time-frozen epistemic replay is a later evaluation program: freeze exact state at
`t0`, hide later evidence, capture a proposed transition, and score it against
protected evidence arriving at `t1`. It creates no Protocol object, repository,
or provider mutation in this pilot.
