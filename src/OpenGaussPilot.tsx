import { useState } from "react";
import { AlertTriangle, ArrowUpRight, Check, FileCheck2, RefreshCw, TerminalSquare } from "lucide-react";
import type {
  CommandErrorDto,
  EvidenceItemDto,
  NativeExecResultDto,
  OpenGaussHandoffPreviewDto,
  OpenGaussHandoffReceiptDto,
  RepositorySnapshotDto,
} from "./contracts/generated/ipc";
import { Badge } from "./components/ui/badge";
import { Button } from "./components/ui/button";
import { workbench } from "./lib/workbench";

function short(value: string, length = 18) {
  return value.slice(0, length);
}

function asError(value: unknown): CommandErrorDto {
  if (typeof value === "object" && value !== null && "message" in value) {
    const error = value as Partial<CommandErrorDto>;
    return { kind: error.kind ?? "unknown", message: String(error.message), detail: error.detail ?? null };
  }
  return { kind: "unknown", message: String(value), detail: null };
}

function PilotFact({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return <div className="fact"><dt>{label}</dt><dd className={mono ? "mono" : undefined} title={value}>{value}</dd></div>;
}

type Props = {
  snapshot: RepositorySnapshotDto;
  evidence: EvidenceItemDto[];
  result: NativeExecResultDto | null;
};

export function OpenGaussPilot({ snapshot, evidence, result }: Props) {
  const [preview, setPreview] = useState<OpenGaussHandoffPreviewDto | null>(null);
  const [receipt, setReceipt] = useState<OpenGaussHandoffReceiptDto | null>(null);
  const [selectedEvidence, setSelectedEvidence] = useState<string[]>([]);
  const [includeCheck, setIncludeCheck] = useState(false);
  const [busy, setBusy] = useState<"inspect" | "launch" | "refresh" | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<CommandErrorDto | null>(null);

  async function inspect() {
    setBusy("inspect"); setError(null); setStatus(null); setPreview(null); setReceipt(null);
    try {
      const next = await workbench.selectOpenGauss(snapshot.path);
      if (next) setPreview(next); else setStatus("OpenGauss inspection cancelled. No executable was run.");
    } catch (value) { setError(asError(value)); }
    finally { setBusy(null); }
  }

  async function launch() {
    if (!preview) return;
    setBusy("launch"); setError(null); setStatus(null);
    try {
      const next = await workbench.launchOpenGaussHandoff(preview);
      if (next) { setReceipt(next); setStatus("Terminal opened at the exact project root. Workbench did not start OpenGauss or type a workflow command."); }
      else setStatus("Interactive handoff cancelled. No Terminal was opened.");
    } catch (value) { setError(asError(value)); }
    finally { setBusy(null); }
  }

  async function refresh() {
    if (!receipt) return;
    setBusy("refresh"); setError(null); setStatus(null);
    try {
      const sources = evidence.filter((item) => selectedEvidence.includes(item.sha256)).map((item) => item.source);
      const next = await workbench.refreshOpenGaussHandoff(receipt, sources, includeCheck && result ? [result.run_id] : []);
      setReceipt(next);
      setStatus("Receipt refreshed from exact current Git identity and explicit Workbench selections.");
    } catch (value) { setError(asError(value)); }
    finally { setBusy(null); }
  }

  function toggleEvidence(sha256: string) {
    setSelectedEvidence((current) => current.includes(sha256) ? current.filter((item) => item !== sha256) : [...current, sha256]);
  }

  return <section className="section-block opengauss-pilot" aria-labelledby="opengauss-title">
    <div className="section-heading"><div><p className="eyebrow">WB-OPENGAUSS-01 · external local orchestrator</p><h2 id="opengauss-title">OpenGauss handoff pilot</h2></div><Badge>authority effect: none</Badge></div>
    <p className="boundary-copy">Workbench verifies one selected executable and one repository-owned <code>.gauss/project.yaml</code>, then can open Terminal at the exact project root. OpenGauss, Codex, or Claude owns the work; this pilot never starts a workflow, ingests sessions, or treats progress as Verification or Standing.</p>
    {!preview && <div className="pilot-empty"><div><strong>No OpenGauss tool inspected</strong><p>Select an exact executable named <code>gauss</code>. A native confirmation shows its path, digest, fixed <code>--version</code> argv, cwd, bounded probe environment, and current-user trust warning before anything executes.</p></div><Button variant="secondary" onClick={inspect} loading={busy === "inspect"}><TerminalSquare size={14} />Select OpenGauss</Button></div>}
    {preview && <div className="pilot-stack">
      <div className="trust-warning"><AlertTriangle /><p>{preview.tool.trust_warning}</p></div>
      <dl className="fact-grid compact">
        <PilotFact label="Executable / version" value={`${preview.tool.path} · ${preview.tool.version}`} mono />
        <PilotFact label="Executable digest / size" value={`${preview.tool.sha256} · ${preview.tool.size} bytes`} mono />
        <PilotFact label="Project config" value={preview.project.manifest_path} mono />
        <PilotFact label="Config digest / size" value={`${preview.project.manifest_sha256} · ${preview.project.manifest_size} bytes`} mono />
        <PilotFact label="Project / schema" value={`${preview.project.name} · ${preview.project.kind} · schema ${preview.project.schema_version}`} />
        <PilotFact label="Lean root / cwd" value={`${preview.project.lean_root} · ${preview.cwd}`} mono />
        <PilotFact label="Git before" value={`${preview.git_before.commit} · ${preview.git_before.tree} · ${preview.git_before.dirty ? "dirty" : "clean"}`} mono />
        <PilotFact label="Interactive argv boundary" value={preview.interactive_argv.map((value) => JSON.stringify(value)).join(" ")} mono />
      </dl>
      <details><summary>Version-probe environment ({preview.tool.probe_environment.length})</summary><pre>{preview.tool.probe_environment.map((entry) => `${entry.name}=${entry.value}`).join("\n")}</pre></details>
      <details><summary>Bounded Terminal launcher environment ({preview.launcher_environment.length})</summary><pre>{preview.launcher_environment.map((entry) => `${entry.name}=${entry.value}`).join("\n")}</pre><p className="boundary-copy">This applies to Workbench's <code>/usr/bin/open</code> launcher only. Terminal owns the later interactive shell environment.</p></details>
      <div className="workflow-boundary"><div><strong>Documented interactive workflows</strong><div className="workflow-list">{preview.documented_workflows.map((workflow) => <code key={workflow}>{workflow}</code>)}</div></div><p>{preview.documented_entrypoint}. These are slash commands, not stable shell workflow argv. Workbench does not type or automate them.</p></div>
      <div className="pilot-limit"><strong>Observed boundary</strong><p>{preview.backend_identity}. Hidden model transport visible: {String(preview.hidden_transport_visible)}. Terminal owns the interactive shell environment; Workbench does not observe or constrain it.</p></div>
      {!receipt && <div className="review-actions"><Button onClick={launch} loading={busy === "launch"}><ArrowUpRight size={14} />Open explicit Terminal handoff</Button><Button variant="ghost" onClick={inspect} disabled={Boolean(busy)}>Re-select</Button></div>}
    </div>}
    {status && <div className="running-state" role="status" aria-live="polite"><Check />{status}</div>}
    {error && <div className="inline-refusal" role="alert"><AlertTriangle /><div><strong>OpenGauss pilot refused · {error.kind}</strong><p>{error.message}</p>{error.detail && <code>{error.detail}</code>}</div></div>}
    {receipt && <div className="pilot-receipt">
      <div className="section-heading"><div><p className="eyebrow">External handoff receipt</p><h3>Select only reusable result evidence</h3></div><Badge tone="positive">{receipt.terminal_owner} opened</Badge></div>
      <p>{receipt.result_boundary}</p>
      <fieldset className="artifact-fieldset"><legend>Explicit files already captured by Workbench</legend>
        {evidence.length ? evidence.map((item) => <label key={item.sha256}><input type="checkbox" checked={selectedEvidence.includes(item.sha256)} onChange={() => toggleEvidence(item.sha256)} /><span><strong>{item.display_name}</strong><small>{item.sha256} · {item.size} bytes · source {short(item.source_commit)}</small></span></label>) : <p>No file evidence is selected. Use Capture after the external work completes.</p>}
      </fieldset>
      {result ? <label className="check-field"><input type="checkbox" checked={includeCheck} onChange={(event) => setIncludeCheck(event.target.checked)} /><span>Bind exact reviewed check <code>{result.run_id}</code> · {result.producer_check_method}:{result.producer_check_outcome}</span></label> : <p className="empty-copy">No reviewed NativeExec check is in bounded memory.</p>}
      <Button variant="secondary" onClick={refresh} loading={busy === "refresh"}><RefreshCw size={14} />Refresh Git and bind selected evidence</Button>
      {receipt.git_after && <dl className="fact-grid compact receipt-grid"><PilotFact label="Git before" value={`${receipt.preview.git_before.commit} · ${receipt.preview.git_before.tree}`} mono /><PilotFact label="Git after" value={`${receipt.git_after.commit} · ${receipt.git_after.tree}`} mono /><PilotFact label="Selected evidence" value={String(receipt.selected_evidence.length)} /><PilotFact label="Selected checks" value={String(receipt.selected_checks.length)} /></dl>}
      {(receipt.selected_evidence.length > 0 || receipt.selected_checks.length > 0) && <div className="selected-result-list"><strong>Bound exact result facets</strong>{receipt.selected_evidence.map((item) => <div key={item.sha256}><FileCheck2 /><span>{item.display_name}<small>{item.sha256} · {item.size} bytes · {item.source}</small></span></div>)}{receipt.selected_checks.map((item) => <div key={item.run_id}><Check /><span>{item.run_id}<small>{item.profile} · {item.producer_check_method}:{item.producer_check_outcome} · stdout {item.stdout_sha256}</small></span></div>)}</div>}
      <p className="boundary-copy"><strong>Next boundary:</strong> Capture exact selected bytes and prepare an ordinary Submission v3. OpenGauss provenance remains external-tool provenance with <code>authority_effect:none</code>. A later scoped Verification and attributed Repository Decision stay separate.</p>
    </div>}
  </section>;
}
