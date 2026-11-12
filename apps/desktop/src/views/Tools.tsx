import { useEffect, useState } from "react";
import { call, useData } from "../api";
import { useApp } from "../App";

export function Council() {
  const app = useApp();
  const [q, setQ] = useState("How should the Compact's quiet factor move against the Reeve this season without taking choices away from the players?");
  const [memo, setMemo] = useState<any>(null);
  return (
    <>
      <h1>Writers' Brains council</h1>
      <div className="note">
        A decision procedure, not an oracle: route the question to craft lenses → competing proposals → adversarial critique → what the simulation can test → a recommendation for you to accept or reject. The council never writes canon, and never puts words in a named author's mouth: the registry lists candidate voices per lens with <b>zero verified sources</b> so far.
      </div>
      <div className="card">
        <textarea rows={3} value={q} onChange={(e) => setQ(e.target.value)} aria-label="Question for the council" />
        <div className="row" style={{ marginTop: 8 }}>
          <button className="primary" onClick={() => app.run(async () => setMemo(await call("council.ask", { question: q, branch: app.branch })))}>Convene</button>
          <span className="faint">Uses the selected timeline's current situation as the brief.</span>
        </div>
      </div>
      {memo && (
        <>
          {memo.ai_error && <div className="err">AI unavailable ({memo.ai_error.code}: {memo.ai_error.message}). Showing the offline framework.</div>}
          {memo.memo_markdown && <div className="card"><h2>Decision memo <span className="tag acc">{memo.model}</span></h2><div className="md">{memo.memo_markdown}</div></div>}
          <div className="grid g2">
            <div className="card">
              <h2>Lenses and candidate voices</h2>
              {memo.lenses.map((l: string) => (
                <div key={l} style={{ marginBottom: 10 }}>
                  <b>{l.replace(/_/g, " ")}</b> <span className="dim">— {memo.voices[l]?.about}</span>
                  <div className="faint">{(memo.voices[l]?.candidates ?? []).map((c: any) => c.name + (c.kind === "fictional_persona" ? " (fictional)" : "")).join(" · ")}</div>
                </div>
              ))}
            </div>
            <div className="card">
              <h2>{memo.memo_markdown ? "Baseline proposals (offline framework)" : "Competing proposals"}</h2>
              {memo.proposals.map((p: any) => <p key={p.label}><b>{p.label}.</b> {p.sketch}<br /><span className="faint mono">{p.implements_with.join(" · ")}</span></p>)}
              <h3>Critic's checklist</h3>
              {memo.critic_checklist.map((c: string) => <div key={c} className="dim">☐ {c}</div>)}
            </div>
          </div>
        </>
      )}
    </>
  );
}

export function AiAgents({ onChange }: { onChange: () => void }) {
  const app = useApp();
  const cfg = useData<any>("ai.config.get", {}, [app.rev]);
  const usage = useData<any>("ai.usage", {}, [app.rev]);
  const agents = useData<any[]>("agents.list", {}, [app.rev]);
  const [key, setKey] = useState("");
  const [sel, setSel] = useState("t1-resident");
  const [body, setBody] = useState("");
  const [newName, setNewName] = useState("custom-");
  const cur = (agents.data ?? []).find((a) => a.name === sel);
  useEffect(() => { if (cur) setBody(cur.body); }, [cur?.name, cur?.body]); // eslint-disable-line react-hooks/exhaustive-deps
  const c = cfg.data;
  if (!c) return <p className="dim">{cfg.error ?? "Loading…"}</p>;
  const owner = app.role === "world_owner";
  const set = (args: Record<string, unknown>) => app.run(async () => { await call("ai.config.set", args); onChange(); });
  const modelSel = (k: string, label: string, hint: string) => (
    <div className="field"><label>{label}</label>
      <select value={c[k]} disabled={!owner} onChange={(e) => set({ [k]: e.target.value })}>{c.known_models.map((m: string) => <option key={m}>{m}</option>)}</select>
      <span className="faint">{hint}</span>
    </div>
  );
  return (
    <>
      <h1>AI &amp; Agents</h1>
      <div className="grid g2">
        <div className="card">
          <h2>Provider</h2>
          <div className="row">
            <button className={c.provider === "offline" ? "primary" : ""} disabled={!owner} onClick={() => set({ provider: "offline" })}>Offline</button>
            <button className={c.provider === "anthropic" ? "primary" : ""} disabled={!owner} onClick={() => set({ provider: "anthropic" })}>Claude (Anthropic API)</button>
            <span className={"tag " + (c.has_key ? "good" : "warn")}>{c.has_key ? "API key loaded" : "no API key"}</span>
          </div>
          <p className="dim">Offline, the whole world still runs: the simulation never calls a model, and NPCs answer from a deterministic voice built on the same knowledge. Online, models voice NPCs, narrate and advise — and can only ever <i>propose</i>; every outcome passes the same rules.</p>
          <div className="row">
            <input type="password" placeholder="sk-ant-…" value={key} onChange={(e) => setKey(e.target.value)} aria-label="Anthropic API key" style={{ flex: 1 }} autoComplete="off" />
            <button disabled={!owner || !key} onClick={() => { set({ api_key: key }); setKey(""); }}>Use key</button>
            <button disabled={!owner || !c.has_key} onClick={() => set({ api_key: "" })}>Forget</button>
          </div>
          <p className="faint">The key is held in memory (the desktop app keeps it in the Windows Credential Manager). It is never written to a world file, event, log or export. You can also set the ANTHROPIC_API_KEY environment variable.</p>
          <div className="row">
            <div className="field"><label>Monthly budget (USD)</label><input type="number" min={0} step={1} defaultValue={c.monthly_budget_usd} disabled={!owner} onBlur={(e) => Number(e.target.value) !== c.monthly_budget_usd && set({ monthly_budget_usd: Number(e.target.value) })} style={{ width: 100 }} /></div>
            <div className="tile" style={{ flex: 1 }}><div className="k">Spent this month</div><div className="v">${c.spent_this_month_usd.toFixed(4)}</div><div className="s">hard stop at ${c.monthly_budget_usd.toFixed(2)}; requests fail visibly, never silently</div></div>
          </div>
        </div>
        <div className="card">
          <h2>Model routing by importance tier</h2>
          <div className="grid g2">
            {modelSel("model_light", "T1–T2 residents, narration", "frequent, cheap")}
            {modelSel("model_capable", "T3 major characters", "leaders, investigators, nemeses")}
            {modelSel("model_strategic", "T4 world-shaping actors", "rare, highest capability")}
            {modelSel("model_council", "Writers' Brains council", "design memos")}
          </div>
          <h3 style={{ marginTop: 14 }}>Recent requests</h3>
          <div className="scroll" style={{ maxHeight: 180 }}>
            <table><tbody>
              {(usage.data?.recent ?? []).map((u: any, i: number) => <tr key={i}><td className="faint">{u.at.slice(5, 16).replace("T", " ")}</td><td>{u.kind}</td><td className="mono">{u.model}</td><td className="dim">{u.input_tokens}→{u.output_tokens} tok</td><td className={u.status === "ok" ? "good" : "bad"}>{u.status}</td></tr>)}
              {!usage.data?.recent?.length && <tr><td className="dim">No model requests yet.</td></tr>}
            </tbody></table>
          </div>
        </div>
      </div>
      <div className="card">
        <h2>NPC agent profiles (.md)</h2>
        <p className="dim">Generic profiles cover whole tiers; custom profiles are for story-critical characters. A profile is persona and voice only — what an NPC knows and may do is decided by the engine from world state, and cannot be widened by editing this text. Assign a profile to a person from People → inspect (GM Tools → Edit NPC).</p>
        <div className="grid g23">
          <div>
            <table><tbody>
              {(agents.data ?? []).map((a) => (
                <tr key={a.name} className={"click " + (a.name === sel ? "sel" : "")} onClick={() => setSel(a.name)}>
                  <td><span className="mono">{a.name}.md</span><div className="faint">{a.role}</div></td>
                  <td><span className="tag">T{a.tier}</span>{a.builtin ? <span className="tag">{a.modified ? "built-in, edited" : "built-in"}</span> : <span className="tag acc">custom</span>}</td>
                </tr>
              ))}
            </tbody></table>
            <div className="row" style={{ marginTop: 10 }}>
              <input value={newName} onChange={(e) => setNewName(e.target.value)} aria-label="New profile name" />
              <button disabled={!owner} onClick={() => app.run(async () => { await call("agents.set", { name: newName, body: `---\nname: ${newName}\ntier: 3\nrole: Custom character\nknowledge_scope: [self, household, own_memories, known_claims, relationships]\npermitted_tools: [say, disclose_known_claim, adjust_attitude, state_intent]\nforbidden_writes: [world_state, other_minds, canon_facts]\n---\nDescribe this character's voice, method and limits here. Do not copy facts that can change (name, age, events): those are injected at runtime.\n` }); setSel(newName); })}>New custom profile</button>
            </div>
          </div>
          <div>
            <textarea rows={22} value={body} onChange={(e) => setBody(e.target.value)} readOnly={!owner} aria-label="Profile body" spellCheck={false} />
            <div className="row" style={{ marginTop: 8 }}>
              <button className="primary" disabled={!owner || body === cur?.body} onClick={() => app.run(() => call("agents.set", { name: sel, body }))}>Save</button>
              <button disabled={!owner} onClick={() => app.run(() => call("agents.reset", { name: sel }))}>{cur?.builtin ? "Reset to built-in" : "Delete"}</button>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

export function GmTools() {
  const app = useApp();
  const ov = useData<any>("overview", { branch: app.branch }, [app.branch, app.rev]);
  const people = useData<any[]>("people.list", { branch: app.branch }, [app.branch, app.rev]);
  const rej = useData<any[]>("rejections.list", { branch: app.branch }, [app.branch, app.rev]);
  const profiles = useData<any[]>("agents.list", {}, [app.rev]);
  const [dry, setDry] = useState(false);
  const [result, setResult] = useState<any>(null);
  const [shock, setShock] = useState({ kind: "drought", days: 120, severity: 70 });
  const [asset, setAsset] = useState("");
  const [sale, setSale] = useState({ buyer: "org_lodge", price: 500 });
  const [fact, setFact] = useState({ text: "", protected: false, gm_only: false });
  const [griev, setGriev] = useState({ from: "", to: "", what: "", weight: 30 });
  const [edit, setEdit] = useState({ person: "", agent_profile: "" });
  const [verify, setVerify] = useState<any>(null);
  const o = ov.data;
  if (!o) return <p className="dim">{ov.error ?? "Loading…"}</p>;
  const cmd = (type: string, payload: Record<string, unknown>) => app.run(async () => setResult({ type, ...(await call("gm.command", { branch: app.branch, type, payload, dry_run: dry })) }));
  const assets = o.assets.filter((a: any) => a.active);
  const ppl = (people.data ?? []).filter((p) => !p.is_pc);
  const pSel = (v: string, on: (v: string) => void, label: string) => (
    <select value={v} onChange={(e) => on(e.target.value)} aria-label={label}><option value="">— {label} —</option>{ppl.map((p) => <option key={p.id} value={p.id}>{p.name} ({p.occupation})</option>)}</select>
  );
  return (
    <>
      <div className="row" style={{ marginBottom: 12 }}>
        <h1 style={{ margin: 0 }}>GM Tools</h1>
        <span style={{ flex: 1 }} />
        <label><input type="checkbox" checked={dry} onChange={(e) => setDry(e.target.checked)} /> Dry run (preview consequences, commit nothing)</label>
      </div>
      <div className="note">Direct intervention, simulation-first: every change here is a typed, validated, permanently logged event with you as its author. The world's rules still apply — you cannot sell what was destroyed, spend coin that is not there, or break protected canon.</div>
      {result && (
        <div className={"card"} style={{ borderColor: "var(--accent2)" }}>
          <h2>{result.dry_run ? "Dry run — nothing committed" : "Committed"} <span className="mono faint">{result.type}</span></h2>
          {result.dry_run ? <><p>Valid. It would produce {result.consequences.length} immediate consequence(s){result.population_change ? ` and change the population by ${result.population_change}` : ""}.</p>{result.consequences.map((c: string, i: number) => <div key={i} className="dim">• {c}</div>)}</> : result.events.map((e: any) => <div key={e.event_id}><span className="faint mono">{e.event_id}</span> {e.summary}</div>)}
        </div>
      )}
      <div className="grid g2">
        <div className="card">
          <h2>Weather shock</h2>
          <div className="row">
            <select value={shock.kind} onChange={(e) => setShock({ ...shock, kind: e.target.value })}><option>drought</option><option>flood</option><option>frost</option></select>
            <label>days</label><input type="number" min={1} max={720} value={shock.days} onChange={(e) => setShock({ ...shock, days: Number(e.target.value) })} style={{ width: 80 }} />
            <label>severity</label><input type="number" min={1} max={100} value={shock.severity} onChange={(e) => setShock({ ...shock, severity: Number(e.target.value) })} style={{ width: 70 }} />
            <button onClick={() => cmd("weather.shock", shock)}>Apply</button>
          </div>
          <p className="faint" style={{ marginBottom: 0 }}>A drought through Seedmonth–Goldmonth cuts the harvest and strands barges: watch prices, hunger, theft, the Compact's doors and the Reeve's edicts follow. Try it on an experiment branch and compare.</p>
        </div>
        <div className="card">
          <h2>Property</h2>
          <div className="row">
            <select value={asset} onChange={(e) => setAsset(e.target.value)} aria-label="Asset"><option value="">— asset —</option>{assets.map((a: any) => <option key={a.id} value={a.id}>{a.name} ({a.owner}){a.protected ? " — protected" : ""}</option>)}</select>
            <button className="danger" disabled={!asset} onClick={() => cmd("asset.destroyed", { asset_id: asset, reason: "GM intervention" })}>Destroy</button>
          </div>
          <div className="row" style={{ marginTop: 8 }}>
            <label>Sell to</label>
            <select value={sale.buyer} onChange={(e) => setSale({ ...sale, buyer: e.target.value })}>{o.orgs.map((g: any) => <option key={g.id} value={g.id}>{g.name}</option>)}</select>
            <label>for</label><input type="number" min={0} value={sale.price} onChange={(e) => setSale({ ...sale, price: Number(e.target.value) })} style={{ width: 90 }} /><span className="dim">marks</span>
            <button disabled={!asset} onClick={() => cmd("asset.sold", { asset_id: asset, buyer: sale.buyer, price_milli: sale.price * 1000 })}>Sell</button>
          </div>
        </div>
        <div className="card">
          <h2>Record a fact</h2>
          <textarea rows={2} value={fact.text} onChange={(e) => setFact({ ...fact, text: e.target.value })} placeholder="e.g. The Reeve's seal was recast in the year 380." aria-label="Fact text" />
          <div className="row" style={{ marginTop: 6 }}>
            <label><input type="checkbox" checked={fact.protected} onChange={(e) => setFact({ ...fact, protected: e.target.checked })} /> protected canon</label>
            <label><input type="checkbox" checked={fact.gm_only} onChange={(e) => setFact({ ...fact, gm_only: e.target.checked })} /> GM-only</label>
            <button disabled={!fact.text.trim()} onClick={() => cmd("fact.recorded", fact)}>Record</button>
          </div>
        </div>
        <div className="card">
          <h2>Plant a grievance</h2>
          <div className="row">{pSel(griev.from, (v) => setGriev({ ...griev, from: v }), "who")}<span className="dim">against</span>{pSel(griev.to, (v) => setGriev({ ...griev, to: v }), "whom")}</div>
          <div className="row" style={{ marginTop: 6 }}>
            <input style={{ flex: 1 }} value={griev.what} onChange={(e) => setGriev({ ...griev, what: e.target.value })} placeholder="cheated my father at the scales" aria-label="Grievance" />
            <label>weight</label><input type="number" min={1} max={80} value={griev.weight} onChange={(e) => setGriev({ ...griev, weight: Number(e.target.value) })} style={{ width: 70 }} />
            <button disabled={!griev.from || !griev.to || !griev.what} onClick={() => cmd("relation.grievance", griev)}>Plant</button>
          </div>
          <p className="faint" style={{ marginBottom: 0 }}>15+ makes rivals, 35+ enemies, 70+ (or one grave wrong) a nemesis who will scheme: slander, sabotage, denunciation, violence, or a bid for someone's office.</p>
        </div>
        <div className="card">
          <h2>Assign an agent profile</h2>
          <div className="row">
            {pSel(edit.person, (v) => setEdit({ ...edit, person: v }), "person")}
            <select value={edit.agent_profile} onChange={(e) => setEdit({ ...edit, agent_profile: e.target.value })} aria-label="Profile"><option value="">— profile —</option>{(profiles.data ?? []).map((a) => <option key={a.name}>{a.name}</option>)}</select>
            <button disabled={!edit.person || !edit.agent_profile} onClick={() => cmd("npc.edited", edit)}>Assign</button>
          </div>
        </div>
        <div className="card">
          <h2>Integrity</h2>
          <div className="row">
            <button onClick={() => app.run(async () => setVerify(await call("world.verify", { branch: app.branch })))}>Replay &amp; verify this timeline</button>
            {verify && <span className={verify.ok ? "good" : "bad"}>{verify.ok ? `✔ ${verify.events_checked.toLocaleString()} events replayed, all hashes agree` : `✘ ${verify.problems.length} problem(s)`}</span>}
          </div>
          {verify && !verify.ok && verify.problems.map((p: string) => <div key={p} className="bad">• {p}</div>)}
        </div>
      </div>
      <div className="card">
        <h2>Rejected commands (diagnostics)</h2>
        <table><tbody>
          {(rej.data ?? []).map((r, i) => <tr key={i}><td className="faint">{r.at.slice(0, 19).replace("T", " ")}</td><td className="mono">{r.event_type}</td><td><span className="tag bad">{r.code}</span></td><td className="dim">{r.message}</td></tr>)}
          {!rej.data?.length && <tr><td className="dim">None. A rejected command never becomes an event; it is only noted here.</td></tr>}
        </tbody></table>
      </div>
    </>
  );
}
