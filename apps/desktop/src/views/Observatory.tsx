import { useState } from "react";
import { call, marks, price, useData } from "../api";
import { useApp } from "../App";

const COLORS: Record<string, string> = { grain: "#d9a441", fish: "#7fb7a4", ale: "#c98b5a", cloth: "#b58fd0", timber: "#8fa86b", tools: "#9cc4e4" };

/** Small dependency-free line chart. `series` values are plotted on a shared y-scale. */
export function LineChart({ series, height = 180, format = (v: number) => String(Math.round(v)), x0 = 0 }: { series: { name: string; color: string; values: number[] }[]; height?: number; format?: (v: number) => string; x0?: number }) {
  const W = 760, H = height, pl = 46, pr = 8, pt = 8, pb = 20;
  const n = Math.max(1, ...series.map((s) => s.values.length));
  const all = series.flatMap((s) => s.values);
  if (!all.length) return <p className="dim">No history yet — advance time.</p>;
  const lo = Math.min(...all, 0), hi = Math.max(...all, 1);
  const x = (i: number) => pl + (i / Math.max(1, n - 1)) * (W - pl - pr);
  const y = (v: number) => pt + (1 - (v - lo) / (hi - lo || 1)) * (H - pt - pb);
  return (
    <>
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={"Chart of " + series.map((s) => s.name).join(", ")} style={{ width: "100%" }}>
        {[0, 0.25, 0.5, 0.75, 1].map((t) => {
          const v = lo + t * (hi - lo);
          return (
            <g key={t}>
              <line x1={pl} x2={W - pr} y1={y(v)} y2={y(v)} stroke="#3a322a" strokeDasharray="2 4" />
              <text x={pl - 6} y={y(v) + 4} textAnchor="end">{format(v)}</text>
            </g>
          );
        })}
        <text x={pl} y={H - 4}>day {x0}</text>
        <text x={W - pr} y={H - 4} textAnchor="end">day {x0 + n - 1}</text>
        {series.map((s) => (
          <polyline key={s.name} fill="none" stroke={s.color} strokeWidth={1.6} points={s.values.map((v, i) => `${x(i).toFixed(1)},${y(v).toFixed(1)}`).join(" ")} />
        ))}
      </svg>
      <div className="legend">
        {series.map((s) => <span key={s.name}><i style={{ background: s.color }} />{s.name}</span>)}
      </div>
    </>
  );
}

export function Observatory() {
  const app = useApp();
  const ov = useData<any>("overview", { branch: app.branch }, [app.branch, app.rev]);
  const hist = useData<any[]>("market.history", { branch: app.branch }, [app.branch, app.rev]);
  const news = useData<any[]>("events.list", { branch: app.branch, filter: "notable", limit: 14 }, [app.branch, app.rev]);
  const o = ov.data;
  if (!o) return <p className="dim">{ov.error ?? "Loading…"}</p>;
  const h = hist.data ?? [];
  const gm = app.role !== "player";
  const advance = (days: number) => app.run(() => call("sim.advance", { branch: app.branch, days }));
  const nav = o.navigable_permille;
  const grainPct = Math.round((o.prices.grain / o.base_prices.grain) * 100);

  return (
    <>
      <div className="row" style={{ marginBottom: 12 }}>
        <h1 style={{ margin: 0 }}>Observatory — {o.town}</h1>
        <span className="tag">{o.season}</span>
        <span className="dim">{o.weather.desc}</span>
        <span style={{ flex: 1 }} />
        {gm && (
          <>
            <span className="faint">Advance time:</span>
            {[[1, "1 day"], [7, "1 week"], [30, "1 month"], [360, "1 year"]].map(([d, l]) => (
              <button key={d} onClick={() => advance(d as number)}>{l}</button>
            ))}
          </>
        )}
      </div>
      <div className="grid g4" style={{ marginBottom: 14 }}>
        <div className="tile"><div className="k">Residents</div><div className="v">{o.population}</div><div className="s">{o.households} households · {o.hinterland_population} in the hinterland (cohort)</div></div>
        <div className="tile"><div className="k">Hunger</div><div className="v" style={{ color: o.avg_hunger > 30 ? "var(--bad)" : o.avg_hunger > 12 ? "var(--warn)" : undefined }}>{o.avg_hunger}<span className="s"> /100</span></div><div className="s">{o.avg_hunger > 30 ? "famine conditions" : o.avg_hunger > 12 ? "households going short" : "the town is fed"}</div></div>
        <div className="tile"><div className="k">Grain price</div><div className="v">{price(o.prices.grain)}</div><div className="s">{grainPct}% of normal · downstream {price(o.foreign_prices.grain)}</div></div>
        <div className="tile"><div className="k">River</div><div className="v">{nav === 0 ? "Closed" : nav < 1000 ? "Restricted" : "Open"}</div><div className="s">level {o.weather.river}/100 · {o.weather.temp}° · rain {o.weather.rain}</div></div>
      </div>
      <div className="grid g32">
        <div className="card">
          <h2>Prices relative to normal (%)</h2>
          <LineChart x0={h[0]?.day ?? 0} series={Object.keys(COLORS).map((g) => ({ name: g, color: COLORS[g], values: h.map((d) => (d.prices[g] / o.base_prices[g]) * 100) }))} format={(v) => Math.round(v) + "%"} />
          <h2 style={{ marginTop: 16 }}>Hunger, population and crime</h2>
          <LineChart height={150} x0={h[0]?.day ?? 0} series={[{ name: "avg hunger", color: "#e0685a", values: h.map((d) => d.hunger) }, { name: "population ÷ 5", color: "#9cc4e4", values: h.map((d) => d.population / 5) }, { name: "crimes that day ×10", color: "#c3acec", values: h.map((d) => d.crimes * 10) }]} />
          <p className="faint" style={{ marginBottom: 0 }}>Measured from the simulation ledger each day. Model status: simplified synthetic economics, conserved to the milli-unit, not calibrated to any real economy.</p>
        </div>
        <div>
          <div className="card">
            <h2>Latest</h2>
            <ul className="tl">
              {(news.data ?? []).map((e) => (
                <li key={e.event_id} onClick={() => app.go("chronicle", e.event_id)}><span className="d">{e.date}</span><span>{e.privacy !== "public" && <span className="tag gm">GM</span>}{e.summary}</span></li>
              ))}
              {!news.data?.length && <li><span className="d" /><span className="dim">Nothing notable yet.</span></li>}
            </ul>
          </div>
          <div className="card">
            <h2>Minds by importance tier</h2>
            <dl className="kv">
              <dt>T0 cohort</dt><dd>{o.tiers.t0_cohort} hinterland villagers — aggregate accounts, no individuals</dd>
              <dt>T1 residents</dt><dd>{o.tiers.t1} — routines and needs</dd>
              <dt>T2 recurring</dt><dd>{o.tiers.t2} — stakes in current events</dd>
              <dt>T3 major</dt><dd>{o.tiers.t3} — leaders, investigators, nemeses</dd>
              <dt>T4 world-shaping</dt><dd>{o.tiers.t4} — custom agent profile</dd>
            </dl>
            <p className="faint" style={{ marginBottom: 0 }}>Tier is a compute budget. People are promoted automatically as they matter more.</p>
          </div>
        </div>
      </div>
      <div className="grid g3">
        {o.orgs.map((g: any) => (
          <div className="card" key={g.id}>
            <h2>{g.name}</h2>
            <div className="dim" style={{ marginBottom: 6 }}>{g.kind} · led by <a onClick={() => app.go("people", g.leader_id)}>{g.leader ?? "no one"}</a> · {g.members} sworn</div>
            <p className="faint" style={{ fontStyle: "italic" }}>{g.charter}</p>
            {gm && <dl className="kv"><dt>Treasury</dt><dd>{marks(g.coin)} marks</dd><dt>Grain in store</dt><dd>{marks(g.grain)} measures</dd></dl>}
            <div style={{ marginTop: 6 }}>
              {Object.entries(g.policy).filter(([, v]) => v != null).map(([k, v]) => <span className={"tag " + ((k === "hoarding" || k === "strike" || k === "edict_antihoarding") && v ? "warn" : "")} key={k}>{k.replace(/_/g, " ")}: {String(v)}</span>)}
              {g.unpaid_days > 0 && <span className="tag bad">wages unpaid {g.unpaid_days}d</span>}
            </div>
          </div>
        ))}
      </div>
      <div className="grid g2">
        <div className="card">
          <h2>Potential field (magic law {o.magic.law_version})</h2>
          <div className="bar"><i style={{ width: `${(o.magic.ambient / o.magic.capacity_total) * 100}%` }} /></div>
          <p className="dim" style={{ marginBottom: 0 }}>{marks(o.magic.ambient)} thaums ambient + {marks(o.magic.dissipated)} dissipated = {marks(o.magic.capacity_total)} total (conserved). {o.magic.spells_cast} workings cast. Cost(level) = 4·level².</p>
        </div>
        <div className="card">
          <h2>Ledger integrity</h2>
          <p className="dim" style={{ marginBottom: 0 }}>
            Conservation residual per commodity (must be 0):{" "}
            {Object.entries(o.conservation_residual).map(([k, v]) => <span key={k} className={"tag " + (v === 0 ? "good" : "bad")}>{k} {String(v)}</span>)}
          </p>
        </div>
      </div>
    </>
  );
}

export function Chronicle() {
  const app = useApp();
  const [filter, setFilter] = useState("notable");
  const [sel, setSel] = useState<string | null>(app.focus);
  const ev = useData<any[]>("events.list", { branch: app.branch, filter, limit: 250 }, [app.branch, app.rev, filter]);
  const why = useData<any>(sel && app.role !== "player" ? "event.why" : null, { event_id: sel }, [sel]);
  return (
    <>
      <div className="row" style={{ marginBottom: 12 }}>
        <h1 style={{ margin: 0 }}>Chronicle</h1>
        <span style={{ flex: 1 }} />
        {["notable", "crime", "social", "politics", "play", "commands", "all"].map((f) => (
          <button key={f} className={filter === f ? "primary" : ""} onClick={() => setFilter(f)}>{f}</button>
        ))}
      </div>
      <div className="grid g32">
        <div className="card">
          <ul className="tl" style={{ maxHeight: "72vh", overflowY: "auto" }}>
            {(ev.data ?? []).map((e) => (
              <li key={e.event_id} onClick={() => setSel(e.event_id)} style={sel === e.event_id ? { background: "#2f281f" } : undefined}>
                <span className="d">{e.date}{e.inherited && <div className="faint">inherited</div>}</span>
                <span>
                  {e.privacy !== "public" && <span className="tag gm">GM</span>}
                  {e.origin === "ai_proposal" && <span className="tag acc">AI-voiced</span>}
                  {e.origin === "merge" && <span className="tag">merged</span>}
                  {e.summary}
                  <div className="faint mono">{e.event_type}</div>
                </span>
              </li>
            ))}
            {!ev.data?.length && <li><span className="d" /><span className="dim">No events match.</span></li>}
          </ul>
        </div>
        <div className="card">
          <h2>Why did this happen?</h2>
          {app.role === "player" ? <p className="dim">Causal provenance is a GM view.</p> : !sel ? <p className="dim">Select an event to see its recorded causes and consequences.</p> : !why.data ? <p className="dim">{why.error ?? "Loading…"}</p> : (
            <>
              <p><b>{why.data.event.date}</b> — {why.data.event.summary}</p>
              <dl className="kv">
                <dt>Event</dt><dd className="mono">{why.data.record.event_id} · {why.data.record.event_type} v{why.data.record.event_version}</dd>
                <dt>Origin</dt><dd>{why.data.record.origin} · by {why.data.record.actor.principal} ({why.data.record.actor.role})</dd>
                <dt>Visibility</dt><dd>{why.data.record.privacy}</dd>
                <dt>Record hash</dt><dd className="mono">{why.data.record.record_hash.slice(0, 20)}…</dd>
                <dt>State after</dt><dd className="mono">{why.data.record.state_hash.slice(0, 20)}…</dd>
              </dl>
              <h3 style={{ marginTop: 12 }}>Caused by</h3>
              {why.data.caused_by.length ? why.data.caused_by.map((c: any) => <div key={c.event_id}><a onClick={() => setSel(c.event_id)}>{c.date}</a> — {c.summary} <span className="faint mono">{c.event_type}</span></div>) : <p className="dim">A root command: nothing upstream is recorded.</p>}
              <h3 style={{ marginTop: 12 }}>Led to</h3>
              {why.data.led_to.length ? why.data.led_to.slice(0, 25).map((c: any) => <div key={c.event_id}><a onClick={() => setSel(c.event_id)}>{c.date}</a> — {c.summary}</div>) : <p className="dim">No recorded consequences.</p>}
              <p className="faint">{why.data.note}</p>
              <details><summary className="dim">Payload</summary><pre className="log">{JSON.stringify(why.data.record.payload, null, 2)}</pre></details>
            </>
          )}
        </div>
      </div>
    </>
  );
}

export function Bible() {
  const app = useApp();
  const b = useData<any>("bible.get", { branch: app.branch }, [app.branch, app.rev]);
  if (!b.data) return <p className="dim">{b.error ?? "Loading…"}</p>;
  return (
    <>
      <h1>World Bible</h1>
      <div className="note">Structured lore for this world. Entries marked <i>authored fixture</i> were written for the first vertical slice; the simulation produces everything in the Chronicle. Facts recorded by the GM appear at the end.</div>
      <div className="grid g2">
        {Object.entries(b.data.sections).map(([sec, entries]: [string, any]) => (
          <div className="card" key={sec}>
            <h2>{sec.replace(/^\d+ /, "")}</h2>
            <div className="prose">
              {entries.map((e: any) => (
                <p key={e.title}><b>{e.title}.</b> {e.body} <span className="tag">{e.provenance.replace("_", " ")}</span></p>
              ))}
            </div>
          </div>
        ))}
      </div>
      <div className="card">
        <h2>Recorded facts</h2>
        {b.data.facts.length ? b.data.facts.map((f: any) => <p key={f.id}>{f.protected && <span className="tag acc">protected</span>}{f.gm_only && <span className="tag gm">GM</span>}{f.text} <span className="faint mono">{f.id}</span></p>) : <p className="dim">None yet. Record facts in GM Tools.</p>}
      </div>
    </>
  );
}
