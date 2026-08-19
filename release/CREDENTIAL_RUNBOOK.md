# External-credential runbook: sign and notarize the macOS release

Everything internal to this repository is already done before this runbook
starts: `bun install --frozen-lockfile`, the full `bun run check` gate, and
the locked release build all pass on clean, remote-equal `main`. The only
missing inputs are Apple credentials. `scripts/release-macos.sh` is the sole
release entry point and it fails closed at each of these boundaries.

The script does **not** use a `notarytool store-credentials` keychain
profile. It requires the App Store Connect API key as three environment
variables (`APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`) which
it withholds from every build stage and hands only to Tauri's final
bundling/notarization step. Do not create a keychain profile for this path.

## One-time Apple account actions

1. **Apple Developer Program.** Enroll the releasing Apple ID (individual or
   organization) at developer.apple.com. Developer ID certificates can only
   be created by the **Account Holder**.

2. **Developer ID Application certificate.** As Account Holder, in Xcode:
   Settings → Accounts → select the team → Manage Certificates… → **+** →
   **Developer ID Application**. Xcode creates the certificate and installs
   it with its private key into the login keychain. (Equivalent: create a
   CSR in Keychain Access and use developer.apple.com → Certificates → + →
   Developer ID Application, then double-click the downloaded `.cer`.)
   Confirm it is visible:

   ```sh
   security find-identity -v -p codesigning
   ```

   The line must read `Developer ID Application: <Name> (<TEAMID>)`.

3. **App Store Connect API key** (for notarization). At
   appstoreconnect.apple.com → Users and Access → Integrations →
   App Store Connect API → **Team Keys** → Generate API Key.
   - Role: **Developer** (sufficient for notarization; Admin also works).
   - Record the **Issuer ID** (UUID shown at the top of the Team Keys page)
     and the key's **Key ID**.
   - Download `AuthKey_<KEYID>.p8` — Apple offers this download exactly
     once. Store it at a stable private path and restrict it:

   ```sh
   mkdir -p ~/.appstoreconnect/private_keys
   mv ~/Downloads/AuthKey_<KEYID>.p8 ~/.appstoreconnect/private_keys/
   chmod 600 ~/.appstoreconnect/private_keys/AuthKey_<KEYID>.p8
   ```

## The release run

Set the four variables the script requires (names exactly as
`scripts/release-macos.sh` reads them), then preflight:

```sh
export APPLE_SIGNING_IDENTITY='Developer ID Application: <Name> (<TEAMID>)'   # full displayed name from security find-identity
export APPLE_API_ISSUER='<ISSUER-UUID>'
export APPLE_API_KEY='<KEYID>'
export APPLE_API_KEY_PATH="$HOME/.appstoreconnect/private_keys/AuthKey_<KEYID>.p8"
bun run release:macos:check
```

`release:macos:check` validates the host, clean remote-equal `main`, the
signing identity in the keychain, and the complete API-key triple without
building. When it prints `macOS release preflight passed`, run the release:

```sh
bun run release:macos
```

That single command produces and verifies the signed, notarized, stapled
DMG at `src-tauri/target/release/bundle/dmg/Vela Workbench_0.1.0_aarch64.dmg`
with its sibling `.sha256`, refusing to finish unless codesign, Gatekeeper
(`spctl` on app and DMG), stapler validation on both, `hdiutil verify`,
signer identity and TeamIdentifier match, and the app mounted from the final
DMG all pass. After those checks it emits two SPDX SBOM sidecars next to the
DMG — `...dmg.spdx.json` (the signed DMG) and `...source.spdx.json` (the
source-locked manifests) — each with its own `.sha256`, using the pinned
syft 1.50.0 and refusing if syft is absent or a different version. It never
prints credential values and never publishes.

After it succeeds: run the clean-account first-run checks in INSTALL.md,
fill every `TODO-SIGNING` slot in `release/RELEASE_BODY_v0.1.0.md` from the
run's output, and publish those exact bytes as a separate attributed action.
