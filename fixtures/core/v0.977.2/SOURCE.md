# Frozen signed Core v0.977.2 fixtures

Immutable release source:

- Repository: `vela-science/vela`
- Tag: `v0.977.2`
- Annotated tag object: `2b0c90a2ec4350da38236ad3788278309bd07119`
- Release commit: `c1a34373c2cdd937ed34fd128174a66fa12be71a`
- Release tree: `b9188626039cfc1a4d7d4098d1b7fc6a4a92ad55`
- Protocol 1 manifest root: `sha256:50c8dc2d99a40b535f9b03e9759fb463a64d6e942feba4655c8f7192171cece6`
- Protocol manifest file SHA-256: `50c8dc2d99a40b535f9b03e9759fb463a64d6e942feba4655c8f7192171cece6`

Signed distribution evidence:

- macOS arm64 archive SHA-256: `dab66bd3bed975bcf253bf74947803cbfd966a373ebb291307b4b1836fc7124e`
- macOS arm64 binary SHA-256: `286ed839ea81b7ed283e04ea1823c1515ad242dcee02b424787b8daa667625e2`
- macOS release manifest SHA-256: `afaf60b88eb5eeafeefda9b1edd4080f137eb86ad64964f1d70cdf3762c3183a`
- macOS manifest signature SHA-256: `34d1cdffbf9d59a6643127a3d4c7f4cdcd81aaf5dbbc48707a0205f2d87cb874`
- Linux x86_64 archive SHA-256: `23f03735f97820cbf56e5f2cc0c9d56b5657d7113dcbd0b738aafb1e241498b3`
- Linux x86_64 binary SHA-256: `3e2e12ac3410aa4a62013d3d7e2ceb828504c7beaff09cf1d126bc2d7ba077cd`
- Linux release manifest SHA-256: `7d3af1bf4ae81bcbf8cee190e34d53c0fa0af2e6ce3c1d785a36fc10169cf603`
- Linux manifest signature SHA-256: `14f7ec810e9e5d9de8310fa04b140e1f55013b5d7b8675c688ff806062c2f05f`

Both detached manifest signatures verified under namespace `vela-release` as `release@vela.space`, Ed25519 fingerprint `SHA256:MX3Eo1o9S5pLnx2kiNyAy2aME7PAWDtvqtUBljJst1M`. This is the distribution identity, not Repository authority.

The four schema files, `decision-inbox-v3.json`, signed `submission-bundle/submission.json`, and its content-addressed artifact are copied byte-for-byte from the release Git object. The signed envelope remains only a frozen parser/CLI-interface fixture; its signature does not grant Repository authority. The two frozen release bundle manifests are copied byte-for-byte from the immutable GitHub release. The Math and lean-proofs command fixtures were generated directly with the verified macOS arm64 v0.977.2 binary:

- Math checkout: `/Users/williamblair/personal/math`, commit `84b118ed1622d34e5a1431821cf35dca91fb8720`
- lean-proofs checkout: `/Users/williamblair/personal/lean-proofs`, commit `06d1322e62aa28b860da1ec66465d913c1902c78`; its integration manifest declares revision `a8c2872a27cf8d11cf6744ca4a2c5b49ace5fea0`

The v0.977.2 integration command bytes and selected schemas are identical to the prior reviewed interface fixtures; the Math fixtures intentionally bind the current migrated Repository head. These files freeze release and interface behavior for tests. They are not runtime executables, Vela Protocol objects, scientific authority, or permission for any Workbench action.
