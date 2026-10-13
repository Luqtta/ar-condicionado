import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, Search } from "lucide-react";
import { useState } from "react";
import type { Info } from "../lib/mappings";

export function Settings({ info, onRename, onBack, onScan }: { info: Info; onRename: (name: string) => void; onBack: () => void; onScan: () => void }) {
  const [name, setName] = useState(info.name);

  const save = () => {
    const n = name.trim();
    if (!n || n === info.name) return setName(info.name);
    invoke("rename", { name: n }).then(() => onRename(n));
  };

  return (
    <div className="flex flex-col h-full p-5 gap-5">
      <div className="flex items-center gap-2">
        <button onClick={onBack} className="size-8 grid place-items-center rounded-full hover:bg-white/10" aria-label="Voltar">
          <ArrowLeft size={18} />
        </button>
        <h1 className="text-[17px] font-semibold">Configurações</h1>
      </div>

      <label className="flex flex-col gap-1.5 text-[12px] text-white/50">
        Nome do aparelho
        <input
          value={name}
          maxLength={40}
          onChange={(e) => setName(e.target.value)}
          onBlur={save}
          onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
          className="h-10 rounded-lg bg-white/6 px-3 text-[14px] text-white outline-none focus:ring-2 focus:ring-accent select-text"
        />
      </label>

      <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-[13px]">
        <dt className="text-white/50">IP</dt>
        <dd className="tabular-nums">{info.ip}</dd>
        <dt className="text-white/50">MAC</dt>
        <dd className="tabular-nums">{info.mac}</dd>
      </dl>

      <button onClick={onScan} className="h-10 flex items-center justify-center gap-2 rounded-lg bg-white/8 hover:bg-white/12 text-[14px]">
        <Search size={16} />
        Procurar aparelhos
      </button>
    </div>
  );
}
