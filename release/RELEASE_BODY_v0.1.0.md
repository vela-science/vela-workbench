# Vela Workbench 0.1.0 (macOS, Apple silicon)

Vela Workbench is the local continuation of a scientific Problem: choose the
exact source checkout, work with explicit local tools, capture evidence,
prepare a Result for Submission, record a scoped Check, and ask a Repository
authority for an attributed Decision through signed Vela. Workbench owns no
Git refs, sessions, protocol objects, or authority, and has no server,
telemetry, or updater.

This release is qualified for Apple-silicon macOS only. There is no Linux or
BSD distribution.

## Release identity

- Source commit: `TODO-SIGNING: git rev-parse HEAD of the released main`
- Source tree: `TODO-SIGNING: git rev-parse HEAD^{tree} of the released main`
- App version: `0.1.0`
- Signing: `TODO-SIGNING: Developer ID Application identity (name and team ID) printed by scripts/release-macos.sh`
- Notarization: App Store Connect API, submission `TODO-SIGNING: notarytool submission id from the release run log`
- Required Vela Core runtime: signed `v0.977.3`, macOS arm64 binary SHA-256
  `3a1173918bdcb887155bab681411bf5e9ff64d925fe1b50369ac37ab020b94ad`

## Assets

| Asset | SHA-256 |
| --- | --- |
| `Vela Workbench_0.1.0_aarch64.dmg` | `TODO-SIGNING: contents of the sibling .sha256 written by scripts/release-macos.sh` |
| `Vela Workbench_0.1.0_aarch64.dmg.sha256` | checksum sidecar for the DMG above |
| `Vela Workbench_0.1.0_aarch64.dmg.spdx.json` | `TODO-SIGNING: contents of its sibling .spdx.json.sha256 (SPDX SBOM of the signed DMG, syft 1.50.0)` |
| `Vela Workbench_0.1.0_aarch64.dmg.spdx.json.sha256` | checksum sidecar for the DMG SBOM above |
| `Vela Workbench_0.1.0_aarch64.source.spdx.json` | `TODO-SIGNING: contents of its sibling .spdx.json.sha256 (SPDX SBOM of the source-locked manifests, syft 1.50.0)` |
| `Vela Workbench_0.1.0_aarch64.source.spdx.json.sha256` | checksum sidecar for the source SBOM above |

Only artifacts produced by `scripts/release-macos.sh` on a clean,
remote-equal `main` checkout, with every signing, Gatekeeper, stapling, and
DMG check passed, are published here. An unsigned or locally rebuilt artifact
is a development build and is never a substitute for these assets.

## Verify before opening

```sh
shasum -a 256 -c "Vela Workbench_0.1.0_aarch64.dmg.sha256"
```

The output must read `Vela Workbench_0.1.0_aarch64.dmg: OK`. If it does not,
delete the download and fetch this release again; do not open a DMG whose
digest does not match.

Optionally confirm the notarization staple offline:

```sh
xcrun stapler validate "Vela Workbench_0.1.0_aarch64.dmg"
```

## Install

1. Open the verified DMG and drag `Vela Workbench.app` to `/Applications`.
2. Keep the DMG and its `.sha256` until the new build has passed first-run
   checks; they are your rollback artifacts.
3. Launch Workbench once from `/Applications` so macOS registers the
   `vela-workbench://` scheme.
4. On first run, select the signed Vela Core `v0.977.3` macOS arm64 binary
   with the SHA-256 above. A locally rebuilt `vela 0.977.3` is a different
   binary and is refused.
5. Start from a Problem on problems.science and choose **Continue locally**.
   Full first-run and handoff steps are in
   [INSTALL.md](https://github.com/vela-science/vela-workbench/blob/main/INSTALL.md).

## Rollback

Rollback replaces only `/Applications/Vela Workbench.app`:

1. Quit Workbench.
2. Delete `/Applications/Vela Workbench.app`.
3. Verify the retained prior DMG against its retained `.sha256`
   (`shasum -a 256 -c ...`), open it, and drag the prior
   `Vela Workbench.app` back to `/Applications`.

Rollback changes no repository bytes, Git refs, Entire checkpoints, Vela
objects, Decisions, Events, Standing, or problems.science activity. If the
preferences file is lost, reinstall, reselect the signed Vela binary, and
reselect the source and authority repositories; repository and public state
reconstruct from Git, signed Vela replay, and problems.science — not from
Workbench process memory.

## Deleting Workbench

Deleting the app and its application-data preferences file removes everything
Workbench persists. The preferences file contains clearable repository and
tool paths only.
