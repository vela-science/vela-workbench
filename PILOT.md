# External pilot packet

This pilot tests one bounded researcher loop:

```text
find a Problem -> continue in its source repository -> retain a Result and evidence
-> obtain a separately performed scoped Check -> receive an attributed Repository Decision
-> publish Git -> verify Results and History on problems.science
```

It does not test scientific novelty, field adoption, general Protocol maturity,
or institutional independence. A check, merge, signature, account, or imported
record is never a Repository Decision.

## Gate before invitations

Do not invite outside participants until all three are true:

- the exact Workbench DMG has passed the signed/notarized and clean-account
  checks in [INSTALL.md](INSTALL.md); and
- problems.science publishes a monitored private support address for account,
  privacy, accessibility, and security reports; and
- the named problems.science deployment has passed its governed release gates,
  the frozen Problem emits the exact Workbench handoff, and that deployment's
  Results and History routes can render the authority Repository projection.
  This preflight proves the path exists; it does not claim that a later pilot
  Decision is already published or fresh.

The pilot Problem, source repository and exact starting revision, authority
Repository, public readback URL, rollback deployment, and Workbench DMG digest
must be frozen in the invitation. Never include a Palomar submission-status URL,
credential, private repository name, or private scientific contents.

Name a **publication custodian** in the run header. This is an operational Git
and projection responsibility, not a fifth scientific role. The custodian must
already have ordinary Git write access to the authority repository, publish the
exact post-Decision commit through that repository's normal path, wait for the
named projection release, and verify its commit/root plus the frozen Results and
History routes. Vela authority never implies Git write access, and a Git push
never implies a Decision. A producer, verifier, authority operator, reader, or
separate maintainer may also serve as custodian only when that responsibility
and access are explicit before the run.

## Four separate invitations

Send these invitations to four named outside participants only after they agree
to the role and data boundary. The Repository-authority operator must be outside
the producer, verifier, and Vela product team for this pilot. That staffing fact
is recorded in the pilot log, not inferred by Vela or added as a Protocol object.
One person may not silently fill multiple roles.

- **Producer:** choose the frozen Problem, use **Continue locally**, work in the
  exact source checkout, and submit one bounded Result with explicit evidence
  and caveats. GitHub owns source collaboration; Workbench does not upload it.
- **Verifier:** inspect the submitted Result and perform one scoped Check. State
  the method, outcome, outputs, declared independence, shared dependencies, and
  what the Check does not establish. Actor difference alone is not independence.
- **Repository authority:** inspect the current inbox and exact roots, then make
  an attributed accept or reject Decision under that Repository's policy. The
  authority principal, authentication, signer, performer, Decision, Event, and
  Standing remain separate.
- **Reader:** begin from the public Problem without architecture help, explain
  what is known and why it is trusted, then confirm the resulting Results and
  History after publication.

Each invitation should say: “This is a usability pilot, not peer review or a
request to endorse Vela. Stop if a role, source, authority, privacy boundary, or
scientific claim is unclear.”

## Run and evidence

The operator records only consented product evidence:

- start/end time for discovery, install/first run, source match, Result capture,
  Check, Decision, Git publication, and public readback;
- retry counts, visible refusal codes, route or stage, and the participant's own
  short confusion note;
- public URLs, exact Git revisions, object/root identifiers, release/deployment
  identifiers, and the DMG digest; and
- whether the reader could state the Problem, current knowledge, trust basis,
  and next action within 30 seconds.

Do not record artifact contents, prompts, transcripts, credentials, account
tokens, private paths, or private repository names. The current product has no
third-party analytics SDK; the pilot operator owns this minimal consented log.

The run succeeds only when the source and authority repositories stay distinct,
the Check's scope and dependencies remain visible, the Decision replays, Git
publication is completed by the named custodian, the named projection commit and
root are validated, and the reader observes the same Result/Standing on that
problems.science release. A refusal is a valid pilot outcome when its
reason is exact and recovery is repeatable; it is not silently retried into
success.

## Recovery and rollback

For an interrupted Vela transaction, select the exact recent Repository and use
the signed recovery inspection; Workbench never scans for or chooses an
operation. For an application or provider failure, retain the source Git state,
authority Repository, prior notarized DMG/checksum, and prior production
deployment. Reconstruct from Git, signed Vela replay, and the public release
manifest—not Workbench process memory.
