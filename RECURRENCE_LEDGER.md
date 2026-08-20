# External recurrence ledger

This plain Markdown ledger measures outcomes. It is not a schema, service,
authority record or substitute for signed Vela objects. Update it only from
exact source, Repository and publication evidence; do not record private
scientific contents, credentials, prompts or participant contact details.

No outreach or contact is authorized. Every person-dependent entry below is a
dormant future gate.

The counter epoch begins on 2026-08-20 after Math main
`0ee3e94d754e12a6ad5079f32448871fc2182780`. Current Math main is
`5de716c896065c03c0a470d015ba2a328a527f73`; its later integration of the
already-frozen candidate packets and dormant outreach drafts does not start a
Result loop. Repository history and Standing already present before the epoch
are baseline evidence, not new recurrence. A case becomes eligible only when a
producer explicitly sponsors a new bounded Result or revision during this
epoch and an in-scope Repository is selected.

## Denominators

| Outcome | Numerator | Denominator | Exclusions |
|---|---|---|---|
| Results submitted | Distinct bounded Results whose Submission was successfully retained by the explicitly selected Repository during this counter epoch | Cases with an exact source assertion, producer explicitly sponsoring a new Result or revision during this epoch, and selected in-scope Repository | Pre-epoch history/Standing, draft packets, failed previews, imports and source commits without a Submission |
| Independent Checks completed | Submitted Results with a completed scoped Check whose checker, relationship, method and shared dependencies are disclosed and support the stated independence claim | Submitted Results requiring a Check | Producer self-checks, actor difference alone, CI, model judgements and Checks with unresolved independence conflicts |
| Attributed Decisions | Protocol-ready checked submissions receiving an accept or reject Decision by the named Repository authority | Protocol-ready checked submissions presented to that authority | Git merges, reviews, signatures, imported status and authority previews |
| Corrections | Accepted predecessor Results changed when the authority accepts a new bounded correction/supersession Submission carrying the exact `corrects` or `supersedes` relation and consequence-complete replay retires the predecessor | Accepted Results for which an exact correction need was established | Rejected correction proposals, edited prose, source-status changes or retractions not admitted by the Repository |
| Repeat contributors | Distinct participants completing the same declared role on at least two different Result loops | Participants completing at least one declared producer, checker, authority or reader role | Product-team test identities, disposable mechanism actors and repeated actions on one Result |
| Discovery-to-readback time | Elapsed time from first opening the frozen Problem/source occurrence to verified publication of the exact Decision/Standing readback | Complete loops with both timestamps and exact publication evidence | Incomplete loops; report their count and terminal stage separately rather than dropping them silently |

An `independent Check` is counted only from affirmative disclosed evidence. It
is never inferred from different actor strings. An `outside participant` is
reported separately from independence and means a consenting person outside the
Vela product team; it does not imply institutional independence.

## Case ledger

| Case | Producer gate | Submission | Independent Check | Authority / Decision | Correction | Repeat role | Discovery → readback | Current terminal stage |
|---|---|---|---|---|---|---|---|---|
| 1 Erdős 321 | dormant: source scope unresolved | 0 | 0 | Math in-scope only after resolution / 0 | 0 | 0 | — | source clarification |
| 2 Erdős 750 | dormant: sponsorship unresolved | 0 | 0 | new scoped authority unassigned / 0 | 0 | 0 | — | dependency clarification |
| 3 Erdős 56 | dormant: trust/licence unresolved | 0 | 0 | new scoped authority unassigned / 0 | 0 | 0 | — | rights and trust clarification |
| 4 Erdős 461 | producer exists; quality-control only | 0 | 0 | unassigned / 0 | 0 | 0 | — | CLEAN source review |
| 5 Erdős 545 | dormant: source scope unresolved | 0 | 0 | correction authority unassigned / 0 | 0 | 0 | — | NEEDS REVISION |
| 6 Erdős 550 | producer exists; quality-control only | 0 | 0 | unassigned / 0 | 0 | 0 | — | CLEAN source review |
| 7 Hadamard 668 | dormant: draft readiness | 0 | 0 | unassigned / 0 | 0 | 0 | — | source PR draft |
| 8 Erdős 94 bounded identity | bounded RF-01 source Result frozen; product-team disposable lifecycle excluded from this ledger | 0 external | 0 external | disposable reject excluded; existing Math Standing unchanged / 0 external | 0 | 0 | — | internal lifecycle complete; external Check/authority unassigned |
| 9 FAR Ramsey candidate | accountable producer unassigned | 0 | 0 | unassigned / 0 | 0 | 0 | — | discovery candidate only |
| 10 Directed 3-torus | participation not established | 0 | 0 | unassigned / 0 | 0 | 0 | — | external formal-check evidence only |

## Current program totals

- eligible cases: **0/10**;
- Results submitted: **0/0 eligible**;
- independent Checks completed: **0/0 submitted**;
- attributed Decisions: **0/0 protocol-ready**;
- accepted corrections: **0** with **0 established correction opportunities
  admitted to an authority**;
- repeat contributors: **0/0 participants completing a role**; and
- complete discovery-to-readback observations: **0**, with all ten cases
  currently incomplete at the terminal stages above.

## Operating evidence excluded from recurrence

| Evidence | Exact binding | Observed outcome | Recurrence boundary |
|---|---|---|---|
| FC Erdős 430 initial dogfood, run `32336112232` | candidate `84804da2e04a307be223f7dc067704619ca759c1`; input root `sha256:686eb629233dd0a25090c875c00f003971cf6e865e93f23b104051fadbba7f62` | `ERROR-INCONCLUSIVE`: deterministic Lean lane passed, but the AI lane exceeded its provider budget before emitting a receipt | Workflow-control evidence only; not a Result, Check, candidate failure or Decision |
| FC Erdős 430 control preflight, run `32341123076` | workflow `4e73166ec90aade25a7e3774642aacb1bb08e7b3`; same candidate | `ERROR` before model execution because dispatch supplied the wrong config path; zero artifacts and no rerun in that authorization | Dispatch-path evidence only; not a scientific or recurrence outcome |
| FC Erdős 430 controlled dogfood, run `32341367291` | workflow `4e73166ec90aade25a7e3774642aacb1bb08e7b3`; candidate `84804da2e04a307be223f7dc067704619ca759c1`; exact config SHA-256 `b83ac690f9c8e7d162d2be2c532903844c805dfe924c1757abbefd322443beef`; measurement record SHA-256 `7e49bd06af557c61215665bc4a31924fc95022e3dee258e35db2064f4a684ed0` | Internal advisory `SUCCESS/no_findings`; source-fidelity review found no disagreement; deterministic 8,054-job Lean lane passed; controls reported $1.16794915/$2.50, 30/40 turns and 620,438/900,000 ms | Successful internal advisory workflow evidence only; no publication, human disposition, independent Check, Submission, Decision, Event or Standing change |
| Result Factory RF-01, Erdős 94 bounded identity | lean-proofs source `852ffa6b50f3501a66d7ffbc116d8ae9b749c60c`; case `6de3d9b1398f1b4276c0011057784f45dc8e98cb`; Result SHA-256 `e45cd83f128f6f749574e470b75451e526207a489153074e9c7cc81a22a5d2db`; audit SHA-256 `b3dd1114c970e3494fca1d083e4d14d6a67ecb2e2de4b48b029986dbf6b211a0` | Bounded source Result proved; producer checks and a 39.88s same-provider cold check passed. Disposable Repository retained Submission `vsb_5e298f3ec5593dea` and complementary Verification `vvr_6b2936496d650c4a`, then attributed a reject Decision because independence was unsatisfied; final root `sha256:72c75daa5c8f108afae3969548b2859139a8a23848efbbe95b7ccd1589134f81`, strict replay pass | Internal mechanism qualification only; scientific state unchanged, zero accepted claims, no independent Check, real-authority action, Standing change, external recurrence or public readback |
| Result Factory RF-02, Erdős 730 full-density theorem | source `852ffa6b50f3501a66d7ffbc116d8ae9b749c60c`; case `22256c7cbca917ce05c11c880f995a3551f474d7`; Result SHA-256 `0665318674db6c549f1b720963d884770bd409047b6407aeffc64a921964699e` | Exact target build, arithmetic checks and direct axiom audit passed; cached cold check reproduced the whitelist. Pinned dependency contains two admitted declarations package-wide, but neither appears in the target's transitive axiom output | Bounded build-and-axiom evidence with an explicit environment trust caveat; cold check is same-provider; no Submission, independent Check, authority action, recurrence or readback |
| Result Factory RF-03, Erdős 23 q2+q2 branch | source `2d98b64d6a9c26593609260868f6731831bbc850`; case `86030e9113dd9b9f64dce8f9d0ed01ac4bf9f33d`; Result SHA-256 `c3c3b1b3f20b172a170614c657d6ecaf0d90e4afd0071d1f0b6658c1d89502dd` | Exact scoped theorem built with whitelist axioms; cached cold replay passed with qualifications; the tempting pure-overlap declaration is absent and its patch object is corrupt | Local first-party evidence only because source headers say Apache-2.0 while repository metadata says MIT; no redistribution, independent Check, Vela copy, authority action, recurrence or readback |
| Result Factory RF-04, Erdős 686 jet-inheritance boundary | source `800451d2c2dd6624508c0b8525b33a843e30445c`; reviewed case `e10061badd04c16cac1fe8f9eecebbdbd381adb0`; Result SHA-256 `8ad78f7f42923db84dbb9cc31eeb9367f155691298eb122f6282304a2f186189` | `infrastructure-inconclusive`: 0/2 target builds, 0/1 cache setup, zero target modules reached, fresh axiom sets or witnesses; cold same-provider review confirmed exact scope and stable bytes without rerunning exhausted work; two stale source-report module hashes are explicitly excluded from current evidence | Source-attributed rejection and narrower replacement remain separate observations, not a Vela correction or fresh formal Check; no Submission, Decision, recurrence or readback |
| Result Factory RF-05, Erdős 699 normalized-kernel route | source `f2e3db95b3ff0d36441646bb14606132e504f0c3`; case `5fd652960e3491088ab8c45634d55ec2bb726c08`; Result SHA-256 `0018919925cdb23c0d6eea7f00f0ed5d54b20878a66797337ccb69c6b757387c`; audit SHA-256 `b527aa9663caa272d5016dd71f2d814c6876dd30fe0b555a04f70c9d1a723ae1` | Kernel-checked witness rejects global emptiness of one normalized route and proves its row-digit/common-prime boundaries; producer and fresh same-provider internal checks passed with whitelist axioms; full-stream pairs matched | Negative/corrective source evidence only; explicitly not a counterexample to or solution of Erdős 699; no independent Check, Submission, Decision, authority receipt, recurrence or readback |
| Result Factory five-case denominator | preregistration SHA-256 `e63bd3beab5c7d0e1300909eb06a6110b9edda0982b8257aa8cdde2c2ab781cf`; reviewed report commit `b6c3a97dc2ef1a3acb10178f448aa74d5d40c4ef`; report SHA-256 `4c64ebc00c881d3fcc23199a223fcc6f38358ab2adf4928ba2efeaf8ca9221f1`; review wrapper `8f93b8e07d6176cbc7c71ff37aa6ac93d0f054e1`, review SHA-256 `2425fb401a8437458967be9113962d00dd3ed12c95cb1300a563b6e9ed1f7738` | Three bounded positives, one exact route-negative and one infrastructure-inconclusive; 4/5 cases reached fresh target axioms; producer target-build invocations 6/8 pass; cache/setup 1/5 pass; no theorem edits. Exact review corrected RF-02 failure denominators and stale RF-04 source locators, then passed | Internal source-throughput evidence only; same-provider cold checks and disposable actors are excluded from recurrence and independence denominators; all branches remain local and unmerged |
| 50-case Result Campaign Stage 0 | terminal report SHA-256 `572cc8e32eef1c20ed839fab931f00ab614bc227ee6f85f756af290a93325a1a`; payload A SHA-256 `e0e67710af9764b2dbf5ef31d31f2eb2858b8a2d4739835eb34508f59eb9c3d4`; internal manifest M SHA-256 `d2e00e091675b2c6c3f0937aec9d26e79064c059dda0170bb06dc097ce807da1`; terminal preflight receipt SHA-256 `c8e01d13f67dd30f1f4d171e916a96cac91427d151ce131fbeab860f75012248` | `NO-GO` before scientific execution: 50 cases frozen, but 0 producer attempts, 0 blind reviews, 0 inference sessions, $0 spend and 0 Vela lifecycle actions. The stable payload and BusyBox/environment/cgroup preflights passed; a missing `orbctl run -u` flag stopped the ancillary actor-environment probe before extraction/genesis, qualification receipt or seal | Campaign harness-invocation evidence only; not archive/environment/Vela/candidate evidence and excluded from every Result, Check, Decision, correction, recurrence and readback numerator or denominator |
| Phase 0 inheritance benchmark | preregistration SHA-256 `025fb027f948323afda33a846eb9f07dcf78a56f980b6e54fcfbc6728180bbc8`; sealed archive SHA-256 `5a3404ac0e4108787938a5cb3cab81dc44b9f6c18ad43a85dff717d85a065e45` | Independently reviewed as ready for internal sharing | Evaluation evidence only; not a Result loop or external validation |
| Phase 1 inheritance benchmark prelaunch | successor preregistration SHA-256 `ba5f8d22c87cd3c730ba3e7d3697e9d1c3533ad591ca690e8df7958cb10bda12`; terminal report SHA-256 `f0c2bb4500a5b5add0202a6025722dd0075a7244d36ce6a9fc05901e544064f1` | `NO-GO`: deterministic runtime controls passed, but the successor independent reviewer never invoked its mandatory ordinary-tool probe; zero candidate sessions, launches, retries, checkpoints or scores | Runner-control evidence only; not a candidate outcome, Result loop, product finding or external validation |
| Phase 1 deterministic host-gate successor | host receipt SHA-256 `1f80478566c2df186d986270acc4241ffd1086c8c9b4164227340ffbb77235e5`; anomaly receipt SHA-256 `2242c30f26886e894e87232b5a955446f85ba904119b1d76c02ebe54dd890aa8`; machine report SHA-256 `536fcd37f6ef946b00a98c28cbae8043463b9a33910d07db048b9ed433658be1`; human report SHA-256 `c238591322e7c98435d68449599228322e1212fdc97857ae768bfe79762f3c48` | One post-restart host qualification passed 160/160 profiles and 3,362 probes, but sealing stopped after the bound archive streamed zero bytes once; zero candidate/reviewer calls, candidate launches, candidate retries, authorizations, checkpoints or scores | Host-filesystem/provider-loss readability evidence only; not benchmark, Vela, Git, Result-loop or scientific evidence |

Every row above is excluded from every numerator and denominator in this
ledger. A model advisory, deterministic build, budget receipt or successful
workflow never supplies producer sponsorship, checker independence, Repository
authority or public readback.
