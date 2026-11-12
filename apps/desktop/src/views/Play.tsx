import { useEffect, useRef, useState } from "react";
import { call, marks, price, useData } from "../api";
import { useApp } from "../App";

const SKILLS = ["perception", "investigation", "insight", "persuasion", "deception", "intimidation", "stealth", "athletics", "arcana", "medicine", "history", "survival"];
const KIND_ICON: Record<string, string> = { market: "⚖", civic: "⚑", gaol: "▦", warehouse: "▣", quay: "⚓", guildhall: "⛵", tavern: "☕", shrine: "✧", district: "⌂", farmland: "❦", ruin: "☗", bridge: "≋" };

export function TownMap({ onMove, compact }: { onMove?: (id: string) => void; compact?: boolean }) {
  const app = useApp();
  const m = useData<any>("map.get", { branch: app.branch }, [app.branch, app.rev]);
  const [hover, setHover] = useState<any>(null);
  if (!m.data) return <p className="dim">{m.error ?? "Loading…"}</p>;
  const locs = m.data.locations as any[];
  const by = Object.fromEntries(locs.map((l) => [l.id, l]));
  const here = m.data.party ? by[m.data.party] : null;
  const reachable = new Set<string>((here?.links ?? []).map((l: any) => l[0]));
  const riverW = 14 + (m.data.river / 100) * 26;
  const map = (
    <div className={compact ? "" : "card"}>
      <svg viewBox="60 60 880 580" style={{ width: "100%", background: "#191511", borderRadius: 6 }} role="img" aria-label="Map of Marrowick">
        <path d="M 40 372 C 180 360, 300 330, 500 352 S 800 400, 960 372" fill="none" stroke="#2f4a55" strokeWidth={riverW} strokeLinecap="round" />
        <text x={880} y={352} style={{ fill: "#4f7684", fontStyle: "italic" }}>River Sable →</text>
        {locs.flatMap((l) => l.links.filter((k: any) => l.id < k[0]).map((k: any) => (
          <line key={l.id + k[0]} x1={l.x} y1={l.y} x2={by[k[0]].x} y2={by[k[0]].y} stroke={here && (l.id === here.id || k[0] === here.id) ? "#8a7448" : "#3a322a"} strokeWidth={here && (l.id === here.id || k[0] === here.id) ? 2 : 1.2} strokeDasharray="5 4" />
        )))}
        {locs.map((l) => {
          const isHere = here?.id === l.id;
          const can = onMove && reachable.has(l.id);
          return (
            <g key={l.id} className="maploc" transform={`translate(${l.x},${l.y})`} onMouseEnter={() => setHover(l)} onMouseLeave={() => setHover(null)} onClick={() => can && onMove!(l.id)} tabIndex={can ? 0 : -1} onKeyDown={(e) => e.key === "Enter" && can && onMove!(l.id)} role={can ? "button" : undefined} aria-label={l.name}>
              <circle r={l.residential ? 15 : 13} fill={isHere ? "#d9a441" : l.residential ? "#2c3a33" : "#33291f"} stroke={can ? "#d9a441" : l.open_cases ? "#e0685a" : "#5a4d3d"} strokeWidth={can || l.open_cases ? 2 : 1} />
              <text textAnchor="middle" y={4} style={{ fontSize: 13, fill: isHere ? "#1a1408" : "#cdbf9f" }}>{KIND_ICON[l.kind] ?? "•"}</text>
              <text textAnchor="middle" y={compact ? 34 : 30} style={{ fill: isHere ? "#d9a441" : "#b7aa91", fontSize: compact ? 17 : 11.5 }}>{l.name}</text>
              {l.people != null && !compact && <text textAnchor="middle" y={-19} style={{ fontSize: 10, fill: "#736a5b" }}>{l.people} here</text>}
            </g>
          );
        })}
      </svg>
      <p className="dim" style={{ minHeight: 40, margin: "8px 0 0" }}>
        {hover ? <><b>{hover.name}</b> — {hover.desc}{hover.open_cases > 0 && <span className="tag bad" style={{ marginLeft: 6 }}>{hover.open_cases} open case{hover.open_cases > 1 ? "s" : ""}</span>}</> : onMove ? "Gold-ringed places are one walk away: click to go there." : "Hover a place for its description. Counts show where residents are right now (GM view)."}
      </p>
    </div>
  );
  return compact ? map : <><h1>Town Map — Marrowick</h1>{map}</>;
}

export function Play() {
  const app = useApp();
  const sc = useData<any>("play.scene", { branch: app.branch }, [app.branch, app.rev]);
  const journal = useData<any>("journal.get", { branch: app.branch }, [app.branch, app.rev]);
  const cases = useData<any[]>("crimes.list", { branch: app.branch }, [app.branch, app.rev]);
  const [npc, setNpc] = useState<string | null>(null);
  const [chat, setChat] = useState<Record<string, { who: string; text: string; meta?: string }[]>>({});
  const [text, setText] = useState("");
  const [feed, setFeed] = useState<string[]>([]);
  const [narr, setNarr] = useState<{ text: string; source: string } | null>(null);
  const [skill, setSkill] = useState("perception");
  const [dc, setDc] = useState(12);
  const [trade, setTrade] = useState({ good: "grain", qty: 5 });
  const [accuse, setAccuse] = useState({ crime: "", person: "" });
  const chatEnd = useRef<HTMLDivElement>(null);
  const [showEnc, setShowEnc] = useState(false);
  const s = sc.data;
  useEffect(() => { chatEnd.current?.scrollIntoView({ block: "nearest" }); }, [chat, npc]);
  useEffect(() => { setNarr(null); }, [s?.location?.id]);
  useEffect(() => { if (s && npc && !s.people.find((p: any) => p.id === npc)) setNpc(null); }, [s, npc]);

  const cur = app.branches.find((b) => b.branch_id === app.branch);
  if (!cur?.campaign) return <div className="card"><h2>Play happens in a campaign</h2><p className="dim">Select a campaign timeline in the sidebar, or start one.</p><button className="primary" onClick={() => app.go("campaigns")}>Campaigns &amp; Branches</button></div>;
  if (!s) return <div className="card"><h2>{sc.error?.includes("party") || !cur.party.length ? "No characters yet" : "Loading…"}</h2>{sc.error && <p className="dim">{sc.error}</p>}<button className="primary" onClick={() => app.go("campaigns")}>Add a character</button></div>;

  const act = async (type: string, payload: Record<string, unknown> = {}) => {
    const r = await app.run(() => call("play.act", { branch: app.branch, type, payload }));
    if (r) setFeed((f) => [...r.events.filter((e: any) => e.derived || type !== "play.waited").map((e: any) => e.summary), ...f].slice(0, 40));
    return r;
  };
  const say = async () => {
    if (!npc || !text.trim()) return;
    const mine = text;
    setText("");
    setChat((c) => ({ ...c, [npc]: [...(c[npc] ?? []), { who: "me", text: mine }] }));
    const r = await app.run(() => call("play.talk", { branch: app.branch, npc, text: mine }));
    if (r) setChat((c) => ({ ...c, [npc]: [...(c[npc] ?? []), { who: "them", text: r.reply, meta: [r.source === "offline" ? "offline voice" : r.source, r.disclosed.length ? `${r.disclosed.length} thing(s) learned → journal` : "", r.attitude > 0 ? "warmed to you" : r.attitude < 0 ? "cooled toward you" : "", r.ai_error ? `AI unavailable: ${r.ai_error.code}` : ""].filter(Boolean).join(" · ") }] }));
  };
  const enc = s.encounter;
  const fighting = enc?.active;
  if (fighting && !showEnc) setShowEnc(true);
  const lead = s.party[0];
  const who = s.people.find((p: any) => p.id === npc);
  const caster = s.party.some((p: any) => p.class === "wizard" || p.class === "cleric");
  const openCases = (cases.data ?? []).filter((c) => c.status === "open");
  const suspects = Array.from(new Map((journal.data?.known_claims ?? []).filter((k: any) => k.subject && k.crime === accuse.crime).map((k: any) => [k.subject, k.subject_name])).entries());

  return (
    <>
      <div className="grid g32">
        <div>
          <div className="card">
            <div className="row">
              <h2 style={{ margin: 0 }}>{s.location.name}</h2>
              <span className="dim">{s.time} · {s.weather}</span>
              <span style={{ flex: 1 }} />
              <button className="small" onClick={() => app.run(async () => setNarr(await call("play.narrate", { branch: app.branch })))}>{cur.campaign.gm_mode === "human" ? "Describe" : "AI GM: describe"}</button>
            </div>
            <div className="prose" style={{ marginTop: 8 }}><p>{narr?.text ?? s.narration}</p></div>
            {narr && <div className="faint">narration: {narr.source}</div>}
            {s.notices.length > 0 && <details><summary className="dim">Posted notices and common talk ({s.notices.length})</summary>{s.notices.map((n: any) => <div key={n.id} className="dim">• {n.text}</div>)}</details>}
          </div>

          {fighting || (enc && showEnc) ? (
            <div className="card" style={{ borderColor: fighting ? "var(--bad)" : undefined }}>
              <div className="row"><h2 style={{ margin: 0 }}>{fighting ? `Combat — round ${enc.round}` : `Combat over: ${enc.outcome}`}</h2><span className="tag">{s.rules}</span><span style={{ flex: 1 }} />{!fighting && <button className="small" onClick={() => setShowEnc(false)}>Close</button>}</div>
              <table><tbody>
                {enc.combatants.map((c: any, i: number) => (
                  <tr key={c.id} className={fighting && i === enc.turn ? "sel" : ""}>
                    <td>{fighting && i === enc.turn ? "▶ " : ""}{c.name} <span className="faint">{c.side}</span></td>
                    <td style={{ width: 160 }}><div className="bar"><i style={{ width: `${(c.hp / c.max_hp) * 100}%`, background: c.side === "party" ? "var(--accent2)" : "var(--bad)" }} /></div></td>
                    <td className="mono">{c.hp}/{c.max_hp} · AC {c.ac}</td>
                    <td>{c.down && <span className="tag bad">down</span>}{c.fled && <span className="tag warn">fled</span>}{c.dodging && <span className="tag">dodging</span>}{c.surprised && <span className="tag">surprised</span>}</td>
                    <td>{fighting && c.side === "foes" && !c.down && !c.fled && <button className="small" onClick={() => act("combat.acted", { action: "attack", target: c.id })}>Attack</button>}</td>
                  </tr>
                ))}
              </tbody></table>
              {fighting && (
                <div className="row" style={{ marginTop: 8 }}>
                  <button onClick={() => act("combat.acted", { action: "dodge" })}>Dodge</button>
                  <button disabled={!lead?.potions} onClick={() => act("combat.acted", { action: "potion" })}>Potion ({s.rules === "srd-5.2.1" ? "bonus action" : "action"}) ×{lead?.potions ?? 0}</button>
                  <button className="danger" onClick={() => act("combat.acted", { action: "flee" })}>Flee</button>
                </div>
              )}
              <div className="log" style={{ marginTop: 8 }}>{enc.log.join("\n")}</div>
            </div>
          ) : null}

          {!fighting && (
            <div className="card">
              <h2>People here ({s.people.length})</h2>
              {!s.people.length && <p className="dim">No one is about at this hour.</p>}
              <div className="row" style={{ marginBottom: 10 }}>
                {s.people.map((p: any) => (
                  <button key={p.id} className={npc === p.id ? "primary" : ""} onClick={() => setNpc(p.id)} style={p.hostile ? { borderColor: "var(--bad)" } : undefined} title={`${p.trade}, about ${p.age}`}>
                    {p.name}{p.regard !== "neutral" && <span className={"tag " + (["enemy", "nemesis", "rival"].includes(p.regard) ? "bad" : "good")} style={{ marginLeft: 5 }}>{p.regard}</span>}
                  </button>
                ))}
              </div>
              {who && (
                <>
                  <div className="dim">Speaking with <b>{who.name}</b> — {who.trade}, {who.appearance}. <a onClick={() => app.go("people", who.id)}>inspect</a></div>
                  <div className="chat" style={{ margin: "8px 0" }} aria-live="polite">
                    {(chat[who.id] ?? []).map((l, i) => <div key={i} className={l.who === "me" ? "me" : "them"}>{l.text}{l.meta && <div className="faint" style={{ fontFamily: "var(--sans)", fontSize: 11, marginTop: 3 }}>{l.meta}</div>}</div>)}
                    {!(chat[who.id] ?? []).length && <div className="faint">Say anything. They answer only from what they know, how they feel about you, and what kind of person they are.</div>}
                    <div ref={chatEnd} />
                  </div>
                  <div className="row">
                    <input style={{ flex: 1 }} value={text} onChange={(e) => setText(e.target.value)} onKeyDown={(e) => e.key === "Enter" && say()} placeholder={`Say something to ${who.name.split(" ")[0]}…`} aria-label="What you say" />
                    <button className="primary" onClick={say}>Say</button>
                    <button onClick={() => act("play.gave", { to: who.id, commodity: "coin", amount_milli: 5000 })}>Give 5 marks</button>
                    <button className="danger" onClick={() => window.confirm(`Attack ${who.name}? Violence has consequences here: witnesses, the watch, kin who remember.`) && act("combat.started", { targets: [who.id], nonlethal: false })}>Attack</button>
                    <button onClick={() => act("combat.started", { targets: [who.id], nonlethal: true })}>Brawl (non-lethal)</button>
                  </div>
                </>
              )}
            </div>
          )}

          {!fighting && (
            <div className="card">
              <h2>Actions</h2>
              <div className="row">
                <button onClick={() => act("play.searched")}>Search this place (30 min)</button>
                <button onClick={() => act("play.waited", { minutes: 60 })}>Wait an hour</button>
                <button onClick={() => act("play.waited", { minutes: Math.max(1, (24 * 60 - parseInt(s.time) * 60 - parseInt(s.time.slice(3)) + 8 * 60) % 1440 || 1440) })}>Wait until morning</button>
                <button onClick={() => act("play.rested")}>Long rest (8 h)</button>
                {caster && <button onClick={() => act("play.cast", { level: 1, effect: "reveal" })}>Cast: reveal what is hidden (4 thaums)</button>}
                {caster && <button onClick={() => act("play.cast", { level: 1, effect: "heal" })}>Cast: heal (4 thaums)</button>}
              </div>
              <div className="row" style={{ marginTop: 10 }}>
                <label>Skill check</label>
                <select value={skill} onChange={(e) => setSkill(e.target.value)}>{SKILLS.map((k) => <option key={k}>{k}</option>)}</select>
                <label>DC</label><input type="number" value={dc} min={1} max={30} onChange={(e) => setDc(Number(e.target.value))} style={{ width: 60 }} />
                <button onClick={() => act("play.checked", { skill, dc })}>Roll</button>
                <button onClick={() => act("play.checked", { skill, dc, adv: 1 })}>with advantage</button>
                <button onClick={() => act("play.checked", { skill, dc, adv: -1 })}>with disadvantage</button>
              </div>
              {s.prices && (
                <div className="row" style={{ marginTop: 10 }}>
                  <label>Trade with the Compact's factor</label>
                  <select value={trade.good} onChange={(e) => setTrade({ ...trade, good: e.target.value })}>{Object.keys(s.prices).map((g) => <option key={g} value={g}>{g} @ {price(s.prices[g])}</option>)}</select>
                  <input type="number" min={1} value={trade.qty} onChange={(e) => setTrade({ ...trade, qty: Number(e.target.value) })} style={{ width: 70 }} />
                  <button onClick={() => act("play.traded", { good: trade.good, qty_milli: trade.qty * 1000 })}>Buy (+8%)</button>
                  <button onClick={() => act("play.traded", { good: trade.good, qty_milli: -trade.qty * 1000 })}>Sell (−8%)</button>
                </div>
              )}
              {s.stores.length > 0 && (
                <div className="row" style={{ marginTop: 10 }}>
                  <label>Larceny</label>
                  {s.stores.map((o: any) => <button key={o.id} className="danger" onClick={() => act("play.stole", { from: o.id, commodity: "grain", amount_milli: 20000 })}>Steal grain from {o.name.replace("The ", "")}</button>)}
                </div>
              )}
              {openCases.length > 0 && (
                <div className="row" style={{ marginTop: 10 }}>
                  <label>Accuse before the watch</label>
                  <select value={accuse.crime} onChange={(e) => setAccuse({ crime: e.target.value, person: "" })}><option value="">— case —</option>{openCases.map((c) => <option key={c.id} value={c.id}>{c.kind} · {c.victim} · {c.date}</option>)}</select>
                  <select value={accuse.person} onChange={(e) => setAccuse({ ...accuse, person: e.target.value })}><option value="">— named in testimony —</option>{suspects.map(([id, n]) => <option key={id as string} value={id as string}>{n as string}</option>)}</select>
                  <button disabled={!accuse.crime || !accuse.person} onClick={() => act("play.accused", accuse)}>Accuse</button>
                </div>
              )}
            </div>
          )}
          {feed.length > 0 && <div className="card"><h2>What just happened</h2><div className="log">{feed.join("\n")}</div></div>}
        </div>

        <div>
          <div className="card">
            <h2>Party</h2>
            {s.party.map((p: any) => (
              <div key={p.id} style={{ marginBottom: 8 }}>
                <b>{p.name}</b> <span className="dim">{p.class} {p.level}</span> {p.jailed && <span className="tag warn">gaoled</span>}{p.exhaustion > 0 && <span className="tag warn">exhaustion {p.exhaustion}</span>}
                <div className="bar"><i style={{ width: `${(p.hp / p.max_hp) * 100}%` }} /></div>
                <div className="mono faint">HP {p.hp}/{p.max_hp} · AC {p.ac} · STR {p.abilities[0]} DEX {p.abilities[1]} CON {p.abilities[2]} INT {p.abilities[3]} WIS {p.abilities[4]} CHA {p.abilities[5]}</div>
              </div>
            ))}
            <div className="dim">Purse: {marks(s.purse.coin)} marks{Object.entries(s.purse).filter(([k, v]) => k !== "coin" && (v as number) > 0).map(([k, v]) => ` · ${marks(v as number)} ${k}`)}</div>
            <div className="faint">Ambient potential here: {s.ambient_potential} thaums</div>
          </div>
          <div className="card"><h2>Go somewhere</h2><TownMap compact onMove={(id) => !fighting && act("play.moved", { to: id })} />
            <div className="row" style={{ marginTop: 6 }}>{s.exits.map((e: any) => <button key={e.id} className="small" disabled={fighting} onClick={() => act("play.moved", { to: e.id })}>{e.name} ({e.minutes} min)</button>)}</div>
          </div>
          <div className="card">
            <h2>Journal — what your characters know</h2>
            <div className="scroll" style={{ maxHeight: 300 }}>
              {(journal.data?.journal ?? []).slice().reverse().map((j: any, i: number) => <p key={i} style={{ margin: "0 0 6px" }}><span className="tag">{j.kind}</span><span className="faint">day {j.day}</span> {j.text}</p>)}
              {!journal.data?.journal?.length && <p className="dim">Empty. Talk to people; search where things happened.</p>}
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
