import { useEffect, useState } from "react";
import { call, marks, useData } from "../api";
import { useApp } from "../App";

const statusTag = (s: string) => (s === "nemesis" ? "bad" : s === "enemy" ? "bad" : s === "rival" ? "warn" : s === "ally" || s === "friend" ? "good" : "");

function PersonLink({ id, name }: { id: string; name: string }) {
  const app = useApp();
  return <a onClick={() => app.go("people", id)}>{name}</a>;
}

export function People() {
  const app = useApp();
  const [q, setQ] = useState("");
  const [dead, setDead] = useState(false);
  const [sel, setSel] = useState<string | null>(app.focus);
  const [proposal, setProposal] = useState<any>(null);
  useEffect(() => { if (app.focus) setSel(app.focus); }, [app.focus]);
  const list = useData<any[]>("people.list", { branch: app.branch, q, include_dead: dead }, [app.branch, app.rev, q, dead]);
  const p = useData<any>(sel ? "person.get" : null, { branch: app.branch, id: sel }, [app.branch, app.rev, sel]);
  const gm = app.role !== "player";
  const d = p.data;
  useEffect(() => setProposal(null), [sel]);

  return (
    <>
      <div className="row" style={{ marginBottom: 12 }}>
        <h1 style={{ margin: 0 }}>People</h1>
        <input placeholder="Search name or trade…" value={q} onChange={(e) => setQ(e.target.value)} aria-label="Search people" />
        <label><input type="checkbox" checked={dead} onChange={(e) => setDead(e.target.checked)} /> include the dead and departed</label>
        <span className="faint">{list.data?.length ?? 0} shown</span>
      </div>
      <div className="grid g23">
        <div className="card" style={{ padding: 0 }}>
          <div style={{ maxHeight: "76vh", overflowY: "auto" }}>
            <table>
              <thead><tr><th>Name</th><th>Trade</th><th>Age</th><th>Tier</th>{gm && <th>State</th>}</tr></thead>
              <tbody>
                {(list.data ?? []).map((x) => (
                  <tr key={x.id} className={"click " + (x.id === sel ? "sel" : "")} onClick={() => setSel(x.id)}>
                    <td>{x.name}{x.is_pc && <span className="tag acc">PC</span>}{x.status !== "alive" && <span className="tag">{x.status}</span>}{x.flags.map((f: string) => <span className="tag" key={f}>{f.replace("leader:org_", "head of ").replace("_", " ")}</span>)}</td>
                    <td className="dim">{x.occupation}</td>
                    <td className="dim">{x.age}</td>
                    <td><span className={"tag " + (x.tier >= 3 ? "acc" : "")}>T{x.tier}</span></td>
                    {gm && <td>{x.hunger > 40 && <span className="tag bad">hungry</span>}{x.jailed && <span className="tag warn">gaoled</span>}{x.enemies > 0 && <span className="tag bad">{x.enemies} foe{x.enemies > 1 ? "s" : ""}</span>}</td>}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
        <div>
          {!d ? <div className="card"><p className="dim">{p.error ?? "Select someone to inspect them."}</p></div> : (
            <>
              <div className="card">
                <div className="row">
                  <h2 style={{ margin: 0 }}>{d.name}</h2>
                  <span className={"tag " + (d.tier >= 3 ? "acc" : "")}>Tier {d.tier}</span>
                  {d.status !== "alive" && <span className="tag bad">{d.status}</span>}
                  {d.jailed && <span className="tag warn">in the Stone Cellar</span>}
                </div>
                <p className="dim">{d.age}-year-old {d.occupation}{d.org ? `, sworn to ${d.org}` : ""} — {d.appearance || "no description"}. Currently at {d.where ?? "—"}. Speaks {d.languages.join(", ")}.</p>
                <dl className="kv">
                  <dt>Household</dt><dd>{d.household_name}</dd>
                  <dt>Spouse</dt><dd>{d.spouse ? <PersonLink {...d.spouse} /> : "—"}</dd>
                  <dt>Parents</dt><dd>{d.parents.length ? d.parents.map((x: any, i: number) => <span key={x.id}>{i > 0 && ", "}<PersonLink {...x} />{!x.alive && " †"}</span>) : "not recorded"}</dd>
                  <dt>Children</dt><dd>{d.children.length ? d.children.map((x: any, i: number) => <span key={x.id}>{i > 0 && ", "}<PersonLink {...x} />{!x.alive && " †"}</span>) : "—"}</dd>
                  {d.temperament && <><dt>Temperament</dt><dd>{d.temperament.join(", ")}</dd></>}
                  {d.goals?.length > 0 && <><dt>Goals</dt><dd>{d.goals.join(" · ")}</dd></>}
                  {d.stats && <><dt>Game stats</dt><dd className="mono">{d.stats.class} L{d.stats.level} · HP {d.stats.hp}/{d.stats.max_hp} · AC {d.stats.ac} · atk +{d.stats.attack_bonus} {d.stats.dmg_dice}d{d.stats.dmg_sides}+{d.stats.dmg_bonus} · [{d.stats.abilities.join(" ")}]</dd></>}
                  {d.health != null && <><dt>Body</dt><dd>health {d.health}/100 · hunger {d.hunger}/100</dd></>}
                  {d.household_accounts && <><dt>Household stock</dt><dd>{Object.entries(d.household_accounts).filter(([, v]) => (v as number) > 0).map(([k, v]) => `${marks(v as number)} ${k}`).join(" · ") || "nothing"}</dd></>}
                  {d.agent_profile && <><dt>Agent profile</dt><dd className="mono">{d.agent_profile}.md</dd></>}
                </dl>
                {gm && d.status === "alive" && !d.is_pc && (
                  <div className="row" style={{ marginTop: 10 }}>
                    <span className="faint">Set tier:</span>
                    {[1, 2, 3, 4].map((t) => <button key={t} className="small" disabled={d.tier === t} onClick={() => app.run(() => call("gm.command", { branch: app.branch, type: "npc.edited", payload: { person: d.id, tier: t } }))}>T{t}</button>)}
                    <button className="small" onClick={() => app.run(async () => setProposal(await call("npc.deliberate", { branch: app.branch, id: d.id })))}>What do they do next?</button>
                  </div>
                )}
                {proposal && (
                  <div className="note" style={{ marginTop: 10 }}>
                    <b>Proposal</b> (source: {proposal.source}) — <span className="mono">{JSON.stringify(proposal.proposal)}</span>
                    {proposal.reason && <div>“{proposal.reason}”</div>}
                    {proposal.ai_error && <div className="warn">AI unavailable ({proposal.ai_error.code}); used temperament policy.</div>}
                    {proposal.rejection && <div className="warn">The model's proposal was rejected by the rules ({proposal.rejection.code}); showing the policy fallback.</div>}
                    <div className="row" style={{ marginTop: 6 }}>
                      <button className="primary small" onClick={() => app.run(async () => { await call("gm.command", { branch: app.branch, type: "npc.action", payload: proposal.proposal, origin: proposal.source === "policy" ? "human" : "ai_proposal" }); setProposal(null); })}>Approve &amp; commit</button>
                      <button className="small" onClick={() => setProposal(null)}>Discard</button>
                    </div>
                  </div>
                )}
              </div>
              {d.regarded_by?.length > 0 && (
                <div className="card">
                  <h2>Who holds a grudge against them</h2>
                  {d.regarded_by.map((r: any) => <p key={r.id}><span className={"tag " + statusTag(r.status)}>{r.status}</span><PersonLink id={r.id} name={r.name} /> — {r.grievances.map((g: any) => g.what).join("; ")}</p>)}
                </div>
              )}
              {d.relations && (
                <div className="card">
                  <h2>How they regard others</h2>
                  <div className="scroll" style={{ maxHeight: 260 }}>
                    <table><tbody>
                      {[...d.relations].sort((a: any, b: any) => b.hostility - a.hostility || b.fondness - a.fondness).map((r: any) => (
                        <tr key={r.id}>
                          <td><PersonLink id={r.id} name={r.name} /><div className="faint">{r.kinds.join(", ")}</div></td>
                          <td><span className={"tag " + statusTag(r.status)}>{r.status}</span></td>
                          <td className="dim">trust {r.trust} · fond {r.fondness}{r.grievances.length > 0 && <div className="bad">{r.grievances.map((g: any) => g.what).join("; ")}</div>}</td>
                        </tr>
                      ))}
                    </tbody></table>
                  </div>
                </div>
              )}
              {d.memories && (
                <div className="grid g2">
                  <div className="card">
                    <h2>Memories</h2>
                    {d.memories.length ? [...d.memories].reverse().map((m: any, i: number) => <p key={i} style={{ margin: "0 0 6px" }}><span className="faint">day {m.day}</span> {m.text}</p>) : <p className="dim">Nothing that stuck.</p>}
                  </div>
                  <div className="card">
                    <h2>What they believe</h2>
                    {d.knows.length ? d.knows.map((k: any) => <p key={k.id} style={{ margin: "0 0 6px" }}><span className={"tag " + (k.true ? "good" : "bad")}>{k.true ? "true" : "false"}</span><span className="faint">{k.confidence}%</span> {k.text}</p>) : <p className="dim">Nothing beyond common knowledge.</p>}
                    <p className="faint" style={{ marginBottom: 0 }}>True/false is GM-only. When this person speaks, only these beliefs are available to them.</p>
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </>
  );
}

export function Factions() {
  const app = useApp();
  const r = useData<any>("relations.notable", { branch: app.branch }, [app.branch, app.rev]);
  const [show, setShow] = useState("rival");
  if (!r.data) return <p className="dim">{r.error ?? "Loading…"}</p>;
  const rank: Record<string, number> = { rival: 1, enemy: 2, nemesis: 3 };
  const feuds = r.data.feuds.filter((f: any) => rank[f.status] >= rank[show] && f.from_alive);
  const party = r.data.feuds.filter((f: any) => f.to_is_pc && f.from_alive);
  return (
    <>
      <h1>Factions &amp; Nemeses</h1>
      <div className="grid g3">
        {r.data.org_relations.map((o: any) => (
          <div className="card" key={o.a + o.b}>
            <h2>{o.a_name.replace("The ", "")} ↔ {o.b_name.replace("The ", "")}</h2>
            <div className="row"><span className={"tag " + (o.status === "hostile" ? "bad" : o.status === "tense" ? "warn" : o.status === "allied" || o.status === "friendly" ? "good" : "")}>{o.status}</span><span className="dim">standing {o.score > 0 ? "+" : ""}{o.score}</span></div>
            <div className="bar" style={{ margin: "8px 0" }}><i style={{ width: `${(o.score + 100) / 2}%`, background: o.score < -20 ? "var(--bad)" : o.score > 20 ? "var(--good)" : "var(--accent)" }} /></div>
            {o.history.length ? o.history.slice().reverse().map((h: any, i: number) => <div key={i} className="dim" style={{ fontSize: 12 }}>day {h[0]}: {h[1]}</div>) : <div className="faint">No formal change of standing yet.</div>}
          </div>
        ))}
      </div>
      {party.length > 0 && (
        <div className="card" style={{ borderColor: "var(--bad)" }}>
          <h2>Sworn against the party</h2>
          {party.map((f: any) => (
            <div key={f.from + f.to} style={{ marginBottom: 10 }}>
              <span className={"tag " + statusTag(f.status)}>{f.status}</span><b><PersonLink id={f.from} name={f.from_name} /></b> <span className="dim">({f.from_trade}, tier {f.from_tier}, level {f.from_level})</span> against {f.to_name}
              <div className="dim">{f.grievances.map((g: any) => `day ${g.day}: ${g.what}`).join(" · ")}</div>
              <div>{(f.from_flags ?? []).filter((x: string) => /scarred|fled|bested|hunting/.test(x)).map((x: string) => <span className="tag warn" key={x}>{x.split(":")[0].replace("_", " ")}</span>)}</div>
            </div>
          ))}
          <p className="faint" style={{ marginBottom: 0 }}>Opponents who survive the party remember it, grow stronger, and look for the party in play. They never resolve harm to a player character off-screen.</p>
        </div>
      )}
      <div className="card">
        <div className="row">
          <h2 style={{ margin: 0 }}>Feuds in the town</h2>
          <span style={{ flex: 1 }} />
          {["rival", "enemy", "nemesis"].map((s) => <button key={s} className={show === s ? "primary" : ""} onClick={() => setShow(s)}>{s === "rival" ? "all quarrels" : s === "enemy" ? "enemies +" : "nemeses only"}</button>)}
        </div>
        <table style={{ marginTop: 8 }}>
          <thead><tr><th>Who</th><th /><th>Against</th><th>Why</th><th>Heat</th></tr></thead>
          <tbody>
            {feuds.slice(0, 120).map((f: any) => (
              <tr key={f.from + f.to}>
                <td><PersonLink id={f.from} name={f.from_name} /><div className="faint">{f.from_trade}</div></td>
                <td><span className={"tag " + statusTag(f.status)}>{f.status}</span></td>
                <td><PersonLink id={f.to} name={f.to_name} />{!f.to_alive && " †"}<div className="faint">{f.to_trade}</div></td>
                <td className="dim">{f.grievances.map((g: any) => g.what).join("; ")}</td>
                <td>{f.hostility}</td>
              </tr>
            ))}
            {!feuds.length && <tr><td colSpan={5} className="dim">The town is at peace at this level. Hunger, injustice and unpaid wages change that.</td></tr>}
          </tbody>
        </table>
      </div>
    </>
  );
}

export function Cases() {
  const app = useApp();
  const c = useData<any[]>("crimes.list", { branch: app.branch }, [app.branch, app.rev]);
  const [sel, setSel] = useState<string | null>(null);
  const gm = app.role !== "player";
  const cur = (c.data ?? []).find((x) => x.id === sel) ?? (c.data ?? [])[0];
  return (
    <>
      <h1>Cases &amp; Evidence</h1>
      <div className="note">{gm ? "GM view: objective truth (culprit, motive) beside what the watch and the town believe. Witnesses can be mistaken; enemies can lie; the watch can gaol the wrong person." : "What the party has learned. Search crime scenes and talk to people to find more; lay an accusation before the watch from the Play screen when you have grounds."}</div>
      <div className="grid g23">
        <div className="card" style={{ padding: 0 }}>
          <div style={{ maxHeight: "72vh", overflowY: "auto" }}>
            <table>
              <thead><tr><th>When</th><th>Case</th><th>Status</th></tr></thead>
              <tbody>
                {(c.data ?? []).map((x) => (
                  <tr key={x.id} className={"click " + (cur?.id === x.id ? "sel" : "")} onClick={() => setSel(x.id)}>
                    <td className="dim">{x.date}</td>
                    <td>{x.kind} — {x.victim}<div className="faint">{x.location}</div></td>
                    <td><span className={"tag " + (x.status === "open" ? "warn" : x.status === "wrongful" ? "bad" : x.status === "solved" ? "good" : "")}>{x.status === "wrongful" ? "wrong person gaoled" : x.status}</span></td>
                  </tr>
                ))}
                {!c.data?.length && <tr><td colSpan={3} className="dim">No crimes on record. (Advance time — or wait for a lean season.)</td></tr>}
              </tbody>
            </table>
          </div>
        </div>
        {cur && (
          <div className="card">
            <h2>{cur.kind} against {cur.victim}</h2>
            <dl className="kv">
              <dt>When / where</dt><dd>{cur.date} · {cur.location}</dd>
              {cur.amount > 0 && <><dt>Taken</dt><dd>{marks(cur.amount)} {cur.commodity}</dd></>}
              <dt>Accused</dt><dd>{cur.accused ? <PersonLink id={cur.accused_id} name={cur.accused} /> : "no one yet"}</dd>
              {gm && <><dt>Truth <span className="tag gm">GM</span></dt><dd><PersonLink id={cur.culprit_id} name={cur.culprit} /> — {cur.motive}</dd></>}
            </dl>
            {gm && cur.suspicion.length > 0 && <><h3 style={{ marginTop: 12 }}>The watch's suspicion (arrest at 100)</h3>{[...cur.suspicion].sort((a: any, b: any) => b.score - a.score).map((s: any) => <div key={s.id}><PersonLink id={s.id} name={s.name} /> — {s.score}{s.id === cur.culprit_id ? <span className="tag good">guilty</span> : <span className="tag bad">innocent</span>}</div>)}</>}
            <h3 style={{ marginTop: 12 }}>Physical evidence</h3>
            {cur.evidence.length ? cur.evidence.map((e: any, i: number) => (
              <div key={i} style={{ marginBottom: 8 }}>
                <b>{e.kind}</b>{gm && <>{e.found_by_watch && <span className="tag">watch has it</span>}{e.found_by_party && <span className="tag acc">party found it</span>}<span className="faint"> quality {e.quality}</span></>} — {e.desc}
                <div className="faint">{e.custody.join(" → ")}</div>
              </div>
            )) : <p className="dim">{gm ? "None left at the scene." : "The party has found none. Search the scene."}</p>}
            <h3 style={{ marginTop: 12 }}>{gm ? "Testimony and talk in the town" : "Testimony the party has heard"}</h3>
            {(gm ? cur.claims : cur.testimony).length ? (gm ? cur.claims : cur.testimony).map((t: any, i: number) => <div key={i}>{gm && <span className={"tag " + (t.true ? "good" : "bad")}>{t.true ? "true" : "false"}</span>}{t.text}{gm && <span className="faint"> — {t.kind}, known to {t.known_by}</span>}</div>) : <p className="dim">None.</p>}
          </div>
        )}
      </div>
    </>
  );
}
