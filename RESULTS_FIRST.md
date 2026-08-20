# Results-first operating roster

This is the bounded operating roster for the 60–90 day Core and Workbench
feature freeze. It is not a Protocol object, ranking feed, scientific registry,
or claim that any candidate is correct. Source repositories own the work;
checks remain scoped; only an attributed Decision by an explicitly selected
Repository authority can change that Repository's Standing.

Frozen source observations, 2026-08-20:

- Vela Core `bf7a0911ef041904fcce89f77c9c569a32ad2269`, signed reader
  `0.977.3` with macOS SHA-256 `3a1173918bdcb887155bab681411bf5e9ff64d925fe1b50369ac37ab020b94ad`;
- Math main `5de716c896065c03c0a470d015ba2a328a527f73`, with immutable
  benchmark packet commit `4624ea801c43b773b5d4a8b795c91e1882d8c02b` and dormant,
  unsent outreach commit `7d35a741f701eab4dde8160e9c30434ee3cbd551`;
- Formal Conjectures `9f5ee773841921f460b4a26a3552f5eca4accaa0`;
- lean-proofs `852ffa6b50f3501a66d7ffbc116d8ae9b749c60c`;
- FAR `0f498a7e9252affd478cbfe324f51ea6d0119331`; and
- Torus source `753cbe37dc6428b15f5109b801301115ec61eb5d`.

`Independent checker` below names the required role. `Unassigned` means no one
has agreed to it; actor difference, a green build, a model judgement, or an FC
review does not fill that role. `Intended authority` is likewise an operational
destination, not authority already granted to a person or repository.

## Ten cases

`Axis` states why a case is retained. CLEAN quality-control cases remain useful
for cold-successor measurement but are not promoted into substantive Results.

| # | Candidate and exact source | Source owner / producer | Required independent Check | Intended Repository authority | Current evidence | Blocker and dormant future gate | Axis |
|---|---|---|---|---|---|---|---|
| 1 | Erdős 321 status conflict; FC `9f5ee773841921f460b4a26a3552f5eca4accaa0`, teorth/erdosproblems `931e7db4ee3c97705598f802e8358a201b9e422c`; Math packet `4624ea801c43b773b5d4a8b795c91e1882d8c02b` | teorth/erdosproblems, curated by Thomas Bloom / producer unassigned until scope is resolved | Source owner identifies the exact result and an unconflicted checker compares every occurrence, then reruns the official mismatch and known-result checks | Existing Math authority is in scope only after contradiction resolution | Reviewed packet merged; FC and Math say open while source YAML says solved; FC #4444 is open | Assertion is not stable enough to submit; Thomas supplies the controlling scope and citation in #4444 | discovery, scope, correction |
| 2 | Erdős 750 conditional Stiebitz result; FC `9f5ee773841921f460b4a26a3552f5eca4accaa0`, Shashi456/erdos-formalizations `286f856aa3fc08957b80950fd18a45aab8d045ea`; Math packet `4624ea801c43b773b5d4a8b795c91e1882d8c02b` | Shashi456 / Shashi456 only if they elect to sponsor the exact result | Non-producer exact build, `#print axioms`, full `Type` versus `Type*` fidelity and Stiebitz-spec/reference review | New separately scoped Repository and named authority; current Math excludes 750 | Reviewed packet merged; closure adds `Erdos750.stiebitz_lower_bound`; FC conditional metadata is correct | Dependency and sponsorship unresolved; Shashi456 decides whether to sponsor and accept or discharge it | scope, trust, cold-successor |
| 3 | Erdős 56 transitive native-decision result; FC `9f5ee773841921f460b4a26a3552f5eca4accaa0`, plby/lean-proofs `bebe632f2f6227a40e00b145bfbf7b3e1d68f8c2`; Math packet `4624ea801c43b773b5d4a8b795c91e1882d8c02b` | plby / plby only if they elect to sponsor the exact result | Non-producer Lean 4.24 build, full theorem-type fidelity, transitive axiom closure and owner trust/licence-policy review | New separately scoped Repository and named authority; current Math excludes 56 | Reviewed packet merged; closure adds `Lean.ofReduceBool` and `Lean.trustCompiler`; no licence file observed | Trust, licence and sponsorship unresolved; plby states intended trust path, replacement interest and contribution terms | trust/licence, cold-successor |
| 4 | Paul batch #8, `Erdos461.erdos_461`; source branch head `fc772f15aa14e01a081c4bdb6a7472f2ea95e65f`, exact base `398958d3964d738886bd24433918c365df4a2aab` | Paul Lezeau / Paul Lezeau; FC convention Apache-2.0, source-page text not relicensed | Unassigned non-producer source-fidelity review if the case is ever promoted beyond quality control | Vela authority unassigned; FC maintainers separately own source admission | Current-skill review CLEAN in 87s; focused local Lean and reviewer build passed | No substantive Result gap presently identified; Paul and FC maintainers handle existing nits and draft readiness | scope, cold-successor |
| 5 | Paul batch #9, `Erdos545.erdos_545`; source branch head `f11eee52f0189ec222247f855397006fb1cd8f4e`, exact base `398958d3964d738886bd24433918c365df4a2aab` | Paul Lezeau / Paul Lezeau; FC convention Apache-2.0, source-page text not relicensed | Source owner defines the intended large-`m` regime; a non-producer then compares the corrected main declaration and retained literal-negative variant to the exact source | Vela correction authority unassigned; FC maintainers separately own source admission | Current-skill review NEEDS REVISION in 89s; builds passed; declaration proves falsity of the unrestricted small-`m` statement while source remains OPEN | This is the only substantive FC throughput Result candidate; Paul/source owner clarifies scope, then maintainers review the correction | scope, correction, cold-successor |
| 6 | Paul batch #10, `Erdos550.erdos_550`; source branch head `735440a72e3b0f1dbb25b83682438cecc3b6476f`, exact base `398958d3964d738886bd24433918c365df4a2aab` | Paul Lezeau / Paul Lezeau; FC convention Apache-2.0, source-page text not relicensed | Unassigned non-producer source-fidelity review if the case is ever promoted beyond quality control | Vela authority unassigned; FC maintainers separately own source admission | Current-skill review CLEAN in 68s; focused local Lean and reviewer build passed | No substantive Result gap presently identified; Paul and FC maintainers handle existing nits and draft readiness | scope, cold-successor |
| 7 | Hadamard 668; FC PR #4900 head `1721605c41f1bad11f592c2606618c539c43dc1f`, Apache-2.0 Paul-Lez wrapper `da94bc80401b6ece36d8dd2f5c316755fd97dd65` | Paul Lezeau / Paul Lezeau | Non-producer exact source-fidelity review and Comparator replay | Vela authority unassigned; FC maintainers separately own source admission | Draft/review-required; FC build and Comparator run 32030830822 green; no human peer/source review claimed | Paul marks the PR ready and identifies a non-producer reviewer | trust, cold-successor |
| 8 | Erdős 94 `Erdos94.variants.sum_multiplicity`; MIT lean-proofs commit `423344341fbfdf4f8f684a302c5d05379125e7dc`, file SHA-256 `412975add8b6963bb44378f5d8ef41fd1f860b9ec06495432ab97e8ca60ffbe0` | William Blair / existing source producer | Genuine non-producer mathematical and source-fidelity review | Existing Math authority only for an exactly justified revision; current Standing must not be reinterpreted | Source custody, hosted verify and cold build/axiom evidence pass; only `sum_multiplicity` is proved | No genuine independent scientific review; recruit one or obtain separately authorized external formal-verification handoff | trust, correction, cold-successor |
| 9 | FAR/ProbXiv Ramsey counterexample candidate; ProbXiv locator `on-multicolor-ramsey-number-of-paths-versus-cycles-2`, Apache-2.0 FAR pipeline `0f498a7e9252affd478cbfe324f51ea6d0119331` | Source collection/FAR authors; attempt credits a tool, not an accountable producer | Named mathematician reconstructs the counterexample and compares the exact source statement | None selected | ProbXiv remains open; one model attempt and machine judge report a counterexample; no person examined it | No durable pilot snapshot, accountable producer, checker or authority; a mathematician must first adopt the exact candidate | discovery, scope, cold-successor |
| 10 | Directed 3-torus Hamilton decomposition; arXiv `2603.24708v1`, MIT Torus source `753cbe37dc6428b15f5109b801301115ec61eb5d`, ProbXiv locator `directed-3-torus-hamilton` | SangHyun Park / producer participation not established | Cold replay of exact d=3 files plus independent source-statement fidelity | None selected | ProbXiv says solved and Lean-checked but explicitly says no person examined it and compilation does not prove fidelity | No evidenced willingness, checker or authority; ask Park whether the exact d=3 Result may enter source-owned review | discovery, trust, cold-successor |

ProbXiv exposes no observed content licence, public API, export, or
provider-independent snapshot. For cases 9 and 10 retain only public
locators, short factual observations, dates, and digests; do not copy page
bodies.

## First five participant packets — dormant drafts only

No outreach or contact is authorized. Do not send, post, DM, comment, open an
issue or PR, invite a participant, or otherwise act on these drafts. They are
retained only so a future user authorization can evaluate exact bounded asks;
even with later authorization, the full usability pilot remains blocked by the
gates in [PILOT.md](PILOT.md).

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

### 2. Paul Lezeau — Erdős 545 source scope

> The current review of your exact Erdős 545 head
> `f11eee52f0189ec222247f855397006fb1cd8f4e` found that lines 68–74 close the
> unrestricted statement by the known small-`m` counterexamples, while the
> source still marks the intended problem OPEN. Under a future authorization,
> would you state the intended large-`m` regime and whether the literal
> negative should remain only as a solved variant? We would then arrange a
> separate non-producer comparison before any Vela Submission or correction.

### 3. Shashi456 — Erdős 750 conditional result

> We reproduced that the exact Erdős 750 proof closure at
> `286f856aa3fc08957b80950fd18a45aab8d045ea` depends
> on `Erdos750.stiebitz_lower_bound`, matching FC's conditional annotation.
> Would you like to act as producer for this bounded conditional Result, and is
> that Stiebitz statement intended as an accepted assumption, a dependency to
> discharge, or a specification still needing review? If yes, we will arrange
> a separate exact-build/type-fidelity Check. The current Math Repository is
> out of scope and will not admit it.

### 4. plby — Erdős 56 trust and licence

> The exact Erdős 56 theorem at
> `bebe632f2f6227a40e00b145bfbf7b3e1d68f8c2` has a transitive compiled axiom
> closure containing `Lean.ofReduceBool` and `Lean.trustCompiler`, although the
> target declaration does not show that dependency directly. Would you confirm
> whether that trust path is intended, whether a proof avoiding it would be
> welcome, and what contribution/licence terms apply to this source? We will
> not redistribute or submit the Result before those points are explicit.

### 5. Paul Lezeau — Hadamard 668 producer handoff

> Your Hadamard 668 Comparator wrapper at
> `da94bc80401b6ece36d8dd2f5c316755fd97dd65` pins FC PR #4900 head
> `1721605c41f1bad11f592c2606618c539c43dc1f`; the recorded FC and Comparator
> builds are green. If the PR is
> ready, would you mark it ready for review and confirm the exact two-result
> scope (`Hadamard.H_isHadamard` and `exists_hadamard_668`)? We would then seek
> a non-producer source-fidelity/replay checker. A green build will not be
> represented as independent review or a Repository Decision.

The dormant outreach order is 1 → 2 → 3 → 4 → 5: resolve the two exact source
scope conflicts first, then the conditional dependency and trust/licence gates,
then ask whether the already-green Hadamard wrapper is ready for a separate
review. This ordering does not authorize any contact.

## Actionability

Immediately actionable inside the current authorization:

- preserve and independently review these operating records;
- consume the benchmark task's final Phase 0 review without reproducing its
  scoring; and
- keep exact source heads, rights and blocker classifications current through
  read-only checks.

Dormant future gates requiring new authorization or another owner:

- every participant packet and source-owner contact above;
- recruitment of any independent checker or Repository authority;
- the FC fork's ordinary review/merge decision for validator correction
  `06563ebad751b85f89cbd0ff94602e3145d32f15`;
- Apple credential handoff and signed/notarized Workbench distribution; and
- Web support, deployment and public-readback qualification.

Deliberately rejected now: sending outreach, rerunning Erdős 430, running Phase
1 before Phase 0 review, creating a profile migration for out-of-scope Math
cases, copying ProbXiv page bodies, or adding Core/Workbench/Web features,
schemas, services or provider configuration.

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

A later nonpublishing Erdős 430 dogfood run is a funnel-control measurement,
not a Result or Check. GitHub run `32336112232` was bound to candidate head
`84804da2e04a307be223f7dc067704619ca759c1` and input root
`sha256:686eb629233dd0a25090c875c00f003971cf6e865e93f23b104051fadbba7f62`.
The deterministic lane passed its 8,054-job Lean build, warnings/style,
diff-check and import-policy gates in 187.425s. The AI source-fidelity lane
terminated `error_max_budget_usd` after 447.543s and $1.0443573, exceeding its
$1 cap by $0.0443573 and reporting 26 turns despite `maxTurns: 20`. It emitted
no structured receipt, so semantic findings, false-positive count,
aggregation, escalation, human disposition and publication all remain unknown
or skipped. The terminal outcome is typed `ERROR-INCONCLUSIVE`, never candidate
pass/fail. Machine record SHA-256:
`6aae4c7c213d42407a58a606368c8aa1a1ec1e913193e8999712dc06b0353154`.
Do not pay for another run until maximum-turn and hard-budget enforcement are
observable and reliable. This one workflow failure does not meet the repeated
real-Result threshold for Core or Workbench machinery. The conservative
validator correction at `06563ebad751b85f89cbd0ff94602e3145d32f15` should
proceed only through the FC fork's ordinary review path; it neither repairs nor
reclassifies this run.

The same-information inheritance benchmark is owned by Codex task
`01a01da2-e354-7453-a60b-aa044d111971`. Consume its frozen result in the pilot
go/no-go; do not add a second runner or benchmark implementation here.
