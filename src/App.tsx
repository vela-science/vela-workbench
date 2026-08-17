import { useEffect, useMemo, useState } from "react";
import {
  AlertTriangle, ArrowUpRight, Check, ChevronRight, CircleDot, Code2,
  FolderGit2, GitBranch, HardDrive, RefreshCw, Settings2, TerminalSquare, Trash2,
} from "lucide-react";
import type {
  BootstrapDto, CommandErrorDto, LaunchKindDto, RepositorySnapshotDto, VelaBinaryDto,
} from "./contracts/generated/ipc";
import { Badge } from "./components/ui/badge";
import { Button } from "./components/ui/button";
import { WorkbenchTabs } from "./components/ui/tabs";
import { workbench } from "./lib/workbench";
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

function Fact({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return <div className="fact"><dt>{label}</dt><dd className={mono ? "mono" : undefined} title={value}>{value}</dd></div>;
}

function EmptyState({ choose, busy }: { choose: () => void; busy: boolean }) {
  return <section className="empty-state" aria-labelledby="empty-title">
    <div className="empty-mark"><FolderGit2 aria-hidden="true" /></div>
    <p className="eyebrow">Local repository required</p>
    <h1 id="empty-title">Start from sovereign source</h1>
    <p>Choose an existing Git repository. Workbench inspects exact local source and supported Vela JSON without copying either.</p>
    <Button size="lg" onClick={choose} loading={busy}>Choose repository</Button>
    <p className="boundary-note"><HardDrive size={14} /> Private files and credentials stay local.</p>
  </section>;
}

function Refusal({ error, dismiss }: { error: CommandErrorDto; dismiss: () => void }) {
  return <section className="refusal" role="alert"><AlertTriangle aria-hidden="true" /><div>
    <strong>Inspection refused · {error.kind}</strong><p>{error.message}</p>{error.detail && <code>{error.detail}</code>}
  </div><Button variant="ghost" size="sm" onClick={dismiss}>Dismiss</Button></section>;
}

function Orient({ snapshot }: { snapshot: RepositorySnapshotDto }) {
  const { status, claims, integration, refusal, binary } = snapshot.vela;
  return <div className="surface-stack">
    <section className="section-block">
      <div className="section-heading"><div><p className="eyebrow">Observation</p><h2>Scientific state</h2></div>
        <Badge tone={status ? "positive" : integration ? "neutral" : "warning"}>{status ? "Vela repository" : integration ? "Native integration" : "Git source"}</Badge>
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
        <Fact label="Accepted claims" value={String(status.counts.accepted_claims)} /><Fact label="Pending review" value={String(status.counts.pending_review)} />
      </dl>
      <div className="command-strip"><span>Current work</span><code>{status.work_command || status.work_mode}</code><small>{status.work_note}</small></div>
    </section>}

    {claims.length > 0 && <section className="section-block">
      <div className="section-heading"><div><p className="eyebrow">Accepted and current</p><h2>Contributions</h2></div><span className="count-label">{claims.length} rows</span></div>
      <div className="table-wrap"><table><thead><tr><th>Claim</th><th>Standing</th><th>Assertion</th><th>Root</th></tr></thead><tbody>
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

    <section className="section-block handoff-block"><div><p className="eyebrow">Problems boundary</p><h2>No reviewed Problem locator</h2>
      <p>Public discovery and shared contribution coordination remain in problems.science. Tranche 1 does not infer or search for a Problem.</p></div>
      <Button variant="secondary" disabled>Open exact Problem <ArrowUpRight size={14} /></Button></section>
  </div>;
}

function ExecuteSource({ snapshot, launch, launching }: { snapshot: RepositorySnapshotDto; launch: (kind: LaunchKindDto) => void; launching: LaunchKindDto | null }) {
  const launchers: Array<[LaunchKindDto, string, typeof Code2]> = [
    ["terminal", "Terminal", TerminalSquare], ["cursor", "Cursor", Code2],
    ["visual_studio_code", "VS Code", Code2], ["forge", "Forge", ArrowUpRight],
  ];
  return <div className="surface-stack">
    <section className="section-block handoff-block source-handoff"><div><p className="eyebrow">Explicit handoff</p><h2>Open exact source elsewhere</h2>
      <p>Workbench passes the selected Git root or a derived HTTPS forge locator. The receiving tool owns every subsequent action.</p></div>
      <div className="button-cluster">{launchers.map(([kind, label, Icon]) => <Button key={kind} variant={kind === "terminal" ? "primary" : "secondary"} onClick={() => launch(kind)} loading={launching === kind}><Icon size={14} />{label}</Button>)}</div>
    </section>
    <section className="section-block"><div className="section-heading"><div><p className="eyebrow">Git-owned</p><h2>Worktrees</h2></div><span className="count-label">{snapshot.git.worktrees.length}</span></div>
      <div className="row-list">{snapshot.git.worktrees.map((tree) => <div className="data-row" key={tree.path}><GitBranch /><div><strong>{tree.branch ?? "Detached HEAD"}</strong><code>{tree.path}</code></div><span><code>{short(tree.head)}</code>{tree.locked && <Badge tone="warning">locked</Badge>}</span></div>)}</div>
    </section>
    <section className="section-block two-column"><div><div className="section-heading"><div><p className="eyebrow">Entire-owned references</p><h2>Checkpoints</h2></div>
      <Badge tone={snapshot.entire.cli_available ? "positive" : "neutral"}>{snapshot.entire.cli_available ? "CLI available" : "CLI unavailable"}</Badge></div>
      {snapshot.git.entire_checkpoints.length ? <div className="row-list">{snapshot.git.entire_checkpoints.map((item) => <div className="data-row" key={`${item.commit}-${item.checkpoint_id}`}><CircleDot /><div><strong>{item.checkpoint_id}</strong><code>{short(item.commit)}</code></div></div>)}</div>
        : <p className="empty-copy">No Entire-Checkpoint trailers found in the last 100 commits. No substitute session store was created.</p>}</div>
      <div><div className="section-heading"><div><p className="eyebrow">Git-owned</p><h2>Remotes</h2></div><span className="count-label">{snapshot.git.remotes.length}</span></div>
        <div className="row-list">{snapshot.git.remotes.map((remote, index) => <div className="data-row" key={`${remote.name}-${remote.operation}-${index}`}><HardDrive /><div><strong>{remote.name} · {remote.operation}</strong><code>{remote.url}</code></div></div>)}</div></div>
    </section>
  </div>;
}

export default function App() {
  const [boot, setBoot] = useState<BootstrapDto | null>(null);
  const [selected, setSelected] = useState<RepositorySnapshotDto | null>(null);
  const [identity, setIdentity] = useState<VelaBinaryDto | null>(null);
  const [error, setError] = useState<CommandErrorDto | null>(null);
  const [busy, setBusy] = useState(false);
  const [launching, setLaunching] = useState<LaunchKindDto | null>(null);
  useEffect(() => { workbench.bootstrap().then(setBoot).catch((value) => setError(asError(value))); }, []);
  const recents = boot?.preferences.recent_repositories ?? [];
  const sourceBadge = useMemo(() => selected ? `${selected.git.branch ?? "detached"} · ${selected.git.dirty ? `${selected.git.changed_paths} changed` : "clean"}` : "", [selected]);

  async function choose() { setBusy(true); setError(null); try { const result = await workbench.selectRepository(); if (result) { setSelected(result); setBoot(await workbench.bootstrap()); } } catch (value) { setError(asError(value)); } finally { setBusy(false); } }
  async function inspect(path: string) { setBusy(true); setError(null); try { setSelected(await workbench.inspectRepository(path)); } catch (value) { setError(asError(value)); } finally { setBusy(false); } }
  async function selectVela() { setBusy(true); setError(null); try { const result = await workbench.selectVelaBinary(); if (result) { setIdentity(result); setBoot(await workbench.bootstrap()); if (selected) setSelected(await workbench.inspectRepository(selected.path)); } } catch (value) { setError(asError(value)); } finally { setBusy(false); } }
  async function clear() { try { const preferences = await workbench.clearRecents(); setBoot((current) => current ? { ...current, preferences } : current); setIdentity(null); setSelected(null); } catch (value) { setError(asError(value)); } }
  async function launch(kind: LaunchKindDto) { if (!selected) return; setLaunching(kind); setError(null); try { await workbench.launchRepository(selected.path, kind); } catch (value) { setError(asError(value)); } finally { setLaunching(null); } }

  return <div className="app-shell"><aside className="repo-rail">
    <div className="brand"><div className="brand-mark">V</div><div><strong>Vela</strong><span>Workbench</span></div></div>
    <div className="rail-section"><div className="rail-label"><span>Repositories</span>{(recents.length > 0 || boot?.preferences.vela_binary_path) && <button onClick={clear} aria-label="Clear local repository and Vela tool choices" title="Clear all local choices"><Trash2 size={13} /></button>}</div>
      <div className="repository-list">{recents.map((path) => <button className="repository-row" data-selected={selected?.path === path || undefined} key={path} onClick={() => inspect(path)} disabled={busy}><span className="classification-mark"><FolderGit2 size={15} /></span><span><strong>{path.split("/").slice(-1)[0]}</strong><small>{path.split("/").slice(-2).join("/")}</small></span><ChevronRight size={14} /></button>)}</div>
      <Button variant="secondary" className="choose-button" onClick={choose} loading={busy}><FolderGit2 size={14} />Choose repository</Button>
    </div>
    <div className="rail-footer"><button onClick={selectVela} disabled={busy}><Settings2 size={15} /><span><strong>Vela runtime</strong><small>{identity?.version ?? (boot?.preferences.vela_binary_path ? "Selected locally" : "Choose signed binary")}</small></span></button><p>Read-only Tranche 1</p></div>
  </aside><main className="workspace">{error && <Refusal error={error} dismiss={() => setError(null)} />}
    {!selected ? <EmptyState choose={choose} busy={busy} /> : <><header className="source-header"><div className="source-title"><div className="source-icon"><FolderGit2 /></div><div><p className="eyebrow">Selected local source</p><h1>{selected.name}</h1><code>{selected.path}</code></div></div>
      <div className="source-meta"><Badge tone={selected.git.dirty ? "warning" : "positive"}>{selected.git.dirty ? <AlertTriangle size={12} /> : <Check size={12} />}{sourceBadge}</Badge><span><GitBranch size={13} />{short(selected.git.head_commit, 12)}</span><Button variant="ghost" size="sm" onClick={() => inspect(selected.path)} loading={busy}><RefreshCw size={13} />Refresh</Button></div></header>
      <WorkbenchTabs orient={<Orient snapshot={selected} />} source={<ExecuteSource snapshot={selected} launch={launch} launching={launching} />} /></>}
  </main>{boot && <footer className="runtime-footer"><span>Interface <code>{short(boot.runtime.interface_commit)}</code></span><span>Runtime <code>{boot.runtime.runtime_version}</code></span><span>Local observation only</span></footer>}</div>;
}
