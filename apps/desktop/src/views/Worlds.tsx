import { useState } from "react";
import { call, useData } from "../api";
import { useApp } from "../App";

export function Library({ worlds }: { worlds: any[] }) {
  const app = useApp();
  const [name, setName] = useState("Marrowick");
  const [seed, setSeed] = useState(42);
  const [verify, setVerify] = useState<any>(null);

  return (
    <>
      <h1>World Library</h1>
      <div className="note">
        A world is an append-only history. Every campaign is a branch of it, so many campaigns can run at once in the same world without overwriting each other — and their consequences can be merged back by the World Owner.
      </div>
      <div className="card">
        <h2>New world</h2>
        <div className="row">
          <div className="field">
            <label htmlFor="nw-name">Name</label>
            <input id="nw-name" value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="field">
            <label htmlFor="nw-seed">Seed</label>
            <input id="nw-seed" type="number" min={0} value={seed} onChange={(e) => setSeed(Number(e.target.value))} style={{ width: 110 }} />
          </div>
          <button
            className="primary"
            style={{ alignSelf: "flex-end" }}
            onClick={() =>
              app.run(async () => {
                const w = await call("world.create", { name, seed });
                app.select(w.world_id, w.branch_id);
                app.go("observatory");
              })
            }
          >
            Generate world
          </button>
        </div>
        <p className="faint" style={{ marginBottom: 0 }}>
          Generates the river trading town of Marrowick: about 250 residents with three generations of genealogy, households, three rival organisations, a working ledger economy and a seeded world bible. The same seed always produces the same world.
        </p>
      </div>
      <div className="card">
        <h2>Worlds on this machine</h2>
        {!worlds.length && <p className="dim">None yet.</p>}
        <table>
          <tbody>
            {worlds.map((w) => (
              <tr key={w.world_id} className={w.world_id === app.worldId ? "sel" : ""}>
                <td>
                  <b>{w.name}</b>
                  <div className="faint mono">{w.world_id} · seed {w.seed}</div>
                </td>
                <td className="dim">
                  {w.branches} timelines · {w.events.toLocaleString()} events
                  <div className="faint">created {w.created_at.slice(0, 10)}</div>
                </td>
                <td style={{ textAlign: "right" }}>
                  <div className="row" style={{ justifyContent: "flex-end" }}>
                    <button onClick={() => { app.select(w.world_id, w.canon_branch); app.go("observatory"); }}>Open</button>
                    <button onClick={() => app.run(async () => setVerify(await call("world.verify", { branch: w.canon_branch })))}>Verify</button>
                    <button
                      onClick={() =>
                        app.run(async () => {
                          const data = await call("world.export", { branch: w.canon_branch });
                          const url = URL.createObjectURL(new Blob([JSON.stringify(data)], { type: "application/json" }));
                          const a = document.createElement("a");
                          a.href = url;
                          a.download = `${w.name.replace(/\W+/g, "_")}_${w.world_id}.lwe.json`;
                          a.click();
                          URL.revokeObjectURL(url);
                        })
                      }
                    >
                      Export
                    </button>
                    <button
                      className="danger"
                      onClick={() => {
                        if (window.confirm(`Delete "${w.name}" and all ${w.branches} timelines? This cannot be undone. Export first if unsure.`)) app.run(() => call("world.delete", { world_id: w.world_id, confirm: w.world_id }));
                      }}
                    >
                      Delete
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {verify && (
          <div className={verify.ok ? "note" : "err"} style={{ marginTop: 12 }}>
            <b>{verify.ok ? "Verified." : "Verification FAILED."}</b> Replayed {verify.events_checked.toLocaleString()} events from genesis; hash chain, recorded state hashes, snapshots and the live projection {verify.ok ? "all agree" : "do not agree"}.
            <div className="mono">state {verify.state_hash.slice(0, 24)}…</div>
            {verify.problems.map((p: string) => (
              <div key={p}>• {p}</div>
            ))}
          </div>
        )}
      </div>
    </>
  );
}

const CONTENT = ["violence", "horror", "romance"];

export function Campaigns() {
  const app = useApp();
  const [name, setName] = useState("The Lean Year");
  const [rules, setRules] = useState("srd-5.2.1");
  const [gm, setGm] = useState("human");
  const [content, setContent] = useState<Record<string, string>>({ violence: "allowed", horror: "allowed", romance: "limited" });
  const [fork, setFork] = useState("What-if");
  const [pc, setPc] = useState({ name: "Ren Vale", class: "rogue" });
  const cur = app.branches.find((b) => b.branch_id === app.branch);

  return (
    <>
      <h1>Campaigns &amp; Branches</h1>
      <div className="grid g2">
        <div className="card">
          <h2>New campaign</h2>
          <p className="dim">Forks the selected timeline. The campaign has its own clock and party; the parent never changes.</p>
          <div className="row">
            <div className="field"><label>Name</label><input value={name} onChange={(e) => setName(e.target.value)} /></div>
            <div className="field">
              <label>Rules profile (pinned)</label>
              <select value={rules} onChange={(e) => setRules(e.target.value)}>
                <option value="srd-5.2.1">D&amp;D 5e 2024 (SRD 5.2.1)</option>
                <option value="srd-5.1">D&amp;D 5e 2014 (SRD 5.1)</option>
              </select>
            </div>
            <div className="field">
              <label>Game Master</label>
              <select value={gm} onChange={(e) => setGm(e.target.value)}>
                <option value="human">Human GM</option>
                <option value="ai">AI GM (narration &amp; NPCs)</option>
                <option value="collaborative">Collaborative</option>
              </select>
            </div>
          </div>
          <div className="row" style={{ marginTop: 8 }}>
            {CONTENT.map((c) => (
              <div className="field" key={c}>
                <label>{c}</label>
                <select value={content[c]} onChange={(e) => setContent({ ...content, [c]: e.target.value })}>
                  {["excluded", "veil", "limited", "allowed"].map((l) => <option key={l}>{l}</option>)}
                </select>
              </div>
            ))}
            <button
              className="primary"
              style={{ alignSelf: "flex-end" }}
              onClick={() =>
                app.run(async () => {
                  const r = await call("campaign.create", { world_id: app.worldId, from_branch: app.branch, name, rules, gm_mode: gm, content });
                  app.select(app.worldId, r.branch_id);
                })
              }
            >
              Start campaign
            </button>
          </div>
          <p className="faint" style={{ marginBottom: 0 }}>Content settings filter how events are presented at this table. They never change what happened in the world.</p>
        </div>
        <div className="card">
          <h2>Party {cur?.campaign ? `— ${cur.name}` : ""}</h2>
          {!cur?.campaign ? (
            <p className="dim">Select a campaign timeline to add player characters.</p>
          ) : (
            <>
              <p>{cur.party.length ? cur.party.join(", ") : <span className="dim">No characters yet.</span>}</p>
              <div className="row">
                <div className="field"><label>Character name</label><input value={pc.name} onChange={(e) => setPc({ ...pc, name: e.target.value })} /></div>
                <div className="field">
                  <label>Class</label>
                  <select value={pc.class} onChange={(e) => setPc({ ...pc, class: e.target.value })}>
                    {["fighter", "rogue", "wizard", "cleric"].map((c) => <option key={c}>{c}</option>)}
                  </select>
                </div>
                <button style={{ alignSelf: "flex-end" }} onClick={() => app.run(() => call("pc.create", { branch: app.branch, pc }))}>Add character</button>
                <button className="primary" style={{ alignSelf: "flex-end" }} disabled={!cur.party.length} onClick={() => app.go("play")}>Play →</button>
              </div>
              <p className="faint" style={{ marginBottom: 0 }}>Level 1, standard array assigned by class. Starting purse is an explicit, logged issuance.</p>
            </>
          )}
          <h2 style={{ marginTop: 18 }}>Experiment branch</h2>
          <div className="row">
            <input value={fork} onChange={(e) => setFork(e.target.value)} aria-label="Branch name" />
            <button onClick={() => app.run(async () => { const b = await call("branch.fork", { parent: app.branch, name: fork }); app.select(app.worldId, b.branch_id); })}>Fork selected timeline</button>
          </div>
        </div>
      </div>
      <div className="card">
        <h2>All timelines</h2>
        <table>
          <thead><tr><th>Timeline</th><th>Kind</th><th>Date</th><th>Party</th><th>Forked from</th><th /></tr></thead>
          <tbody>
            {app.branches.map((b) => (
              <tr key={b.branch_id} className={"click " + (b.branch_id === app.branch ? "sel" : "")} onClick={() => app.select(app.worldId, b.branch_id)}>
                <td><b>{b.name}</b> {b.is_canon && <span className="tag acc">canon</span>}<div className="faint mono">{b.branch_id}</div></td>
                <td>{b.kind}{b.campaign && <div className="faint">{b.campaign.rules} · {b.campaign.gm_mode} GM</div>}</td>
                <td>{b.date}<div className="faint">day {b.day} · #{b.head_seq}</div></td>
                <td>{b.party.join(", ") || <span className="faint">—</span>}</td>
                <td className="faint">{b.parent_id ? `${app.branches.find((x) => x.branch_id === b.parent_id)?.name} @#${b.fork_seq}` : "genesis"}</td>
                <td style={{ textAlign: "right" }}>
                  {!b.is_canon && app.role === "world_owner" && (
                    <button className="small" onClick={(e) => { e.stopPropagation(); if (window.confirm(`Make "${b.name}" the canonical timeline? Other timelines are kept.`)) app.run(() => call("canon.set", { world_id: app.worldId, branch: b.branch_id })); }}>Make canon</button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}

export function MergeLab() {
  const app = useApp();
  const [source, setSource] = useState("");
  const [target, setTarget] = useState("");
  const list = useData<any[]>("merge.list", { world_id: app.worldId }, [app.worldId, app.rev]);
  const name = (id: string) => app.branches.find((b) => b.branch_id === id)?.name ?? id;
  const canon = app.branches.find((b) => b.is_canon)?.branch_id ?? "";

  return (
    <>
      <h1>Historical Merge Lab</h1>
      <div className="note">
        A merge replays one timeline's world-changing interventions (sales, destruction, recorded facts, deaths caused in play…) onto another, in fictional-time order, through the same rules as everything else. If any step is impossible in the target's history, the merge is blocked and the conflicting events are shown. There is no force-merge: resolve conflicts in a new branch.
      </div>
      <div className="card">
        <h2>Propose</h2>
        <div className="row">
          <div className="field"><label>Bring changes from</label>
            <select value={source} onChange={(e) => setSource(e.target.value)}>
              <option value="">— choose —</option>
              {app.branches.map((b) => <option key={b.branch_id} value={b.branch_id}>{b.name}</option>)}
            </select>
          </div>
          <div className="field"><label>Into</label>
            <select value={target || canon} onChange={(e) => setTarget(e.target.value)}>
              {app.branches.map((b) => <option key={b.branch_id} value={b.branch_id}>{b.name}{b.is_canon ? " (canon)" : ""}</option>)}
            </select>
          </div>
          <button className="primary" style={{ alignSelf: "flex-end" }} disabled={!source} onClick={() => app.run(() => call("merge.propose", { source, target: target || canon }))}>Analyse</button>
        </div>
      </div>
      {(list.data ?? []).map((p) => (
        <div className="card" key={p.proposal_id}>
          <div className="row">
            <h2 style={{ margin: 0 }}>{name(p.source)} → {name(p.target)}</h2>
            <span className={"tag " + (p.status === "ready" ? "good" : p.status === "blocked" ? "bad" : p.status === "merged" ? "acc" : "")}>{p.status}</span>
            <span className="faint mono">{p.proposal_id} · common ancestor {name(p.common_ancestor?.branch)} #{p.common_ancestor?.seq}</span>
          </div>
          {p.conflicts?.length > 0 && (
            <>
              <h3 style={{ marginTop: 12 }}>Conflicts ({p.conflicts.length})</h3>
              {p.conflicts.map((c: any) => (
                <div className="err" key={c.source_event}>
                  <b>{c.date}</b> — “{c.summary}” cannot happen in the target timeline.
                  <div>Rule violated: <span className="mono">{c.violated_rule}</span> — {c.explanation}</div>
                  {c.target_events.map((t: any) => (
                    <div key={t.event_id} className="dim">↳ because of <b>{t.date}</b>: {t.summary} <span className="mono faint">{t.event_id}</span></div>
                  ))}
                </div>
              ))}
            </>
          )}
          {p.imports?.length > 0 && (
            <>
              <h3 style={{ marginTop: 12 }}>Would carry across ({p.imports.length})</h3>
              <ul className="tl">
                {p.imports.map((i: any) => <li key={i.source_event}><span className="d">day {i.day} · {i.event_type}</span><span>{i.summary}</span></li>)}
              </ul>
            </>
          )}
          {p.status === "empty" && <p className="dim">Nothing to carry across: the source made no world-changing interventions since the common ancestor.</p>}
          {p.target_days_advanced > 0 && <p className="faint">The merged timeline would first advance {p.target_days_advanced} days so imported events are not placed in the target's past.</p>}
          <div className="row" style={{ marginTop: 10 }}>
            {p.status === "ready" && app.role === "world_owner" && <button className="primary" onClick={() => app.run(async () => { const r = await call("merge.decide", { proposal_id: p.proposal_id, approve: true }); app.select(app.worldId, r.result_branch); })}>Approve — create merged timeline</button>}
            {(p.status === "ready" || p.status === "blocked") && <button onClick={() => app.run(() => call("merge.decide", { proposal_id: p.proposal_id, approve: false }))}>Reject</button>}
            {p.result_branch && <button onClick={() => app.select(app.worldId, p.result_branch)}>Open {name(p.result_branch)}</button>}
            {p.status === "ready" && app.role !== "world_owner" && <span className="faint">Approval requires the World Owner.</span>}
          </div>
        </div>
      ))}
    </>
  );
}
