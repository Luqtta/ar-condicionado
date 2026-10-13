import { invoke } from "@tauri-apps/api/core";
import { AirVent, ArrowLeft, LoaderCircle, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import type { Found, Info } from "../lib/mappings";

// First run (and "Procurar aparelhos"): scan the LAN, bind the chosen unit.
export function Setup({ onDone, onCancel }: { onDone: (info: Info) => void; onCancel?: () => void }) {
  const [found, setFound] = useState<Found[] | null>(null);
  const [binding, setBinding] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const scan = useCallback(() => {
    setFound(null);
    setError(null);
    invoke<Found[]>("scan")
      .then(setFound)
      .catch((e) => {
        setFound([]);
        setError(String(e));
      });
  }, []);

  useEffect(scan, [scan]);

  const bind = async (f: Found) => {
    setBinding(f.mac);
    setError(null);
    try {
      onDone(await invoke<Info>("bind", { found: f }));
    } catch (e) {
      setError(`Falha ao parear: ${e}`);
    } finally {
      setBinding(null);
    }
  };

  return (
    <div className="flex flex-col h-full p-5 gap-4">
      <div className="flex items-center gap-2">
        {onCancel && (
          <button onClick={onCancel} className="size-8 grid place-items-center rounded-full hover:bg-white/10" aria-label="Voltar">
            <ArrowLeft size={18} />
          </button>
        )}
        <h1 className="text-[17px] font-semibold">Procurar aparelhos</h1>
      </div>
      <p className="text-[13px] text-white/55">
        O computador precisa estar na mesma rede Wi-Fi do ar-condicionado. A comunicação é só local, sem nuvem.
      </p>

      <div className="flex-1 overflow-y-auto flex flex-col gap-2">
        {found === null && (
          <div className="flex-1 flex flex-col items-center justify-center gap-3 text-white/55 text-[13px]">
            <LoaderCircle className="animate-spin text-accent" size={28} />
            Procurando na rede local...
          </div>
        )}
        {found?.length === 0 && !error && (
          <div className="flex-1 grid place-items-center text-center text-white/55 text-[13px]">Nenhum aparelho encontrado.</div>
        )}
        {found?.map((f) => (
          <button
            key={f.mac}
            disabled={binding !== null}
            onClick={() => bind(f)}
            className="flex items-center gap-3 p-3 rounded-xl bg-white/6 hover:bg-white/10 text-left disabled:opacity-60 transition-colors"
          >
            <AirVent size={22} className="text-accent shrink-0" />
            <div className="flex-1 min-w-0">
              <div className="text-[14px] truncate">{f.name}</div>
              <div className="text-[12px] text-white/45 truncate">
                {f.ip} · {f.mac}
                {f.model && f.model !== f.brand ? ` · ${f.model}` : ""}
              </div>
            </div>
            {binding === f.mac && <LoaderCircle size={18} className="animate-spin text-white/60" />}
          </button>
        ))}
      </div>

      {error && <p className="text-[12px] text-red-400 break-words">{error}</p>}
      <button
        onClick={scan}
        disabled={found === null || binding !== null}
        className="h-10 flex items-center justify-center gap-2 rounded-lg bg-accent hover:brightness-110 text-[14px] disabled:opacity-50"
      >
        <RefreshCw size={16} />
        Procurar novamente
      </button>
    </div>
  );
}
