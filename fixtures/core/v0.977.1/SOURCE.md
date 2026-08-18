# Frozen signed Core v0.977.1 fixtures

Immutable release source:

- Repository: `vela-science/vela`
- Tag: `v0.977.1`
- Annotated tag object: `bc832ec79d6301b422c87c43bb614d242c8e4d08`
- Release commit: `0e057c0debcff775a3deb56150ceaccfd4707b41`
- Release tree: `55768612b82b93a4a01bb5aeddeb937dff678e4a`
- Protocol 1 manifest root: `sha256:448e7e80ac1ead40045d87df51f4352e80091c09ceae6e5acea250795b5ff9ed`
- Protocol manifest file SHA-256: `588e2fe1e4a9346938b0889cf3af5ae6d813bd9136c92d507638813e38c275d4`

Signed distribution evidence:

- macOS arm64 archive SHA-256: `c4e591c8683754ac0e310912b5227a697213bd4812e836ac88bef40430d9e7a6`
- macOS arm64 binary SHA-256: `a4f5594b2777b265f6d58296cc8e9efd85d0a72c82b49c0fce4805438ed46948`
- macOS release manifest SHA-256: `6942a9f215909a55e849c7722a49ea961ef6259294c9f8ca36f944d6fd88d884`
- macOS manifest signature SHA-256: `75bba62a571b0d343503945c959c1e0f555e7b01c9ac0d1a6abfec2718b8ef86`
- Linux x86_64 archive SHA-256: `a8a6c74c7694ea64b69b70d412c90506bc681de137b15c72416b3e7b2f7abf56`
- Linux x86_64 binary SHA-256: `3c25344f2a636a803d82fd7cf663e5638778d1121198301f478ff3dcc18f0270`
- Linux release manifest SHA-256: `6cd034646b57b1c5e8c3d85a95a882a3221f7277c543f6bdde69fc9757b423e4`
- Linux manifest signature SHA-256: `425bc7edacc006ba3919c951ef3e24b4da82502905b93867bb455c0d85bd50e3`

Both detached manifest signatures verified under namespace `vela-release` as `release@vela.space`, Ed25519 fingerprint `SHA256:MX3Eo1o9S5pLnx2kiNyAy2aME7PAWDtvqtUBljJst1M`. This is the distribution identity, not Repository authority.

The four schema files, `decision-inbox-v3.json`, signed `submission-bundle/submission.json`, and its content-addressed artifact are copied byte-for-byte from the release Git object. The signed envelope remains only a frozen parser/CLI-interface fixture; its signature does not grant Repository authority. The two frozen release bundle manifests are copied byte-for-byte from the immutable GitHub release. The Math and lean-proofs command fixtures were generated directly with the verified macOS arm64 v0.977.1 binary:

- Math checkout: `/Users/williamblair/personal/math`, commit `508b39adac51e6823ea0d666e789a1e016b20227`
- lean-proofs checkout: `/Users/williamblair/personal/lean-proofs`, commit `06d1322e62aa28b860da1ec66465d913c1902c78`; its integration manifest declares revision `a8c2872a27cf8d11cf6744ca4a2c5b49ace5fea0`

The v0.977.1 command bytes are identical to the prior reviewed interface fixtures. These files freeze release and interface behavior for tests. They are not runtime executables, Vela Protocol objects, scientific authority, or permission for any Tranche 2 action.
