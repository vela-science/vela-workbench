# Vela Workbench Tranche 3 threat and capability contract

Status: frozen before privileged implementation  
Runtime: signed Vela `v0.977.1` only  
Base: `1d5e8e29b1429b273461dd56d9347dc630c1536c`

## Scope

Tranche 3 completes one local Repository loop through the existing Vela CLI:
scoped Verification author/import, consequence-only Decision Inbox reads, one
attributed accept or reject, immediate replay/readback, and explicit recovery of
one exact interrupted operation. All mutating packaged QA uses disposable
Repositories. Math, lean-proofs, Web, Core, providers, and production state are
read-only.

This contract adds no Protocol object, hosted authority, signer, queue, session
store, runner, public catalogue, score, or aggregation. Verification outcome,
Repository Decision, Event, and Standing remain separate axes. Reviewer count
never implies independence.

## Exact private capability

The existing local `main` capability may add only these typed commands:

- `refresh_decision_inbox`
- `select_verification_method`
- `preview_verification_record`
- `record_verification`
- `select_verification_import`
- `import_verification`
- `preview_decision`
- `execute_decision`
- `preview_recovery`
- `recover_transaction`

There is no generic shell, argv, filesystem, signer, policy, authentication,
HTTP, upload, provider, or recovery-journal command. Rust owns parsing,
canonicalization, validation, confirmations, and renderer DTO generation.

## CLI contracts

Only the selected, hash-pinned `v0.977.1` binary may run:

- `vela review inbox --repo <root> --json`
- `vela review show --repo <root> <proposal> --json`
- `vela verification record --repo <root> <proposal> ... --json`
- `vela verification import --repo <root> <record> --as <actor> --json`
- `vela review accept|reject --repo <root> <proposal> --reason <reason>
  --if-entry-root <root> --as <human-or-agent> [--session-ref <ref>] --json`
- `vela replay --repo <root> --json`, `vela review show ... --json`, and
  `vela why ... --json` for immediate post-Decision readback
- `vela recover --repo <root> <operation-id> --json`

Every argv is assembled from closed enums and bounded validated values. The
renderer cannot provide an executable, arbitrary flag, environment, root,
signer, principal, or authentication result. Child environments remain cleared
and bounded. Vela alone owns key custody, authentication, Repository policy,
authorization, transaction signing, atomic state, recovery, and JSON semantics.

## Preview and revalidation invariants

Every mutation is one-shot and requires an OS-native confirmation after Rust
rebuilds the preview. Immediately after confirmation Rust rebuilds it again and
requires exact equality before spawning Vela.

- Verification authoring binds the exact Proposal, Submission, Claim,
  Artifacts, current Git commit/tree, repository root, method path/root,
  profile, property, outcome, nonclaims, performer/attesting actor, declared
  independence, shared dependencies, selected output paths/roots, executable
  identity, and fixed argv.
- Verification import binds the exact signed envelope bytes/root, verifier,
  Proposal/Submission/Claim roots, output Artifact bindings, repository root,
  executable identity, and fixed argv.
- Decision preview comes from a fresh `vela.decision-inbox.v3` entry plus
  `review show`. It binds action, reason, performer identity/kind, optional
  source-owned session reference, Repository authority principal boundary,
  local-OS authentication boundary, Repository-authority signer boundary,
  Proposal/Submission/Verification roots, exact current `entry_root`, current
  repository root, and the Inbox's accept/reject Standing delta.
- Decision execution must pass the exact reviewed `--if-entry-root`. Rust does
  not infer eligibility, authority, independence, or successor state.
- Recovery accepts only the exact `vop_...` operation surfaced by a structured
  `repository_incomplete` refusal for the currently selected repository. It is
  never automatic and never retries a Decision.

## Verification dependence and task orientation

Each Verification remains a separate scoped observation. The UI preserves, when
present, verifier/attester, reviewer kind, model/provider/version, Method
profile/path/root, property, evidence/output roots, toolchain/environment root,
declared independence, shared dependencies, outcome, nonclaims, and requirement
role. Actor-ID difference alone never produces an "independent" label.

Source interpretation, evidential judgment, Decision, Event, and Standing use
separate sections and language. A compact current-task orientation is derived
only from current Git/Vela roots and visibly lists its task, included records,
omissions, observed time, and staleness. It is memory-only UI, not Packet,
WorkingProjection, Handover, BranchState, ResearchState, MethodQualification, or
canonical scientific state. It may not hide Standing, Decisions, unresolved
objections, scope, dependencies, counterevidence, or degraded reads.

## Structured failures

Rust requires `vela.error.v1` and branches only on exact `error.kind` and
`error.code`. Stable v0.977.1 codes handled explicitly include:

- `missing_independent_verification`
- `decision_entry_stale`
- `authority_refused`
- `repository_incomplete`
- file/path refusal codes
- unsupported Submission media/schema and invalid signature codes

Null or unknown future codes remain typed degraded failures; prose is display
only and never controls behavior. Cancellation is a Workbench state distinct
from Vela refusal. Zero-delta errors must preserve their structured operation,
retained-marker, and next-action fields. Post-commit failures are never retried;
the app immediately replays and rereads current state, presents committed facts
separately from command failure, and offers only exact structured recovery.

## Primary threats and controls

1. **Stale Decision link.** Concurrent state changes after preview. Control:
   fresh Inbox/show after confirmation plus exact `--if-entry-root`; structured
   `decision_entry_stale`; no retry.
2. **Authority confusion.** Performer is mistaken for principal or signer.
   Control: three separate preview/result fields; `--as` never selects or grants
   authority; actual result cross-checked against the previewed performer.
3. **Verification vote laundering.** Multiple actors are called independent.
   Control: preserve declared independence and shared dependencies per record;
   no counts, score, consensus, or inferred independence.
4. **Evidence/method substitution.** Method, output, Artifact, envelope, or Git
   source changes. Control: canonical contained regular files, tracked/clean
   current-commit requirements where Core requires them, byte/root checks, and
   post-confirmation preview equality.
5. **Renderer-forged eligibility or effect.** Renderer supplies roots or delta.
   Control: all roots/effects come from signed Vela JSON parsed in Rust; request
   carries only closed user choices and expected preview identity.
6. **Duplicate/post-commit Decision.** Receipt is lost after canonical commit.
   Control: immediate Inbox/replay/show/why readback, no automatic retry, and
   recovery only for an exact surfaced operation.
7. **Signer or secret leakage.** Child inherits credentials or signer material.
   Control: cleared bounded environment; Vela uses its supported local OS and
   SSH-agent interfaces without exposing secrets to renderer DTOs or logs.
8. **Real authority mutation in QA.** Test accidentally targets Math. Control:
   mutation tests require a disposable-root marker and refuse configured real
   repository roots; packaged QA creates and destroys a dedicated fixture.
9. **Capability drift.** Generic or Tranche 4 action appears. Control: exact
   handler/permission snapshots and forbidden-symbol tests.

## Falsifiers and completion gates

- stale entry root produces `decision_entry_stale` and zero state movement;
- missing independent pass blocks accept but not the separately previewed
  reject route;
- signer/policy refusal produces `authority_refused` and zero state movement;
- invalid actor, method, output, envelope, Proposal, root, action, and reason
  fail before spawn or before transaction marker as applicable;
- cancellation before Vela spawn moves no state;
- post-commit receipt loss is reconciled by readback and never retried;
- exact recovery is explicit and idempotent;
- accept and reject receipts distinguish Verification outcome from actual
  Decision/Event/Standing;
- all real-source no-mutation gates remain green;
- clearing preferences/process state changes no Repository byte;
- macOS package, 760 px layout, keyboard/a11y, capability/CSP, fixture hashes,
  and independent exact security/delta review pass before push.

## Residual risks

Vela authority actions intentionally use the current user's OS session and SSH
agent. Native confirmation is not a hardware transaction display. Same-user
TOCTOU cannot be eliminated without immutable handles. A successful canonical
commit can outlive a later process, publication, renderer, or receipt failure;
readback and exact recovery manage that condition but do not erase it. The
package remains local, unreleased, and macOS-only. The open GLib advisory still
blocks Linux/BSD qualification.

Time-frozen epistemic replay is recommended as a later evaluation program:
freeze exact state at t0, hide later evidence, capture a proposed transition,
then score it against protected evidence arriving at t1. It is not implemented
here and creates no repository, provider mutation, Protocol object, or policy.
