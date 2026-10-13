import { listen } from "@tauri-apps/api/event";
import { Power, Settings as SettingsIcon, Thermometer } from "lucide-react";
import { useEffect, useState } from "react";
import { AcIllustration } from "./components/AcIllustration";
import { Dropdown } from "./components/Dropdown";
import { ModeControl } from "./components/ModeControl";
import { Settings } from "./components/Settings";
import { Setup } from "./components/Setup";
import { TempControl } from "./components/TempControl";
import { Toggles } from "./components/Toggles";
import { useAcState } from "./hooks/useAcState";
import { DIRECTIONS, directionOf, directionProps, FAN_SPEEDS, MODES } from "./lib/mappings";

export default function App() {
  const ac = useAcState();
  const [view, setView] = useState<"main" | "setup" | "settings">("main");

  useEffect(() => {
    const off = listen("ac-scan", () => setView("setup"));
    return () => void off.then((f) => f());
  }, []);

  return (
    <div className="h-full rounded-[12px] bg-panel border border-white/10 text-white overflow-hidden relative">
      {ac.info === undefined ? null : ac.info === null || view === "setup" ? (
        <Setup
          onDone={(info) => {
            ac.setInfo(info);
            setView("main");
          }}
          onCancel={ac.info ? () => setView("main") : undefined}
        />
      ) : view === "settings" ? (
        <Settings
          info={ac.info}
          onBack={() => setView("main")}
          onScan={() => setView("setup")}
          onRename={(name) => ac.setInfo({ ...ac.info!, name })}
        />
      ) : (
        <Main ac={ac} onSettings={() => setView("settings")} />
      )}
      {ac.toast && (
        <div role="status" className="absolute bottom-14 left-1/2 -translate-x-1/2 px-3 py-1.5 rounded-full bg-[#2c3038] border border-white/10 text-[12px] text-white/85 shadow-lg whitespace-nowrap">
          {ac.toast}
        </div>
      )}
    </div>
  );
}

function Main({ ac, onSettings }: { ac: ReturnType<typeof useAcState>; onSettings: () => void }) {
  const p = ac.props ?? {};
  const disabled = ac.props === null;
  const on = p.Pow === 1;
  const mode = MODES[p.Mod] ?? MODES[1];

  return (
    <div className="flex flex-col h-full">
      <header className="flex items-center gap-3 px-5 pt-4">
        <h1 className="flex-1 text-[16px] font-semibold truncate">{ac.info?.name || "Ar-condicionado"}</h1>
        {ac.room !== null && (
          <span className="flex items-center gap-1 h-7 px-2.5 rounded-full bg-white/6 text-[12.5px] text-white/75 tabular-nums" title="Temperatura ambiente">
            <Thermometer size={14} />
            {ac.room}°C
          </span>
        )}
        <button
          onClick={() => ac.send({ Pow: on ? 0 : 1 })}
          disabled={disabled}
          aria-label={on ? "Desligar" : "Ligar"}
          aria-pressed={on}
          className={`size-10 grid place-items-center rounded-full transition-colors disabled:opacity-40 ${
            on ? "bg-accent text-white shadow-[0_0_16px_#3b8cff66]" : "bg-white/8 text-white/70 hover:bg-white/12"
          }`}
        >
          <Power size={18} />
        </button>
      </header>

      <AcIllustration on={on} color={mode.color} fan={p.WdSpd ?? 0} light={!!p.Lig} />

      <main className={`flex flex-col gap-4 px-5 flex-1 transition-opacity ${on ? "" : "opacity-70"}`}>
        <TempControl value={p.SetTem} disabled={disabled} onChange={ac.setTemp} />
        <ModeControl value={p.Mod} disabled={disabled} onChange={(m) => ac.send({ Mod: m })} />
        <div className="grid grid-cols-2 gap-3">
          <Dropdown label="Vento" options={FAN_SPEEDS} value={p.WdSpd} disabled={disabled} onChange={(v) => ac.send({ WdSpd: v })} />
          <Dropdown
            label="Direção"
            options={DIRECTIONS}
            value={disabled ? undefined : directionOf(p)}
            disabled={disabled}
            onChange={(d) => ac.send(directionProps(d, p))}
          />
        </div>
        <Toggles props={p} disabled={disabled} onChange={(c) => ac.send(c)} />
      </main>

      <footer className="flex items-center gap-2 h-12 px-5 border-t border-white/8 text-[12px] text-white/55">
        <span className={`size-2 rounded-full shrink-0 ${ac.error ? "bg-red-400" : disabled ? "bg-white/30" : "bg-emerald-400"}`} />
        <span className="flex-1 truncate" title={ac.error ?? undefined}>
          {ac.error ? "Sem conexão com o aparelho" : disabled ? "Conectando..." : `Conectado · ${ac.info?.ip}`}
        </span>
        <button onClick={onSettings} className="size-8 grid place-items-center rounded-full hover:bg-white/10" aria-label="Configurações">
          <SettingsIcon size={16} />
        </button>
      </footer>
    </div>
  );
}
