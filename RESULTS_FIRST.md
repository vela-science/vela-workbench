# Results-first operating roster

This is the bounded operating roster for the 60–90 day Core and Workbench
feature freeze. It is not a Protocol object, ranking feed, scientific registry,
or claim that any candidate is correct. Source repositories own the work;
checks remain scoped; only an attributed Decision by an explicitly selected
Repository authority can change that Repository's Standing.

Frozen source observations, 2026-08-20:

- Vela Core `bf7a0911ef041904fcce89f77c9c569a32ad2269`, signed reader
  `0.977.3` with macOS SHA-256 `3a1173918bdcb887155bab681411bf5e9ff64d925fe1b50369ac37ab020b94ad`;
- Math `0ee3e94d754e12a6ad5079f32448871fc2182780` with the reviewed
  candidate packet commit `4624ea801c43b773b5d4a8b795c91e1882d8c02b`;
- Formal Conjectures `9f5ee773841921f460b4a26a3552f5eca4accaa0`;
- lean-proofs `852ffa6b50f3501a66d7ffbc116d8ae9b749c60c`;
- FAR `0f498a7e9252affd478cbfe324f51ea6d0119331`; and
- Torus source `753cbe37dc6428b15f5109b801301115ec61eb5d`.

`Independent checker` below names the required role. `Unassigned` means no one
has agreed to it; actor difference, a green build, a model judgement, or an FC
review does not fill that role. `Intended authority` is likewise an operational
destination, not authority already granted to a person or repository.

## Ten candidates

| # | Candidate and exact source | Source owner / producer | Independent checker | Intended Repository authority | Current status | Next human action | Present failure reason |
|---|---|---|---|---|---|---|---|
| 1 | Erdős 321 status conflict; FC `9f5ee773841921f460b4a26a3552f5eca4accaa0`, teorth/erdosproblems `931e7db4ee3c97705598f802e8358a201b9e422c`; Math packet `erdos-321-status-conflict.md` at `4624ea801c43b773b5d4a8b795c91e1882d8c02b` | teorth/erdosproblems, curated by Thomas Bloom / producer unassigned until scope is resolved | Unassigned; must compare each occurrence and rerun the official mismatch and known-result checks | Existing Math authority is in scope only after the source contradiction is resolved | Reviewed packet merged; FC and Math say open while source YAML says solved; FC #4444 remains open | Thomas resolves #4444 with an exact source/literature scope decision | The candidate assertion is not stable enough to submit |
| 2 | Erdős 750 conditional Stiebitz result; FC `9f5ee773841921f460b4a26a3552f5eca4accaa0`, Shashi456/erdos-formalizations `286f856aa3fc08957b80950fd18a45aab8d045ea`; Math packet at `4624ea801c43b773b5d4a8b795c91e1882d8c02b` | Shashi456 / Shashi456 if they elect to sponsor the exact result | Unassigned; exact build, `#print axioms`, full `Type` versus `Type*` fidelity, and Stiebitz-spec review | A new, separately scoped Repository with an explicitly named authority; current Math excludes 750 | Reviewed packet merged; proof closure adds `Erdos750.stiebitz_lower_bound` | Shashi456 states whether to sponsor the result and accept or discharge the Stiebitz dependency | Conditional dependency is unresolved and current Math is out of scope |
| 3 | Erdős 56 transitive native-decision result; FC `9f5ee773841921f460b4a26a3552f5eca4accaa0`, plby/lean-proofs `bebe632f2f6227a40e00b145bfbf7b3e1d68f8c2`; Math packet at `4624ea801c43b773b5d4a8b795c91e1882d8c02b` | plby / plby if they elect to sponsor the exact result | Unassigned; exact Lean 4.24 build, full type fidelity, transitive axiom closure, and trust/licence review | A new, separately scoped Repository with an explicitly named authority; current Math excludes 56 | Reviewed packet merged; closure adds `Lean.ofReduceBool` and `Lean.trustCompiler` | plby states intended compiled-reduction trust, whether a replacement is welcome, and applicable contribution terms | Trust policy and source licence are unresolved; current Math is out of scope |
| 4 | Hadamard 668; FC PR #4900 head `1721605c41f1bad11f592c2606618c539c43dc1f`, Paul-Lez wrapper `da94bc80401b6ece36d8dd2f5c316755fd97dd65` | Paul Lezeau / Paul Lezeau | Unassigned; exact source-fidelity and Comparator replay by a non-producer | Vela Repository authority unassigned; FC maintainers separately own source admission | Draft/review-required; FC build and Comparator run 32030830822 green; no human peer/source review claimed | Paul marks the PR ready and identifies a non-producer reviewer | Draft state and missing independent review |
| 5 | OEIS A129365, four exact results; FC PR #5016 head `acbeffc22ce2609d25eef324a4755878f3d6c84a`, KitaKen1 proof `9c0201540c337733d6b8afb2aff209f5489c122a` | Kenta Kitamura / Kenta Kitamura; codrut3 owns the umbrella integration | Unassigned; cold exact build and four-target statement-fidelity review | Vela Repository authority unassigned; FC maintainers separately own source admission | Proof signatures match; umbrella CI fails import-linter targets rather than proof checking | Kenta confirms producer scope; codrut3 fixes imports and pins the immutable `formal_proof`; maintainer reviews | Integration build is red and no independent rerun exists |
| 6 | OEIS A100434 correction plus three proofs; FC PR #5029 head `d5ed44b10fe187542e24dbf68312cc5b8b89afc9`, proof `32f88077a444b83741f1db6734390eebd3678ecf` | chy4pro / chy4pro | Unassigned; exact definition, corrected even-branch sign, and proof replay | Vela correction authority unassigned; FC maintainers separately own source admission | Proposed correction; old statement is refutable at `n = 0`; no full current-head build | chy4pro and a non-producer reviewer freeze correction scope, then an FC maintainer decides | Correction has not received independent fidelity review or maintainer admission |
| 7 | OEIS A374265 unboundedness correction; FC PR #4946 head `00f8b1898751bb51710f8c46534a8dc11981c6b0`, proof ancestor `ce05cfb9dbe2e5aac6634402ffcc8a38ce368ef7` | Moritz / Moritz | Unassigned; exact process-definition and correction-fidelity review | Vela correction authority unassigned; FC maintainers separately own source admission | Batch CI green; prior bounded direction was false; no approval | Moritz freezes the corrected assertion and requests a non-producer fidelity review | No independent review or maintainer admission |
| 8 | Erdős 94 `Erdos94.variants.sum_multiplicity`; lean-proofs proof commit `423344341fbfdf4f8f684a302c5d05379125e7dc`, file SHA-256 `412975add8b6963bb44378f5d8ef41fd1f860b9ec06495432ab97e8ca60ffbe0` | William Blair / existing source producer | Unassigned; genuine mathematical and source-fidelity reviewer | Existing Math authority only for an exact justified revision; its current Standing must not be reinterpreted | Source custody and hosted/cold checks pass; only the bounded identity is proved | Recruit a non-producer reviewer or obtain explicit authorization for a later external formal-verification handoff | No genuine independent scientific review; no broader Erdős 94 claim is allowed |
| 9 | FAR/ProbXiv Ramsey counterexample candidate; ProbXiv locator `on-multicolor-ramsey-number-of-paths-versus-cycles-2`, observed FAR `0f498a7e9252affd478cbfe324f51ea6d0119331` | Source collection/FAR authors; attempt credits a tool, not an accountable producer | Unassigned; reconstruct the counterexample and compare the exact source statement | None selected | ProbXiv remains open; one model attempt and machine judge report a counterexample; no person examined it | A named mathematician adopts the exact candidate and reproduces it from source-safe locators | No durable FAR pilot snapshot, accountable producer, independent check, or authority |
| 10 | Directed 3-torus Hamilton decomposition; arXiv `2603.24708v1`, Torus source `753cbe37dc6428b15f5109b801301115ec61eb5d`, ProbXiv locator `directed-3-torus-hamilton` | SangHyun Park / producer participation not established | Unassigned; cold replay of the exact d=3 files plus independent statement fidelity | None selected | ProbXiv says solved and Lean-checked but explicitly says no person examined it and compilation does not prove fidelity | Ask Park whether the exact d=3 result may enter a source-owned review, then recruit a non-producer checker | No evidenced willingness, independent fidelity check, or Repository authority |

ProbXiv exposes no observed content licence, public API, export, or
provider-independent snapshot. For candidates 9 and 10 retain only public
locators, short factual observations, dates, and digests; do not copy page
bodies.

## First five asks — drafts only

These messages are ready for user approval. They have not been sent. They ask
for source decisions or participation, not endorsement of Vela. Full usability
pilot invitations remain blocked by the gates in [PILOT.md](PILOT.md).

### 1. Thomas Bloom — Erdős 321 source scope

> We found one exact status conflict for Erdős 321: the current
> erdosproblems YAML at `931e7db4ee3c97705598f802e8358a201b9e422c`
> says solved, while Formal Conjectures at
> `9f5ee773841921f460b4a26a3552f5eca4accaa0` and the existing Math record say
> open. FC issue #4444 already
> tracks it. Would you state whether “solved” applies to the exact FC
> occurrence, only an asymptotic result, or another formulation, and point to
> the controlling citation? We will not submit or revise a Vela Result until
> that source decision is explicit. This is a scope check, not a request to
> endorse Vela.

### 2. Shashi456 — Erdős 750 conditional result

> We reproduced that the exact Erdős 750 proof closure at
> `286f856aa3fc08957b80950fd18a45aab8d045ea` depends
> on `Erdos750.stiebitz_lower_bound`, matching FC's conditional annotation.
> Would you like to act as producer for this bounded conditional Result, and is
> that Stiebitz statement intended as an accepted assumption, a dependency to
> discharge, or a specification still needing review? If yes, we will arrange
> a separate exact-build/type-fidelity Check. The current Math Repository is
> out of scope and will not admit it.

### 3. plby — Erdős 56 trust and licence

> The exact Erdős 56 theorem at
> `bebe632f2f6227a40e00b145bfbf7b3e1d68f8c2` has a transitive compiled axiom
> closure containing `Lean.ofReduceBool` and `Lean.trustCompiler`, although the
> target declaration does not show that dependency directly. Would you confirm
> whether that trust path is intended, whether a proof avoiding it would be
> welcome, and what contribution/licence terms apply to this source? We will
> not redistribute or submit the Result before those points are explicit.

### 4. Paul Lezeau — Hadamard 668 producer handoff

> Your Hadamard 668 Comparator wrapper at
> `da94bc80401b6ece36d8dd2f5c316755fd97dd65` pins FC PR #4900 head
> `1721605c41f1bad11f592c2606618c539c43dc1f`; the recorded FC and Comparator
> builds are green. If the PR is
> ready, would you mark it ready for review and confirm the exact two-result
> scope (`Hadamard.H_isHadamard` and `exists_hadamard_668`)? We would then seek
> a non-producer source-fidelity/replay checker. A green build will not be
> represented as independent review or a Repository Decision.

### 5. Kenta Kitamura — OEIS A129365 producer handoff

> We matched all four A129365 theorem signatures in FC PR #5016 head
> `acbeffc22ce2609d25eef324a4755878f3d6c84a` to your Apache-2.0 proof commit
> `9c0201540c337733d6b8afb2aff209f5489c122a`. Would you confirm
> those four exact results as the producer scope and permit that immutable
> commit to be used as the `formal_proof` locator? The umbrella PR currently
> fails import-linter checks, not proof checking; after the integration fix we
> would ask a separate person for a cold build and statement-fidelity Check.

## Outcome counters

Program counters start from this roster; prior repository history is not counted
as new pilot throughput.

- submitted Results: **0**;
- independently performed Checks: **0**;
- attributed Decisions: **0**;
- accepted corrections: **0**;
- repeat contributors: **0**; and
- discovery-to-public-readback observations: **0**.

The current FC review-loop measurement is operational evidence, not a Vela
counter: against review-skill head
`75b35e7c5f86e895debdf717a5beda18bc042f5e`, Paul cases Erdős 461 and 550
were CLEAN in 87s and 68s, while Erdős 545 NEEDS REVISION in 89s because the
formal declaration closes the unrestricted small-`m` statement but the source
still marks the intended large-`m` problem open. No skill change was justified:
two CLEAN cases and one already-documented material case did not establish the
same new failure twice.

The same-information inheritance benchmark is owned by Codex task
`01a01da2-e354-7453-a60b-aa044d111971`. Consume its frozen result in the pilot
go/no-go; do not add a second runner or benchmark implementation here.
