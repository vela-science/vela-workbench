# Frozen Core interface fixtures

Interface source:

- Repository: `vela-science/vela`
- Commit: `3bfcf23f12fb6a38a924a257ba25ad3d8594dc78`
- Tree: `ab85ef6ec7f6cd7c49fc4664bbbbd4f597e71816`
- Review state: merged current interface, exact re-review PASS
- Release state: not released

The three schema files and `decision-inbox-v3.json` are copied byte-for-byte from that Git object. The Math and lean-proofs command fixtures were captured on 2026-08-17 with the installed signed runtime:

- Version: `vela 0.977.0`
- Executable: `/Users/williamblair/.local/bin/vela`
- Executable SHA-256: `4332427789bf3dac83ebad9843670047b448f6ba370661f48a0100cbb61bc00c`
- Math checkout: `/Users/williamblair/personal/math`, commit `508b39adac51e6823ea0d666e789a1e016b20227`
- lean-proofs checkout: `/Users/williamblair/personal/lean-proofs`; its integration manifest declares revision `a8c2872a27cf8d11cf6744ca4a2c5b49ace5fea0`

These files freeze interface behavior for tests. They are not runtime executables, Vela Protocol objects, scientific authority, or a reason to execute unreleased Core `main` from the Workbench.
