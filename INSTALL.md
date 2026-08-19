# Install and release Vela Workbench on macOS

Workbench is currently qualified for Apple-silicon macOS. It does not bundle Vela Core, repository credentials, an updater, or a signer. Scientific source stays in Git repositories, and Vela Repository state stays in its authority repository.

## Researcher installation

Install a signed and notarized Workbench DMG from its recorded release location, verify the published SHA-256 before opening it, drag `Vela Workbench.app` to `/Applications`, and keep the prior DMG until the new build has passed first-run checks. A raw `bun run tauri build` artifact is a development build, not a distributable release.

Workbench also requires the signed Vela Core `v0.977.3` macOS arm64 binary. The accepted binary SHA-256 is:

```text
3a1173918bdcb887155bab681411bf5e9ff64d925fe1b50369ac37ab020b94ad
```

On first run:

1. Choose that exact Vela binary. A locally rebuilt `vela 0.977.3` is a different binary and is refused.
2. Start at a Problem on [problems.science](https://problems.science), choose **Continue locally**, and approve the `vela-workbench://` handoff.
3. Select the source checkout. Workbench requires the exact fetch remote and revision; it never clones or switches the checkout.
4. Select the authority Repository separately. Workbench requires its exact remote and current Vela Repository state; this does not grant Decision authority.
5. Work and capture evidence in the source repository. Export only explicitly selected bytes, then reselect that copy inside the authority Repository before Submission.

If the custom URL does not open, launch Workbench once from `/Applications` so macOS registers the scheme, then use **Continue locally** again. If the signed runtime, Git checkout, or provider is unavailable, Workbench keeps the refusal visible; it does not substitute cached scientific state.

## Release operator path

The repository-owned entry point is `scripts/release-macos.sh`. It refuses any checkout except clean, remote-equal `main`. It validates credential presence and the API-key path and never prints or stores credential values. Apple variables are removed before dependency installation, qualification, frontend hooks, and the locked Cargo build; only the final Tauri bundling/notarization process receives them.

The Mac must have a valid `Developer ID Application` certificate and private key in its keychain. Set its complete displayed name:

```sh
export APPLE_SIGNING_IDENTITY='Developer ID Application: Example Name (TEAMID)'
```

Provide an App Store Connect API key for notarization:

```sh
export APPLE_API_ISSUER='issuer-id'
export APPLE_API_KEY='KEYID'
export APPLE_API_KEY_PATH='/absolute/path/AuthKey_KEYID.p8'
```

Run the fail-fast preflight, then the full release:

```sh
bun run release:macos:check
bun run release:macos
```

The full path installs locked dependencies, runs the complete local qualification, performs a locked unsigned Cargo build without Apple variables, then exposes the signing identity and API credentials only to Tauri's `.app`/`.dmg` bundling and notarization step. It rechecks the exact source commit, tree, clean state, and remote equality after each build stage. `codesign`, Gatekeeper, stapler, DMG integrity, the signer identity/team, and the app mounted from the final DMG must all pass before it writes a sibling `.sha256`. It then emits two SPDX SBOM sidecars next to the DMG — one for the signed DMG and one for the source-locked manifests — each with its own `.sha256`, using the pinned syft version and refusing when syft is absent or different. It does not publish, create a GitHub release, or change provider configuration.

Before publication, install the DMG in a clean macOS user account and prove: Gatekeeper launch; first-run runtime selection; one problems.science handoff; exact source and authority matching; app restart; and return to the public Problem. Record the source commit/tree, DMG digest, Apple notarization result, test account/macOS version, and problems.science deployment. Publishing those exact bytes is a separate attributed provider action.

## Rollback and reconstruction

Retain the prior notarized DMG and checksum. Rollback replaces only `/Applications/Vela Workbench.app`; it does not rewrite Git, Entire checkpoints, Vela objects, Decisions, Events, Standing, or public projections. The clearable preferences file contains repository and tool paths only. If it is lost, reinstall Workbench, choose the signed Vela binary, and reselect the source and authority repositories. Repository and public state reconstruct from Git, signed Vela replay, and problems.science—not from Workbench process memory.
