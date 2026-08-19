# Frozen signed Core v0.977.3 fixtures

Immutable release source:

- Repository: `vela-science/vela`
- Tag: `v0.977.3`
- Annotated tag object: `73d35e820f69b288fc620b3be946db3ffa0b4e01`
- Release commit: `1c1abe8f365f16803fea889bf9280877992a6d02`
- Release tree: `66bb4cb5173ff50beeef45c03fa11060e1e9e377`
- Protocol 1 manifest root: `sha256:534120b2dfcfb8357d7f20be3e3a0e0dfbe21d0cd51082edb5dea686ca82ec85`
- Protocol manifest file SHA-256: `d13fe8b9dfd3273ed7eb847f4b52aec3777218cce26fc7d0db95fa5e31063fd6`

Signed distribution evidence:

- macOS arm64 archive SHA-256: `f1299f217985c1eecdfb20ef8750014bbf7b2fc6d3cc31ca33bdd433e2be8991`
- macOS arm64 binary SHA-256: `3a1173918bdcb887155bab681411bf5e9ff64d925fe1b50369ac37ab020b94ad`
- macOS release manifest SHA-256: `b0da2d3e9a5d896fdfad96fe096c9caee5c66e54ed713bd0a0c408e14a1b65a0`
- macOS manifest signature SHA-256: `828e5c392ac4fa93936e88d73aa0072189e57dd64f1930e29070fe4c995904b8`
- Linux x86_64 archive SHA-256: `072af0182152ac4b4a8f04cec7e37f1dc3b5b7a42f49ef4066cce559e26835b3`
- Linux x86_64 binary SHA-256: `89e5f366db5480a011c722bdc7d3c7f09e07fe78c0cd2855d2e53d3a419520a0`
- Linux release manifest SHA-256: `3308d8867575f3703070b93f9a664a2e35f98bc1aac5a060efa4d3a5d1ae9b72`
- Linux manifest signature SHA-256: `686ced3067be2ff5f092c4379f099db6e647d85567744e045c7e161e7a3a5dec`

Both detached manifest signatures verified under namespace `vela-release` as `release@vela.space`, Ed25519 fingerprint `SHA256:MX3Eo1o9S5pLnx2kiNyAy2aME7PAWDtvqtUBljJst1M`. This is the distribution identity, not Repository authority.

The four schema files, `decision-inbox-v3.json`, signed `submission-bundle/submission.json`, and its content-addressed artifact are copied byte-for-byte from the release Git object. The signed envelope remains only a frozen parser/CLI-interface fixture; its signature does not grant Repository authority. The two frozen release bundle manifests are copied byte-for-byte from the immutable GitHub release. The Math and lean-proofs command fixtures were generated directly with the verified macOS arm64 v0.977.3 binary:

- Math checkout: `/Users/williamblair/personal/math`, commit `84b118ed1622d34e5a1431821cf35dca91fb8720`
- lean-proofs checkout: clean temporary clone of main commit `852ffa6b50f3501a66d7ffbc116d8ae9b749c60c`; its native integration manifest intentionally declares exact bounded proof revision `423344341fbfdf4f8f684a302c5d05379125e7dc`

The recovery inspection fixture is the clean, read-only `vela.recovery-inspection.v1` result for Math. The other command fixtures freeze current read interfaces against the same signed runtime. These files are not runtime executables, Vela Protocol objects, scientific authority, or permission for any Workbench action.
