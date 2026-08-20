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
| 8 Erdős 94 bounded identity | no new revision sponsored; existing source producer and Standing are pre-epoch baseline | 0 new | 0 new | existing Math Standing unchanged / 0 new | 0 new | 0 | — | independent review unassigned |
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
| Phase 0 inheritance benchmark | preregistration SHA-256 `025fb027f948323afda33a846eb9f07dcf78a56f980b6e54fcfbc6728180bbc8`; sealed archive SHA-256 `5a3404ac0e4108787938a5cb3cab81dc44b9f6c18ad43a85dff717d85a065e45` | Independently reviewed as ready for internal sharing | Evaluation evidence only; not a Result loop or external validation |

Every row above is excluded from every numerator and denominator in this
ledger. A model advisory, deterministic build, budget receipt or successful
workflow never supplies producer sponsorship, checker independence, Repository
authority or public readback.
