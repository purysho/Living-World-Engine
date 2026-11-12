import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import { call, getRole, Role, setRole } from "./api";
import { Library, Campaigns, MergeLab } from "./views/Worlds";
import { Observatory, Chronicle, Bible } from "./views/Observatory";
import { People, Factions, Cases } from "./views/Society";
import { Play, TownMap } from "./views/Play";
import { Council, AiAgents, GmTools } from "./views/Tools";

export interface Ctx {
  worldId: string | null;
  branch: string | null;
  branches: any[];
  rev: number;
  role: Role;
  /** Run a mutating call with busy indicator and error banner; refreshes views on success. */
  run: <T>(fn: () => Promise<T>) => Promise<T | undefined>;
  bump: () => void;
  select: (worldId: string | null, branch?: string | null) => void;
  go: (view: string, focus?: string) => void;
  focus: string | null;
}
const AppCtx = createContext<Ctx>(null as unknown as Ctx);
export const useApp = () => useContext(AppCtx);

const NAV: [string, [string, string, boolean?][]][] = [
  ["Worlds", [["library", "World Library"], ["campaigns", "Campaigns & Branches"], ["merge", "Merge Lab", true]]],
  ["Play", [["play", "Play"], ["map", "Town Map"]]],
  ["Observe", [["observatory", "Observatory"], ["chronicle", "Chronicle"], ["people", "People"], ["factions", "Factions & Nemeses", true], ["cases", "Cases & Evidence"], ["bible", "World Bible"]]],
  ["Create", [["gm", "GM Tools", true], ["council", "Writers' Brains", true], ["ai", "AI & Agents"]]],
];

export default function App() {
  const [worlds, setWorlds] = useState<any[]>([]);
  const [worldId, setWorldId] = useState<string | null>(null);
  const [branches, setBranches] = useState<any[]>([]);
  const [branch, setBranch] = useState<string | null>(null);
  const [view, setView] = useState("library");
  const [focus, setFocus] = useState<string | null>(null);
  const [rev, setRev] = useState(0);
  const [role, setRoleState] = useState<Role>(getRole());
  const [busy, setBusy] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<any>(null);

  const bump = useCallback(() => setRev((r) => r + 1), []);
  const run = useCallback(async function <T>(fn: () => Promise<T>): Promise<T | undefined> {
    setBusy((b) => b + 1);
    setError(null);
    try {
      const out = await fn();
      setRev((r) => r + 1);
      return out;
    } catch (e: any) {
      setError(`${e.code ? e.code + ": " : ""}${e.message}`);
      return undefined;
    } finally {
      setBusy((b) => b - 1);
    }
  }, []);

  useEffect(() => {
    call("app.info").then(setInfo).catch((e) => setError(e.message));
  }, []);
  useEffect(() => {
    call<any[]>("world.list")
      .then((w) => {
        setWorlds(w);
        if (w.length && !w.find((x) => x.world_id === worldId)) {
          setWorldId(w[w.length - 1].world_id);
          setBranch(null);
        }
        if (!w.length) setWorldId(null);
      })
      .catch((e) => setError(e.message));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rev]);
  useEffect(() => {
    if (!worldId) {
      setBranches([]);
      return;
    }
    call<any[]>("branch.list", { world_id: worldId })
      .then((b) => {
        setBranches(b);
        if (!b.find((x) => x.branch_id === branch)) setBranch((b.find((x) => x.is_canon) ?? b[0])?.branch_id ?? null);
      })
      .catch(() => setBranches([]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [worldId, rev]);

  const ctx: Ctx = useMemo(
    () => ({
      worldId, branch, branches, rev, role, run, bump, focus,
      select: (w, b) => {
        setWorldId(w);
        setBranch(b ?? null);
      },
      go: (v, f) => {
        setView(v);
        setFocus(f ?? null);
      },
    }),
    [worldId, branch, branches, rev, role, run, bump, focus],
  );

  const cur = branches.find((b) => b.branch_id === branch);
  const needsWorld = !["library", "ai"].includes(view);
  const gmOnly = NAV.flatMap((g) => g[1]).find((n) => n[0] === view)?.[2];

  return (
    <AppCtx.Provider value={ctx}>
      <div className="app">
        <aside className="side">
          <div className="brand">
            <b>Living World Engine</b>
            <span>v{info?.app_version ?? "…"} · engine {info?.engine ?? ""}</span>
          </div>
          <label htmlFor="sel-world">World</label>
          <select id="sel-world" value={worldId ?? ""} onChange={(e) => ctx.select(e.target.value || null)}>
            {!worlds.length && <option value="">(none yet)</option>}
            {worlds.map((w) => (
              <option key={w.world_id} value={w.world_id}>{w.name}</option>
            ))}
          </select>
          <label htmlFor="sel-branch">Timeline</label>
          <select id="sel-branch" value={branch ?? ""} onChange={(e) => setBranch(e.target.value)}>
            {branches.map((b) => (
              <option key={b.branch_id} value={b.branch_id}>
                {b.is_canon ? "★ " : b.kind === "campaign" ? "⚔ " : b.kind === "merge" ? "⇄ " : "⑂ "}
                {b.name}
              </option>
            ))}
          </select>
          <label htmlFor="sel-role">Viewing as</label>
          <select
            id="sel-role"
            value={role}
            onChange={(e) => {
              setRole(e.target.value as Role);
              setRoleState(e.target.value as Role);
              bump();
            }}
          >
            <option value="world_owner">World Owner (all truth)</option>
            <option value="gm">Game Master</option>
            <option value="player">Player (what the party knows)</option>
          </select>
          <nav aria-label="Workspaces">
            {NAV.map(([group, items]) => (
              <div key={group}>
                <div className="group">{group}</div>
                {items.map(([id, label]) => (
                  <button key={id} aria-current={view === id ? "page" : undefined} onClick={() => ctx.go(id)}>
                    {label}
                  </button>
                ))}
              </div>
            ))}
          </nav>
          <div className="sidefoot">
            AI: {info?.ai?.provider === "anthropic" ? "Claude (online)" : "offline"} · local-first
            <br />
            Simulation is authoritative; AI only proposes.
          </div>
        </aside>
        <main>
          {error && (
            <div className="err" role="alert">
              {error}{" "}
              <button className="small" onClick={() => setError(null)}>dismiss</button>
            </div>
          )}
          {cur && needsWorld && (
            <div className="top">
              <span className="date">{cur.date}</span>
              <span className="dim">
                {cur.name} {cur.is_canon && <span className="tag acc">canon</span>}
                {cur.campaign && <span className="tag">{cur.campaign.rules}</span>}
                {cur.campaign && <span className="tag">{cur.campaign.gm_mode} GM</span>}
              </span>
              <span className="sp" />
              <span className="faint">head #{cur.head_seq} · pop. {cur.population}</span>
            </div>
          )}
          {needsWorld && !branch ? (
            <div className="card">
              <h2>No world yet</h2>
              <p>Create a world in the World Library to begin.</p>
              <button className="primary" onClick={() => ctx.go("library")}>Open World Library</button>
            </div>
          ) : gmOnly && role === "player" ? (
            <div className="card">
              <h2>GM-only workspace</h2>
              <p className="dim">You are viewing as a player. This workspace shows objective truth and is hidden. The filter is enforced by the engine, not just this screen.</p>
            </div>
          ) : (
            <>
              {view === "library" && <Library worlds={worlds} />}
              {view === "campaigns" && <Campaigns />}
              {view === "merge" && <MergeLab />}
              {view === "observatory" && <Observatory />}
              {view === "chronicle" && <Chronicle />}
              {view === "bible" && <Bible />}
              {view === "people" && <People />}
              {view === "factions" && <Factions />}
              {view === "cases" && <Cases />}
              {view === "play" && <Play />}
              {view === "map" && <TownMap />}
              {view === "council" && <Council />}
              {view === "ai" && <AiAgents onChange={() => call("app.info").then(setInfo)} />}
              {view === "gm" && <GmTools />}
            </>
          )}
          {busy > 0 && <div className="busy" role="status">Working…</div>}
        </main>
      </div>
    </AppCtx.Provider>
  );
}
