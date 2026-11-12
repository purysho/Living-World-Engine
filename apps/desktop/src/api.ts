// One gateway, two transports: Tauri IPC inside the desktop app, HTTP when served by lwe-server.
// Both end in the same Rust `App::dispatch`, so behaviour is identical.
import { useCallback, useEffect, useState } from "react";

export type Role = "world_owner" | "gm" | "player";
export class ApiError extends Error {
  constructor(public code: string, message: string) {
    super(message);
  }
}

let currentRole: Role = "world_owner";
export function setRole(r: Role) {
  currentRole = r;
}
export function getRole(): Role {
  return currentRole;
}

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export async function call<T = any>(op: string, args: Record<string, unknown> = {}): Promise<T> {
  const request = { op, args, principal: { id: "local_" + currentRole, role: currentRole } };
  let res: any;
  if (window.__TAURI_INTERNALS__) {
    const { invoke } = await import("@tauri-apps/api/core");
    res = await invoke("lwe_dispatch", { request });
  } else {
    const r = await fetch("/api", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(request) });
    res = await r.json();
  }
  if (!res.ok) throw new ApiError(res.error.code, res.error.message);
  return res.data as T;
}

/** Load data for a view; reloads when `deps` change. */
export function useData<T = any>(op: string | null, args: Record<string, unknown>, deps: unknown[]) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const reload = useCallback(() => {
    if (!op) return;
    setLoading(true);
    call<T>(op, args)
      .then((d) => {
        setData(d);
        setError(null);
      })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);
  useEffect(reload, [reload]);
  return { data, error, loading, reload };
}

export const marks = (milli: number | null | undefined) => (milli == null ? "—" : (milli / 1000).toLocaleString(undefined, { maximumFractionDigits: 0 }));
export const price = (milli: number) => (milli / 1000).toFixed(2);
