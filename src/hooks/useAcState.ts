import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { tempProps, TEMP_MAX, TEMP_MIN, type Info, type Props, type Snapshot } from "../lib/mappings";

const DEBOUNCE_MS = 400;
// Polls landing right after a local change would flip the UI back; ignore them for a bit.
const SETTLE_MS = 1500;

const pick = (p: Props, keys: string[]) => Object.fromEntries(keys.map((k) => [k, p[k]]));

export function useAcState() {
  // undefined = still loading, null = no device configured
  const [info, setInfo] = useState<Info | null | undefined>(undefined);
  const [props, setProps] = useState<Props | null>(null);
  const [room, setRoom] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  const propsRef = useRef<Props>({});
  propsRef.current = props ?? {};
  const busy = useRef(0);
  const settleUntil = useRef(0);
  const tempTimer = useRef<number | undefined>(undefined);
  const tempBefore = useRef<number | undefined>(undefined);

  const applySnapshot = useCallback((s: Snapshot) => {
    setRoom(s.room);
    setError(null);
    if (busy.current === 0 && Date.now() >= settleUntil.current) setProps(s.props);
  }, []);

  useEffect(() => {
    invoke<Info | null>("get_info").then((i) => {
      setInfo(i);
      if (!i) invoke("show"); // first run: open the setup screen
    });
    invoke<Snapshot | null>("get_snapshot").then((s) => s && applySnapshot(s));
    const off = [
      listen<Snapshot>("ac-status", (e) => applySnapshot(e.payload)),
      listen<string>("ac-error", (e) => setError(e.payload)),
      listen<Info | null>("ac-info", (e) => setInfo(e.payload)),
    ];
    return () => off.forEach((p) => p.then((f) => f()));
  }, [applySnapshot]);

  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), 3000);
    return () => clearTimeout(t);
  }, [toast]);

  /** Optimistic: shows `change` now, reverts to `prev` (default: current values) if the unit rejects it. */
  const send = useCallback(async (change: Props, prev?: Props) => {
    const keys = Object.keys(change);
    const before = prev ?? pick(propsRef.current, keys);
    setProps((p) => ({ ...p, ...change }));
    busy.current++;
    try {
      await invoke("send", { props: Object.entries(change) });
    } catch {
      setProps((p) => ({ ...p, ...before }));
      setToast("Não foi possível enviar o comando");
    } finally {
      busy.current--;
      settleUntil.current = Date.now() + SETTLE_MS;
    }
  }, []);

  /** Debounced: rapid −/+ clicks and slider drags end up as a single cmd. */
  const setTemp = useCallback(
    (t: number) => {
      t = Math.min(TEMP_MAX, Math.max(TEMP_MIN, Math.round(t)));
      if (tempTimer.current === undefined) tempBefore.current = propsRef.current.SetTem;
      setProps((p) => ({ ...p, SetTem: t }));
      settleUntil.current = Date.now() + DEBOUNCE_MS + SETTLE_MS;
      clearTimeout(tempTimer.current);
      tempTimer.current = window.setTimeout(() => {
        tempTimer.current = undefined;
        send(tempProps(t), { SetTem: tempBefore.current!, TemUn: propsRef.current.TemUn });
      }, DEBOUNCE_MS);
    },
    [send],
  );

  const refresh = useCallback(() => invoke<Snapshot>("refresh").catch(() => {}), []);

  return { info, setInfo, props, room, error, toast, send, setTemp, refresh };
}
