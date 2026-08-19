import { useEffect, useMemo, useRef, useState } from "react";
import {
  AlertTriangle, ArrowUpRight, Check, ChevronRight, CircleDot, Code2, Copy,
  FileCheck2, FileOutput, FlaskConical, FolderGit2, GitBranch, HardDrive,
  Play, RefreshCw, Settings2, ShieldAlert, Square, TerminalSquare, Trash2,
} from "lucide-react";
import type {
  BootstrapDto, CommandErrorDto, EvidenceExportPreviewDto, EvidenceItemDto,
  EvidenceSourceDto, LaunchKindDto, NativeExecPreviewDto, NativeExecProfileDto,
  NativeExecResultDto, NativeOutputDto, NativeToolDto, ProblemHandoffDto,
  ProblemHandoffAuthorityDto, ProblemHandoffSourceDto, RepositorySnapshotDto, SubmissionImportPreviewDto,
  SubmissionPreviewDto, SubmissionResultDto, VelaBinaryDto, WorktreePreviewDto,
} from "./contracts/generated/ipc";
import { Badge } from "./components/ui/badge";
import { Button } from "./components/ui/button";
import { WorkbenchTabs } from "./components/ui/tabs";
import { workbench } from "./lib/workbench";
import { observeProblemHandoffUrls } from "./lib/problem-handoff";
import { TrancheThree } from "./TrancheThree";
import { OpenGaussPilot } from "./OpenGaussPilot";
import "./App.css";

function short(value: string | null | undefined, length = 9) {
  return value ? value.slice(0, length) : "Not reported";
}

function asError(value: unknown): CommandErrorDto {
  if (typeof value === "object" && value !== null && "message" in value) {
    const error = value as Partial<CommandErrorDto>;
    return { kind: error.kind ?? "unknown", message: String(error.message), detail: error.detail ?? null };
  }
  return { kind: "unknown", message: String(value), detail: null };
}

function newRunId() {
  return globalThis.crypto?.randomUUID?.() ?? `run-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function Fact({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return <div className="fact"><dt>{label}</dt><dd className={mono ? "mono" : undefined} title={value}>{value}</dd></div>;
}

function EmptyState({ choose, busy }: { choose: () => void; busy: boolean }) {
  return <section className="empty-state" aria-labelledby="empty-title">
    <div className="empty-mark"><FolderGit2 aria-hidden="true" /></div>
    <p className="eyebrow">Local repository required</p>
    <h1 id="empty-title">Continue local scientific work</h1>
    <p>Choose the Git repository that owns the work or Repository decision. Workbench inspects exact local source and supported Vela state without copying either.</p>
    <Button size="lg" onClick={choose} loading={busy}>Choose repository</Button>
    <p className="boundary-note"><HardDrive size={14} /> Private files, credentials, and evidence stay local.</p>
  </section>;
}

function Refusal({ error, dismiss }: { error: CommandErrorDto; dismiss: () => void }) {
  return <section className="refusal" role="alert"><AlertTriangle aria-hidden="true" /><div>
    <strong>Action refused · {error.kind}</strong><p>{error.message}</p>{error.detail && <code>{error.detail}</code>}
  </div><Button variant="ghost" size="sm" onClick={dismiss}>Dismiss</Button></section>;
}

function ProblemContinuation({ handoff, source, authority, chooseSource, chooseAuthority, activateSource, activateAuthority, open, dismiss, busy }: {
  handoff: ProblemHandoffDto;
  source: ProblemHandoffSourceDto | null;
  authority: ProblemHandoffAuthorityDto | null;
  chooseSource: () => void;
  chooseAuthority: () => void;
  activateSource: () => void;
  activateAuthority: () => void;
  open: () => void;
  dismiss: () => void;
  busy: boolean;
}) {
  const sourceLabel = source?.ready ? "exact source selected" : source ? "source mismatch" : "source selection required";
  const authorityLabel = authority?.ready ? "Repository selected" : authority ? "Repository mismatch" : "Repository selection required";
  return <section className="problem-continuation" role="status" aria-live="polite" aria-atomic="true" aria-labelledby="problem-continuation-title">
    <div className="problem-continuation-heading"><div><p className="eyebrow">problems.science handoff</p><h2 id="problem-continuation-title">Continue this Problem locally</h2></div><Badge tone={source?.ready && authority?.ready ? "positive" : "warning"}>{source?.ready && authority?.ready ? "source and Repository ready" : "local choices required"}</Badge></div>
    <dl className="fact-grid compact">
      <Fact label="Problem" value={handoff.problem_url} mono />
      <Fact label="Source repository" value={handoff.source_repository_url} mono />
      <Fact label="Exact source ref" value={handoff.source_revision} mono />
      <Fact label="Repository authority" value={handoff.authority_repository_url} mono />
      <Fact label="Artifact references" value={handoff.artifact_paths.length ? handoff.artifact_paths.join(", ") : "None selected"} mono />
      <Fact label="Authority effect" value={handoff.authority_effect} />
    </dl>
    <div className="continuation-status"><div><strong>1 · {sourceLabel}</strong><p>{source?.note ?? "Choose the local checkout for the exact source repository and starting revision."}</p>{source && <code>{source.repository_path}</code>}</div><div className="button-cluster">{source?.ready && <Button variant="secondary" onClick={activateSource} disabled={busy}>Return to source</Button>}<Button onClick={chooseSource} loading={busy}>{source ? "Choose another source" : "Choose local source"}</Button></div></div>
    <div className="continuation-status"><div><strong>2 · {authorityLabel}</strong><p>{authority?.note ?? "Choose the separate local Vela Repository that receives the signed Submission and owns any later Decision."}</p>{authority && <code>{authority.repository_path}</code>}</div><div className="button-cluster">{authority?.ready && <Button onClick={activateAuthority} disabled={busy}>Continue to Repository</Button>}<Button variant="secondary" onClick={chooseAuthority} loading={busy}>{authority ? "Choose another Repository" : "Choose local Repository"}</Button></div></div>
    <div className="continuation-status"><div><strong>Explicit evidence boundary</strong><p>Source bytes never move automatically. Export selected evidence into the Repository with native confirmation, then select that copied file there before Submission.</p><p>{handoff.boundary}</p></div><div className="button-cluster"><Button variant="ghost" onClick={dismiss}>Dismiss</Button><Button variant="secondary" onClick={open} disabled={busy}>Open exact Problem <ArrowUpRight size={14} /></Button></div></div>
  </section>;
}

function Orient({ snapshot }: { snapshot: RepositorySnapshotDto }) {
  const { status, claims, integration, refusal, binary } = snapshot.vela;
  const classification = snapshot.classification === "vela_repository" ? "Vela repository" : snapshot.classification === "native_integration" ? "Native integration" : "Git source";
  return <div className="surface-stack">
    <section className="section-block">
      <div className="section-heading"><div><p className="eyebrow">Observation</p><h2>Scientific state</h2></div>
        <Badge tone={status ? "positive" : integration ? "neutral" : "warning"}>{classification}</Badge>
      </div>
      <dl className="fact-grid">
        <Fact label="Classification basis" value={snapshot.classification_basis} />
        <Fact label="Vela binary" value={binary ? `${binary.version} · ${short(binary.sha256)}` : "No verified runtime selected"} mono />
        <Fact label="Runtime state" value={binary?.state.replace(/_/g, " ") ?? "Unavailable"} />
        <Fact label="Observed" value={new Date(snapshot.observed_at_unix_ms).toLocaleString()} />
      </dl>
    </section>
    {refusal && <section className="inline-refusal"><AlertTriangle /><div>
      <strong>{refusal.area} unavailable · {refusal.code ?? refusal.kind}</strong><p>{refusal.message}</p>{refusal.hint && <small>{refusal.hint}</small>}
    </div></section>}
    {status && <section className="section-block">
      <div className="section-heading"><div><p className="eyebrow">Vela status</p><h2>{status.repository_name}</h2></div>
        <Badge tone={status.blocker_count === 0 ? "positive" : "warning"}>{status.blocker_count} blockers</Badge></div>
      <dl className="fact-grid compact">
        <Fact label="Repository id" value={status.repository_id} mono /><Fact label="Replay / strict" value={`${status.replay} / ${status.strict}`} />
        <Fact label="Repository root" value={short(status.repository_root, 16)} mono /><Fact label="Projection commit" value={short(status.repository_commit, 12)} mono />
        <Fact label="Accepted Results" value={String(status.counts.accepted_claims)} /><Fact label="Pending review" value={String(status.counts.pending_review)} />
      </dl>
      <div className="command-strip"><span>Current work</span><code>{status.work_command || status.work_mode}</code><small>{status.work_note}</small></div>
    </section>}
    {claims.length > 0 && <section className="section-block">
      <div className="section-heading"><div><p className="eyebrow">Accepted and current</p><h2>Results</h2></div><span className="count-label">{claims.length} rows</span></div>
      <div className="table-wrap"><table><thead><tr><th>Result ID</th><th>Standing</th><th>Result</th><th>Exact root</th></tr></thead><tbody>
        {claims.map((claim) => <tr key={claim.claim_root}><td><strong>{claim.claim_id}</strong><small>{claim.origin_era}</small></td>
          <td><Badge tone={claim.standing === "accepted" ? "positive" : "neutral"}>{claim.standing}</Badge></td>
          <td className="assertion">{claim.assertion ?? claim.unreadable_reason ?? "Unreadable"}</td><td><code>{short(claim.claim_root)}</code></td></tr>)}
      </tbody></table></div>
    </section>}
    {integration && <section className="section-block">
      <div className="section-heading"><div><p className="eyebrow">Source-native bridge</p><h2>Integration manifest</h2></div><Badge>authority effect: {integration.authority_effect}</Badge></div>
      <dl className="fact-grid compact"><Fact label="Repository" value={integration.repository} /><Fact label="Revision" value={short(integration.revision, 12)} mono />
        <Fact label="Manifest root" value={short(integration.manifest_root, 16)} mono /><Fact label="Documents checked" value={String(integration.documents_checked)} /></dl>
      <p className="boundary-copy">This manifest locates source-native material. It does not establish scientific acceptance, authority, or Vela state.</p>
    </section>}
  </div>;
}

const profiles: Array<{ id: NativeExecProfileDto; label: string; note: string; needsTool: boolean }> = [
  { id: "git_diff_check", label: "Git diff check", note: "Fixed safe diff argv, no external diff", needsTool: false },
  { id: "lean_build", label: "Lean build", note: "Fixed argv: lake build", needsTool: true },
  { id: "cargo_test", label: "Cargo tests", note: "Fixed argv: cargo test --locked", needsTool: true },
  { id: "bun_test", label: "Bun tests", note: "Fixed argv: bun test", needsTool: true },
];

type ExecuteProps = {
  snapshot: RepositorySnapshotDto; launch: (kind: LaunchKindDto) => void; launching: LaunchKindDto | null;
  worktreeRef: string; setWorktreeRef: (value: string) => void; worktreePreview: WorktreePreviewDto | null;
  previewWorktree: () => void; createWorktree: () => void; profile: NativeExecProfileDto;
  setProfile: (value: NativeExecProfileDto) => void; tool: NativeToolDto | null; selectTool: () => void;
  execPreview: NativeExecPreviewDto | null; previewExec: () => void; runExec: () => void; cancelExec: () => void;
  running: boolean; result: NativeExecResultDto | null; busy: boolean;
  evidence: EvidenceItemDto[];
};

function Execute(props: ExecuteProps) {
  const selectedProfile = profiles.find((item) => item.id === props.profile)!;
  const launchers: Array<[LaunchKindDto, string, typeof Code2]> = [
    ["terminal", "Terminal", TerminalSquare], ["cursor", "Cursor", Code2],
    ["visual_studio_code", "VS Code", Code2], ["forge", "Forge", ArrowUpRight],
  ];
  return <div className="surface-stack">
    <section className="section-block">
      <div className="section-heading"><div><p className="eyebrow">Local work</p><h2>Run one reviewed check</h2></div><Badge>one active process</Badge></div>
      <div className="profile-grid" role="radiogroup" aria-label="Reviewed native command profile">
        {profiles.map((item) => <button key={item.id} role="radio" aria-checked={props.profile === item.id} data-selected={props.profile === item.id || undefined} onClick={() => props.setProfile(item.id)} disabled={props.running}>
          <strong>{item.label}</strong><small>{item.note}</small>
        </button>)}
      </div>
      <div className="action-line"><div><strong>{selectedProfile.label}</strong><small>{props.tool ? `${props.tool.path} · ${short(props.tool.sha256, 18)}` : selectedProfile.needsTool ? "Select the exact executable" : "Select the fixed system Git identity"}</small></div>
        <Button variant="secondary" onClick={props.selectTool} disabled={props.running}>{props.tool ? "Re-select tool" : "Select tool"}</Button>
        <Button onClick={props.previewExec} disabled={!props.tool || props.running}>Review command</Button></div>
      {props.execPreview && <div className="review-panel" aria-label="Native execution preview"><div className="review-header"><div><p className="eyebrow">Exact execution preview</p><h3>{props.execPreview.label}</h3></div><Badge tone="warning">not sandboxed</Badge></div>
        <div className="trust-warning"><ShieldAlert /><p>{props.execPreview.trust_warning}</p></div>
        <dl className="fact-grid compact"><Fact label="Executable" value={`${props.execPreview.executable.path} · ${props.execPreview.executable.sha256}`} mono /><Fact label="Fixed argv" value={props.execPreview.argv.join(" ")} mono /><Fact label="Working directory" value={props.execPreview.working_directory} mono /><Fact label="Source commit" value={props.execPreview.source_commit} mono /><Fact label="Timeout" value={`${Math.round(props.execPreview.timeout_ms / 1000)} seconds`} /><Fact label="Capture bounds" value={`${props.execPreview.max_stdout_bytes} stdout · ${props.execPreview.max_stderr_bytes} stderr`} /></dl>
        <details><summary>Bounded environment ({props.execPreview.environment.length})</summary><pre>{props.execPreview.environment.map((entry) => `${entry.name}=${entry.value}`).join("\n")}</pre></details>
        <div className="review-actions">{props.running ? <Button variant="secondary" onClick={props.cancelExec}><Square size={13} />Cancel process tree</Button> : <Button onClick={props.runExec}><Play size={13} />Start explicitly</Button>}</div></div>}
      {props.running && <div className="running-state" role="status" aria-live="polite"><CircleDot className="pulse" />Running bounded profile. Cancellation limits lifetime and capture only.</div>}
      {props.result && <div className="result-strip" data-state={props.result.state}><strong>{props.result.state.replace(/_/g, " ")}</strong><span>exit {props.result.exit_code ?? "none"}</span><code>{props.result.run_id}</code><span>{props.result.stdout.size + props.result.stderr.size} captured bytes</span></div>}
    </section>
    <OpenGaussPilot key={props.snapshot.path} snapshot={props.snapshot} evidence={props.evidence} result={props.result} />
    <section className="section-block two-column"><div><div className="section-heading"><div><p className="eyebrow">Controlled Git operation</p><h2>Detached worktree</h2></div></div>
      <label className="field"><span>Exact target ref</span><input value={props.worktreeRef} onChange={(event) => props.setWorktreeRef(event.target.value)} placeholder="HEAD or refs/heads/main" /></label><p className="boundary-copy">Choose an existing empty folder. The selected checkout is never switched or reset.</p><Button variant="secondary" onClick={props.previewWorktree} disabled={!props.worktreeRef.trim() || props.busy}>Choose destination and preview</Button>
      {props.worktreePreview && <div className="compact-review"><strong>{props.worktreePreview.target_ref} → {short(props.worktreePreview.target_commit, 12)}</strong><code>{props.worktreePreview.destination}</code><p>{props.worktreePreview.warning}</p><details><summary>Exact Git argv</summary><pre>{props.worktreePreview.command.map((arg) => JSON.stringify(arg)).join(" ")}</pre></details><small>Rollback: {props.worktreePreview.rollback.map((arg) => JSON.stringify(arg)).join(" ")}</small><Button onClick={props.createWorktree}>Create after native confirmation</Button></div>}</div>
      <div><div className="section-heading"><div><p className="eyebrow">Git-owned</p><h2>Existing worktrees</h2></div><span className="count-label">{props.snapshot.git.worktrees.length}</span></div><div className="row-list">{props.snapshot.git.worktrees.map((tree) => <div className="data-row" key={tree.path}><GitBranch /><div><strong>{tree.branch ?? "Detached HEAD"}</strong><code>{tree.path}</code></div><span><code>{short(tree.head)}</code>{tree.locked && <Badge tone="warning">locked</Badge>}</span></div>)}</div></div>
    </section>
    <section className="section-block handoff-block source-handoff"><div><p className="eyebrow">Explicit handoff</p><h2>Open exact source elsewhere</h2><p>Workbench passes the selected Git root or a sanitized HTTPS forge locator. The receiving tool owns every subsequent action.</p></div><div className="button-cluster">{launchers.map(([kind, label, Icon]) => <Button key={kind} variant={kind === "terminal" ? "primary" : "secondary"} onClick={() => props.launch(kind)} loading={props.launching === kind}><Icon size={14} />{label}</Button>)}</div></section>
    <section className="section-block two-column"><div><div className="section-heading"><div><p className="eyebrow">Entire-owned references</p><h2>Checkpoints</h2></div><Badge tone={props.snapshot.entire.cli_available ? "positive" : "neutral"}>{props.snapshot.entire.cli_available ? "CLI available" : "CLI unavailable"}</Badge></div>{props.snapshot.git.entire_checkpoints.length ? <div className="row-list">{props.snapshot.git.entire_checkpoints.map((item) => <div className="data-row" key={`${item.commit}-${item.checkpoint_id}`}><CircleDot /><div><strong>{item.checkpoint_id}</strong><code>{short(item.commit)}</code></div></div>)}</div> : <p className="empty-copy">No Entire-Checkpoint trailers found. No transcript or substitute session store was copied.</p>}</div>
      <div><div className="section-heading"><div><p className="eyebrow">Git-owned</p><h2>Remotes</h2></div><span className="count-label">{props.snapshot.git.remotes.length}</span></div><div className="row-list">{props.snapshot.git.remotes.map((remote, index) => <div className="data-row" key={`${remote.name}-${remote.operation}-${index}`}><HardDrive /><div><strong>{remote.name} · {remote.operation}</strong><code>{remote.url}</code></div></div>)}</div></div></section>
  </div>;
}

type CaptureView = { source: EvidenceSourceDto; name: string; sha256: string; size: number; media: string; kind: string; commit: string; contentUtf8: string | null; contentBase64: string };
function outputView(result: NativeExecResultDto, output: NativeOutputDto): CaptureView { return { source: { source: "command_output", run_id: result.run_id, stream: output.stream }, name: `${result.run_id}-${output.stream}.txt`, sha256: output.sha256, size: output.size, media: "text/plain", kind: "command-output", commit: result.source_commit, contentUtf8: output.content_utf8, contentBase64: output.content_base64 }; }
function itemView(item: EvidenceItemDto): CaptureView { return { source: item.source, name: item.display_name, sha256: item.sha256, size: item.size, media: item.media_type, kind: item.kind_hint, commit: item.source_commit, contentUtf8: item.content_utf8, contentBase64: item.content_base64 }; }

type CaptureProps = { evidence: EvidenceItemDto[]; result: NativeExecResultDto | null; selected: CaptureView | null; setSelected: (value: CaptureView) => void; chooseFile: () => void; exclusions: string; setExclusions: (value: string) => void; redaction: boolean; setRedaction: (value: boolean) => void; derivedText: string; setDerivedText: (value: string) => void; previewExport: () => void; exportPreview: EvidenceExportPreviewDto | null; exportEvidence: () => void; busy: boolean };
function Capture(props: CaptureProps) {
  const outputs = props.result ? [outputView(props.result, props.result.stdout), outputView(props.result, props.result.stderr)] : [];
  return <div className="capture-layout"><section className="section-block capture-index"><div className="section-heading"><div><p className="eyebrow">Explicit selection only</p><h2>Captured evidence</h2></div><Button variant="secondary" onClick={props.chooseFile} loading={props.busy}><FlaskConical size={14} />Choose one file</Button></div>
    {!props.evidence.length && !outputs.length && <div className="teaching-empty"><FileCheck2 /><strong>No evidence selected</strong><p>Choose one repository-contained file, or run a reviewed command and select one captured stream. Workbench never scans the repository.</p></div>}
    <div className="evidence-list">{props.evidence.map((item) => { const view = itemView(item); return <button key={item.sha256 + item.display_name} data-selected={props.selected?.sha256 === item.sha256 || undefined} onClick={() => props.setSelected(view)}><FileOutput /><span><strong>{item.display_name}</strong><small>{item.media_type} · {item.size} bytes · private</small></span><code>{short(item.sha256, 18)}</code></button>; })}{outputs.map((view) => <button key={view.name} data-selected={props.selected?.name === view.name || undefined} onClick={() => props.setSelected(view)}><TerminalSquare /><span><strong>{view.name}</strong><small>captured command stream · {view.size} bytes · private</small></span><code>{short(view.sha256, 18)}</code></button>)}</div></section>
    <section className="section-block capture-detail">{!props.selected ? <div className="teaching-empty"><Copy /><strong>Select exact bytes to review</strong><p>Digest, size, source revision, and the full bounded byte representation appear here before export.</p></div> : <><div className="section-heading"><div><p className="eyebrow">Private local bytes</p><h2>{props.selected.name}</h2></div><Badge tone="warning">private</Badge></div><dl className="fact-grid compact"><Fact label="Digest" value={props.selected.sha256} mono /><Fact label="Size / media" value={`${props.selected.size} bytes · ${props.selected.media}`} /><Fact label="Source revision" value={props.selected.commit} mono /></dl>
      <label className="field"><span>Exact bounded bytes</span><textarea className="byte-view" readOnly value={props.selected.contentUtf8 ?? props.selected.contentBase64} aria-label="Exact selected evidence bytes" /></label><div className="redaction-grid"><label className="check-field"><input type="checkbox" checked={props.redaction} disabled={!props.selected.contentUtf8} onChange={(event) => props.setRedaction(event.target.checked)} /><span>Create a derived redacted text output</span></label><label className="field"><span>Exclusions / redactions (one per line)</span><textarea value={props.exclusions} onChange={(event) => props.setExclusions(event.target.value)} placeholder="Removed private participant identifiers" /></label></div>{props.redaction && <label className="field"><span>Exact derived UTF-8 output</span><textarea className="byte-view" value={props.derivedText} onChange={(event) => props.setDerivedText(event.target.value)} /></label>}<p className="boundary-copy">Redaction creates a new derived file. The selected source evidence is never edited.</p><Button onClick={props.previewExport} disabled={props.redaction && (!props.exclusions.trim() || !props.derivedText)}>Choose destination and review export</Button>
      {props.exportPreview && <div className="compact-review"><strong>{props.exportPreview.derived ? "Derived redacted output" : "Exact local copy"}</strong><code>{props.exportPreview.destination}</code><p>{props.exportPreview.output_sha256} · {props.exportPreview.output_size} bytes</p><small>{props.exportPreview.warning}</small><Button onClick={props.exportEvidence}>Export after native confirmation</Button></div>}</>}</section></div>;
}

type ClaimType = "computational" | "theoretical" | "empirical" | "negative" | "contradiction";
type ReviewProps = { snapshot: RepositorySnapshotDto; handoff: ProblemHandoffDto | null; handoffSource: ProblemHandoffSourceDto | null; handoffAuthority: ProblemHandoffAuthorityDto | null; selectedIsAuthority: boolean; evidence: EvidenceItemDto[]; result: NativeExecResultDto | null; assertion: string; setAssertion: (value: string) => void; claimType: ClaimType; setClaimType: (value: ClaimType) => void; producer: string; setProducer: (value: string) => void; caveat: string; setCaveat: (value: string) => void; requirement: string; setRequirement: (value: string) => void; selectedArtifacts: string[]; toggleArtifact: (digest: string) => void; includeCheck: boolean; setIncludeCheck: (value: boolean) => void; preview: SubmissionPreviewDto | null; previewDraft: () => void; submitDraft: () => void; importPreview: SubmissionImportPreviewDto | null; chooseImport: () => void; importSubmission: () => void; submissionResult: SubmissionResultDto | null; busy: boolean };
function ReviewDraft(props: ReviewProps) {
  const runtimeReady = props.snapshot.vela.binary?.state === "signed_runtime_baseline";
  return <div className="surface-stack">{!runtimeReady && <section className="inline-refusal"><AlertTriangle /><div><strong>Submission intake degraded</strong><p>Select the exact signed Vela v0.977.3 runtime. Capture and local export remain available.</p></div></section>}
    {props.handoff && props.handoffSource?.ready && <section className="section-block"><div className="section-heading"><div><p className="eyebrow">Problem-to-Repository handoff</p><h2>{props.selectedIsAuthority ? "Prepare the Repository copy" : "Source work remains local"}</h2></div><Badge tone={props.selectedIsAuthority && props.handoffAuthority?.ready ? "positive" : "warning"}>{props.selectedIsAuthority ? "authority Repository selected" : "source selected"}</Badge></div><dl className="fact-grid compact"><Fact label="Exact Problem" value={props.handoff.problem_url} mono /><Fact label="Source checkout" value={props.handoffSource.repository_path} mono /><Fact label="Starting source ref" value={props.handoffSource.source_revision} mono /><Fact label="Authority Repository" value={props.handoffAuthority?.repository_path ?? props.handoff.authority_repository_url} mono /></dl><p className="boundary-copy">Scientific work stays in the source repository. Workbench never transfers it implicitly. Export each exact selected Artifact into the authority Repository with native confirmation, continue to that Repository, and select the copied file before signing the Submission.</p></section>}
    <section className="section-block review-columns"><div className="draft-form"><div className="section-heading"><div><p className="eyebrow">Result handoff</p><h2>Prepare a Result for submission</h2></div><Badge>creates a pending Proposal</Badge></div><label className="field"><span>Bounded result</span><textarea value={props.assertion} onChange={(event) => props.setAssertion(event.target.value)} placeholder="State one scientifically reusable bounded result" /></label><div className="form-row"><label className="field"><span>Producer attribution</span><input value={props.producer} onChange={(event) => props.setProducer(event.target.value)} placeholder="agent:researcher" /></label><label className="field"><span>Result type</span><select value={props.claimType} onChange={(event) => props.setClaimType(event.target.value as ClaimType)}><option value="theoretical">theoretical</option><option value="computational">computational</option><option value="empirical">empirical</option><option value="negative">negative result</option><option value="contradiction">contradiction</option></select></label><label className="field"><span>Replayability</span><select defaultValue="exact"><option value="exact">exact</option></select></label></div><label className="field"><span>Required caveat</span><textarea value={props.caveat} onChange={(event) => props.setCaveat(event.target.value)} placeholder="What this does not establish" /></label><label className="field"><span>Required independent Check</span><input value={props.requirement} onChange={(event) => props.setRequirement(event.target.value)} placeholder="Replay the exact Artifact" /></label>
      <fieldset className="artifact-fieldset"><legend>Explicit captured Artifacts</legend>{props.evidence.length ? props.evidence.map((item) => <label key={item.sha256}><input type="checkbox" checked={props.selectedArtifacts.includes(item.sha256)} onChange={() => props.toggleArtifact(item.sha256)} /><span><strong>{item.display_name}</strong><small>{item.sha256} · {item.size} bytes</small></span></label>) : <p>No repository file evidence is selected.</p>}</fieldset>{props.result && <label className="check-field"><input type="checkbox" checked={props.includeCheck} onChange={(event) => props.setIncludeCheck(event.target.checked)} /><span>Include producer-reported {props.result.producer_check_method}:{props.result.producer_check_outcome}</span></label>}<Button onClick={props.previewDraft} disabled={!runtimeReady || !props.assertion.trim() || !props.producer.trim() || !props.caveat.trim() || !props.selectedArtifacts.length} loading={props.busy}>Review exact CLI operation</Button></div>
      <div className="draft-review"><div className="section-heading"><div><p className="eyebrow">Authority boundary</p><h2>Review before import</h2></div></div>{!props.preview ? <div className="teaching-empty"><FileCheck2 /><strong>No draft preview</strong><p>Preview freezes exact argv, Repository revision, Artifact digests, producer checks, and the pinned Vela identity. Any stale input refuses import.</p></div> : <div className="review-panel"><Badge tone="warning">no authority effect</Badge><h3>{props.preview.draft.assertion}</h3><p>{props.preview.authority_boundary}</p><pre className="argv-view">{props.preview.argv.map((arg) => JSON.stringify(arg)).join(" ")}</pre><dl className="fact-grid compact"><Fact label="Repository commit" value={props.preview.source_commit} mono /><Fact label="Artifacts" value={`${props.preview.draft.artifacts.length} · ${props.preview.artifact_total_bytes} bytes`} /><Fact label="Producer" value={props.preview.draft.producer} /></dl><p className="boundary-copy">{props.preview.warning}</p><Button onClick={props.submitDraft}>Import after native confirmation</Button></div>}</div></section>
    <section className="section-block handoff-block"><div><p className="eyebrow">Portable producer boundary</p><h2>Import an existing signed envelope</h2><p>Choose one exact signed Submission v3 file. Rust previews its closed bounded shape; the signed Vela CLI verifies its signature and repository preconditions.</p></div><Button variant="secondary" onClick={props.chooseImport} disabled={!runtimeReady}>Choose signed envelope</Button></section>
    {props.importPreview && <section className="section-block review-panel"><div className="review-header"><div><p className="eyebrow">Signed import preview</p><h3>{props.importPreview.assertion}</h3></div><Badge>{props.importPreview.payload_type}</Badge></div><dl className="fact-grid compact"><Fact label="Producer" value={props.importPreview.producer} /><Fact label="Envelope digest" value={props.importPreview.envelope_sha256} mono /><Fact label="Envelope size" value={`${props.importPreview.envelope_size} bytes`} /><Fact label="Source commit" value={props.importPreview.source_commit} mono /><Fact label="Artifacts" value={String(props.importPreview.artifacts.length)} /><Fact label="Vela binary" value={props.importPreview.vela_binary_sha256} mono /></dl><p>{props.importPreview.authority_boundary}</p><Button onClick={props.importSubmission}>Import signed bytes after native confirmation</Button></section>}
    {props.submissionResult && <section className="section-block success-receipt" aria-live="polite"><Check /><div><p className="eyebrow">Submission retained</p><h2>{props.submissionResult.submission_id}</h2><p>Pending Proposal {props.submissionResult.proposal_id}. Accepted-event delta {props.submissionResult.accepted_event_delta}; accepted state changed: {String(props.submissionResult.accepted_state_changed)}.</p><code>{props.submissionResult.submission_root}</code></div><Badge tone="positive">{props.submissionResult.publication_state}</Badge></section>}
    <section className="section-block boundary-ledger"><div><strong>Producer</strong><p>Signs the Submission and reports bounded checks.</p></div><div><strong>Repository authority</strong><p>Remains policy-governed and separate from performer kind.</p></div><div><strong>Next surface</strong><p>Scoped Verification and an attributed Decision remain separate from this draft.</p></div></section>
  </div>;
}

export default function App() {
  const [boot, setBoot] = useState<BootstrapDto | null>(null); const [selected, setSelected] = useState<RepositorySnapshotDto | null>(null); const [identity, setIdentity] = useState<VelaBinaryDto | null>(null); const [error, setError] = useState<CommandErrorDto | null>(null); const [busy, setBusy] = useState(false); const [launching, setLaunching] = useState<LaunchKindDto | null>(null);
  const [handoff, setHandoff] = useState<ProblemHandoffDto | null>(null); const [handoffSource, setHandoffSource] = useState<ProblemHandoffSourceDto | null>(null); const [handoffAuthority, setHandoffAuthority] = useState<ProblemHandoffAuthorityDto | null>(null);
  const [worktreeRef, setWorktreeRef] = useState("HEAD"); const [worktreePreview, setWorktreePreview] = useState<WorktreePreviewDto | null>(null); const [profile, setProfileState] = useState<NativeExecProfileDto>("git_diff_check"); const [tool, setTool] = useState<NativeToolDto | null>(null); const [execPreview, setExecPreview] = useState<NativeExecPreviewDto | null>(null); const [running, setRunning] = useState(false); const [runId, setRunId] = useState<string | null>(null); const [execResult, setExecResult] = useState<NativeExecResultDto | null>(null);
  const [evidence, setEvidence] = useState<EvidenceItemDto[]>([]); const [capture, setCapture] = useState<CaptureView | null>(null); const [exclusions, setExclusions] = useState(""); const [redaction, setRedaction] = useState(false); const [derivedText, setDerivedText] = useState(""); const [exportPreview, setExportPreview] = useState<EvidenceExportPreviewDto | null>(null);
  const [assertion, setAssertion] = useState(""); const [claimType, setClaimTypeState] = useState<ClaimType>("computational"); const [producer, setProducer] = useState("agent:researcher"); const [caveat, setCaveat] = useState(""); const [requirement, setRequirement] = useState(""); const [selectedArtifacts, setSelectedArtifacts] = useState<string[]>([]); const [includeCheck, setIncludeCheck] = useState(true); const [submissionPreview, setSubmissionPreview] = useState<SubmissionPreviewDto | null>(null); const [importPreview, setImportPreview] = useState<SubmissionImportPreviewDto | null>(null); const [submissionResult, setSubmissionResult] = useState<SubmissionResultDto | null>(null);
  const activeUiRun = useRef<string | null>(null);
  const handoffGeneration = useRef(0);
  const handoffRevision = useRef<string | null>(null);
  useEffect(() => { workbench.bootstrap().then(setBoot).catch((value) => setError(asError(value))); }, []);
  useEffect(() => {
    let active = true; let unlisten: (() => void) | undefined;
    observeProblemHandoffUrls((url) => {
      const generation = ++handoffGeneration.current;
      workbench.reviewProblemHandoff(url).then((value) => {
        if (!active || generation !== handoffGeneration.current) return;
        handoffRevision.current = value.source_revision; setHandoff(value); setHandoffSource(null); setHandoffAuthority(null); resetSourceBoundState(value.source_revision); setError(null);
      }).catch((value) => { if (active && generation === handoffGeneration.current) setError(asError(value)); });
    }).then((stop) => { if (active) unlisten = stop; else stop(); }).catch((value) => { if (active) setError(asError(value)); });
    return () => { active = false; unlisten?.(); };
  }, []);
  const recents = boot?.preferences.recent_repositories ?? []; const sourceBadge = useMemo(() => selected ? `${selected.git.branch ?? "detached"} · ${selected.git.dirty ? `${selected.git.changed_paths} changed` : "clean"}` : "", [selected]);
  function guard(task: () => Promise<void>) { setBusy(true); setError(null); task().catch((value) => setError(asError(value))).finally(() => setBusy(false)); }
  function resetRepositoryBoundState(targetRef = "HEAD") { setWorktreeRef(targetRef); setWorktreePreview(null); setExecPreview(null); setExecResult(null); setEvidence([]); setCapture(null); setExclusions(""); setRedaction(false); setDerivedText(""); setExportPreview(null); setSelectedArtifacts([]); setIncludeCheck(true); setSubmissionPreview(null); setImportPreview(null); setSubmissionResult(null); }
  function resetSourceBoundState(targetRef = "HEAD") { resetRepositoryBoundState(targetRef); setAssertion(""); setClaimTypeState("computational"); setProducer("agent:researcher"); setCaveat(""); setRequirement(""); }
  function choose() { guard(async () => { const result = await workbench.selectRepository(); if (result) { setSelected(result); resetSourceBoundState(); setBoot(await workbench.bootstrap()); } }); }
  function chooseHandoffSource() { if (!handoff) return; const expected = handoff; const generation = handoffGeneration.current; guard(async () => { const result = await workbench.selectRepository(); if (!result) return; setSelected(result); resetSourceBoundState(handoffRevision.current ?? "HEAD"); setBoot(await workbench.bootstrap()); if (generation !== handoffGeneration.current) return; const reviewed = await workbench.reviewProblemHandoffSource(result.path, expected); if (generation === handoffGeneration.current) setHandoffSource(reviewed); }); }
  function chooseHandoffAuthority() { if (!handoff) return; const expected = handoff; const generation = handoffGeneration.current; guard(async () => { const result = await workbench.selectRepository(); if (!result) return; setSelected(result); resetRepositoryBoundState(); setBoot(await workbench.bootstrap()); if (generation !== handoffGeneration.current) return; const reviewed = await workbench.reviewProblemHandoffAuthority(result.path, expected); if (generation === handoffGeneration.current) setHandoffAuthority(reviewed); }); }
  function activateHandoffRepository(role: "source" | "authority") { if (!handoff) return; const expected = handoff; const generation = handoffGeneration.current; const path = role === "source" ? handoffSource?.repository_path : handoffAuthority?.repository_path; if (!path) return; if (role === "source") setHandoffSource(null); else setHandoffAuthority(null); guard(async () => { const result = await workbench.inspectRepository(path); resetRepositoryBoundState(role === "source" ? expected.source_revision : "HEAD"); setSelected(result); const reviewed = role === "source" ? await workbench.reviewProblemHandoffSource(path, expected) : await workbench.reviewProblemHandoffAuthority(path, expected); if (generation !== handoffGeneration.current) return; if (role === "source") setHandoffSource(reviewed as ProblemHandoffSourceDto); else setHandoffAuthority(reviewed as ProblemHandoffAuthorityDto); }); }
  function inspect(path: string) { const expected = handoff; const generation = handoffGeneration.current; const sourceBound = Boolean(expected && handoffSource?.repository_path === path); const authorityBound = Boolean(expected && handoffAuthority?.repository_path === path); if (sourceBound) setHandoffSource(null); if (authorityBound) setHandoffAuthority(null); guard(async () => { const result = await workbench.inspectRepository(path); if (sourceBound || authorityBound) resetRepositoryBoundState(sourceBound ? expected?.source_revision : "HEAD"); else resetSourceBoundState(); setSelected(result); if (!expected || generation !== handoffGeneration.current) return; if (sourceBound) { const reviewed = await workbench.reviewProblemHandoffSource(path, expected); if (generation === handoffGeneration.current) setHandoffSource(reviewed); } if (authorityBound) { const reviewed = await workbench.reviewProblemHandoffAuthority(path, expected); if (generation === handoffGeneration.current) setHandoffAuthority(reviewed); } }); }
  function selectVela() { guard(async () => { const result = await workbench.selectVelaBinary(); if (result) { setIdentity(result); setBoot(await workbench.bootstrap()); if (selected) setSelected(await workbench.inspectRepository(selected.path)); } }); }
  function clear() { activeUiRun.current = null; setRunId(null); setRunning(false); guard(async () => { const preferences = await workbench.clearRecents(); setBoot((current) => current ? { ...current, preferences } : current); setIdentity(null); setSelected(null); resetSourceBoundState(); setTool(null); }); }
  function launch(kind: LaunchKindDto) { if (!selected) return; setLaunching(kind); setError(null); workbench.launchRepository(selected.path, kind).catch((value) => setError(asError(value))).finally(() => setLaunching(null)); }
  function openProblem() { if (!handoff) return; guard(async () => { await workbench.openProblemHandoff(handoff); }); }
  function dismissHandoff() { handoffGeneration.current += 1; handoffRevision.current = null; setHandoff(null); setHandoffSource(null); setHandoffAuthority(null); setWorktreeRef("HEAD"); }
  function setProfile(value: NativeExecProfileDto) { setProfileState(value); setTool(null); setExecPreview(null); }
  function selectTool() { guard(async () => { const result = await workbench.selectNativeTool(profile); if (result) { setTool(result); setExecPreview(null); } }); }
  function previewExec() { if (!selected) return; guard(async () => setExecPreview(await workbench.previewNativeExec(selected.path, profile))); }
  async function runExec() { if (!execPreview) return; const id = newRunId(); activeUiRun.current = id; setRunId(id); setRunning(true); setError(null); try { const result = await workbench.runNativeExec(id, execPreview); if (activeUiRun.current !== id) return; setExecResult(result); setEvidence([]); setCapture(null); setExportPreview(null); setSelectedArtifacts([]); setSubmissionPreview(null); setImportPreview(null); setSubmissionResult(null); if (selected) { const refreshed = await workbench.inspectRepository(selected.path); if (activeUiRun.current !== id) return; if (refreshed.git.head_commit !== result.source_commit || refreshed.git.head_tree !== result.source_tree) setExecPreview(null); setSelected(refreshed); } } catch (value) { if (activeUiRun.current === id) setError(asError(value)); } finally { if (activeUiRun.current === id) { activeUiRun.current = null; setRunning(false); setRunId(null); } } }
  function cancelExec() { if (!runId) return; workbench.cancelNativeExec(runId).catch((value) => setError(asError(value))); }
  function previewWorktree() { if (!selected) return; guard(async () => { const result = await workbench.previewWorktree(selected.path, worktreeRef); if (result) setWorktreePreview(result); }); }
  function createWorktree() { if (!worktreePreview) return; guard(async () => { const result = await workbench.createWorktree(worktreePreview); if (result) { resetSourceBoundState(); setSelected(result.repository); setBoot(await workbench.bootstrap()); } }); }
  function chooseEvidence() { if (!selected) return; guard(async () => { const item = await workbench.selectEvidenceFile(selected.path); if (item) { setEvidence((current) => [...current.filter((old) => old.sha256 !== item.sha256), item]); setCapture(itemView(item)); setDerivedText(item.content_utf8 ?? ""); setSelectedArtifacts((current) => current.includes(item.sha256) ? current : [...current, item.sha256]); setExportPreview(null); } }); }
  function chooseCapture(value: CaptureView) { setCapture(value); setDerivedText(value.contentUtf8 ?? ""); setExclusions(""); setRedaction(false); setExportPreview(null); }
  function previewEvidenceExport() { if (!selected || !capture) return; guard(async () => { const result = await workbench.previewEvidenceExport(selected.path, { source: capture.source, expected_sha256: capture.sha256, exclusions: exclusions.split("\n").map((item) => item.trim()).filter(Boolean), redaction_confirmed: redaction, derived_utf8: redaction ? derivedText : null }); if (result) setExportPreview(result); }); }
  function exportEvidence() { if (!selected || !exportPreview) return; guard(async () => { const result = await workbench.exportEvidence(selected.path, exportPreview); if (result) setExportPreview(null); }); }
  function toggleArtifact(digest: string) { setSelectedArtifacts((current) => current.includes(digest) ? current.filter((item) => item !== digest) : [...current, digest]); setSubmissionPreview(null); }
  function setClaimType(value: ClaimType) { setClaimTypeState(value); setSubmissionPreview(null); }
  function previewDraft() { if (!selected) return; const artifacts = evidence.filter((item) => selectedArtifacts.includes(item.sha256)).flatMap((item) => item.source.source === "local_file" ? [{ path: item.source.repository_relative_path, kind: item.kind_hint, sha256: item.sha256, size: item.size }] : []); guard(async () => setSubmissionPreview(await workbench.previewSubmissionDraft(selected.path, { assertion, claim_type: claimType, conditions: [], replayability: "exact", artifacts, caveats: [caveat], producer_check_run_ids: includeCheck && execResult ? [execResult.run_id] : [], verification_requirements: requirement.trim() ? [requirement] : [], source_run: execResult?.run_id ?? null, producer }))); }
  function submitDraft() { if (!submissionPreview) return; guard(async () => { const result = await workbench.submitSubmissionDraft(submissionPreview); if (result) { setSubmissionResult(result); setSubmissionPreview(null); if (selected) setSelected(await workbench.inspectRepository(selected.path)); } }); }
  function chooseImport() { if (!selected) return; guard(async () => { const result = await workbench.selectSubmissionImport(selected.path); if (result) setImportPreview(result); }); }
  function importSubmission() { if (!importPreview) return; guard(async () => { const result = await workbench.importSubmission(importPreview); if (result) { setSubmissionResult(result); setImportPreview(null); if (selected) setSelected(await workbench.inspectRepository(selected.path)); } }); }
  async function refreshAfterAuthorityMutation() { if (selected) setSelected(await workbench.inspectRepository(selected.path)); }
  const selectedIsSource = Boolean(selected && handoffSource?.ready && selected.path === handoffSource.repository_path);
  const selectedIsAuthority = Boolean(selected && handoffAuthority?.ready && selected.path === handoffAuthority.repository_path);
  const selectedRoleLabel = selectedIsSource ? "Selected Problem source" : selectedIsAuthority ? "Selected authority Repository" : "Selected local repository";
  return <div className="app-shell"><aside className="repo-rail"><div className="brand"><div className="brand-mark">V</div><div><strong>Vela</strong><span>Workbench</span></div></div><div className="rail-section"><div className="rail-label"><span>Repositories</span>{(recents.length > 0 || boot?.preferences.vela_binary_path) && <button onClick={clear} aria-label="Clear local repository, tool, run, and evidence choices" title="Clear all local choices"><Trash2 size={13} /></button>}</div><div className="repository-list">{recents.map((path) => <button className="repository-row" data-selected={selected?.path === path || undefined} key={path} onClick={() => inspect(path)} disabled={busy || running}><span className="classification-mark"><FolderGit2 size={15} /></span><span><strong>{path.split("/").slice(-1)[0]}</strong><small>{path.split("/").slice(-2).join("/")}</small></span><ChevronRight size={14} /></button>)}</div><Button variant="secondary" className="choose-button" onClick={choose} loading={busy} disabled={running}><FolderGit2 size={14} />Choose repository</Button></div><div className="rail-footer"><button onClick={selectVela} disabled={busy || running}><Settings2 size={15} /><span><strong>Vela runtime</strong><small>{identity?.version ?? (boot?.preferences.vela_binary_path ? "Signed v0.977.3 selected" : "Choose signed binary")}</small></span></button><p>Local work · explicit Repository authority</p></div></aside>
    <main className="workspace">{error && <Refusal error={error} dismiss={() => setError(null)} />}{handoff && <ProblemContinuation handoff={handoff} source={handoffSource} authority={handoffAuthority} chooseSource={chooseHandoffSource} chooseAuthority={chooseHandoffAuthority} activateSource={() => activateHandoffRepository("source")} activateAuthority={() => activateHandoffRepository("authority")} open={openProblem} dismiss={dismissHandoff} busy={busy} />}{!selected ? <EmptyState choose={choose} busy={busy} /> : <><header className="source-header"><div className="source-title"><div className="source-icon"><FolderGit2 /></div><div><p className="eyebrow">{selectedRoleLabel}</p><h1>{selected.name}</h1><code>{selected.path}</code></div></div><div className="source-meta"><Badge tone={selected.git.dirty ? "warning" : "positive"}>{selected.git.dirty ? <AlertTriangle size={12} /> : <Check size={12} />}{sourceBadge}</Badge><span><GitBranch size={13} />{short(selected.git.head_commit, 12)}</span><Button variant="ghost" size="sm" onClick={() => inspect(selected.path)} loading={busy} disabled={running}><RefreshCw size={13} />Refresh</Button></div></header>
      <WorkbenchTabs orient={<Orient snapshot={selected} />} execute={<Execute snapshot={selected} launch={launch} launching={launching} worktreeRef={worktreeRef} setWorktreeRef={setWorktreeRef} worktreePreview={worktreePreview} previewWorktree={previewWorktree} createWorktree={createWorktree} profile={profile} setProfile={setProfile} tool={tool} selectTool={selectTool} execPreview={execPreview} previewExec={previewExec} runExec={runExec} cancelExec={cancelExec} running={running} result={execResult} busy={busy} evidence={evidence} />} capture={<Capture evidence={evidence} result={execResult} selected={capture} setSelected={chooseCapture} chooseFile={chooseEvidence} exclusions={exclusions} setExclusions={setExclusions} redaction={redaction} setRedaction={setRedaction} derivedText={derivedText} setDerivedText={setDerivedText} previewExport={previewEvidenceExport} exportPreview={exportPreview} exportEvidence={exportEvidence} busy={busy} />} review={<ReviewDraft snapshot={selected} handoff={handoff} handoffSource={handoffSource} handoffAuthority={handoffAuthority} selectedIsAuthority={selectedIsAuthority} evidence={evidence} result={execResult} assertion={assertion} setAssertion={setAssertion} claimType={claimType} setClaimType={setClaimType} producer={producer} setProducer={setProducer} caveat={caveat} setCaveat={setCaveat} requirement={requirement} setRequirement={setRequirement} selectedArtifacts={selectedArtifacts} toggleArtifact={toggleArtifact} includeCheck={includeCheck} setIncludeCheck={setIncludeCheck} preview={submissionPreview} previewDraft={previewDraft} submitDraft={submitDraft} importPreview={importPreview} chooseImport={chooseImport} importSubmission={importSubmission} submissionResult={submissionResult} busy={busy} />} authority={<TrancheThree key={`${selected.path}:${handoff?.handoff_url ?? "local"}`} snapshot={selected} evidence={evidence} onRepositoryChanged={refreshAfterAuthorityMutation} openRepository={() => launch("terminal")} problemUrl={handoff?.problem_url ?? null} openProblem={handoff ? openProblem : undefined} />} /></>}</main>
    {boot && <footer className="runtime-footer"><span>Interface <code>{short(boot.runtime.interface_commit)}</code></span><span>Runtime <code>{boot.runtime.runtime_version}</code></span><span>Scoped Check · attributed Repository Decision</span></footer>}</div>;
}
