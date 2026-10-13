import { Minus, Plus } from "lucide-react";
import { TEMP_MAX, TEMP_MIN } from "../lib/mappings";

const btn =
  "size-11 grid place-items-center rounded-full bg-white/6 hover:bg-white/10 active:bg-white/14 disabled:opacity-40 transition-colors";

export function TempControl({ value, disabled, onChange }: { value?: number; disabled: boolean; onChange: (t: number) => void }) {
  const t = value ?? TEMP_MIN;
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <button className={btn} disabled={disabled || t <= TEMP_MIN} onClick={() => onChange(t - 1)} aria-label="Diminuir">
          <Minus size={20} />
        </button>
        <div className="flex items-start font-light tabular-nums">
          <span className="text-[64px] leading-none tracking-tight">{value ?? "--"}</span>
          <span className="text-xl text-white/50 mt-1.5">°C</span>
        </div>
        <button className={btn} disabled={disabled || t >= TEMP_MAX} onClick={() => onChange(t + 1)} aria-label="Aumentar">
          <Plus size={20} />
        </button>
      </div>
      <input
        type="range"
        min={TEMP_MIN}
        max={TEMP_MAX}
        step={1}
        value={t}
        disabled={disabled}
        onChange={(e) => onChange(+e.target.value)}
        className="w-full accent-accent disabled:opacity-40"
        aria-label="Temperatura alvo"
      />
      <div className="flex justify-between text-[11px] text-white/40 -mt-2">
        <span>{TEMP_MIN}°</span>
        <span>{TEMP_MAX}°</span>
      </div>
    </div>
  );
}
