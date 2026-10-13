import { MODES } from "../lib/mappings";

export function ModeControl({ value, disabled, onChange }: { value?: number; disabled: boolean; onChange: (m: number) => void }) {
  return (
    <div className="grid grid-cols-5 gap-1 p-1 rounded-xl bg-black/25" role="radiogroup" aria-label="Modo">
      {MODES.map(({ label, icon: Icon, color }, i) => {
        const active = value === i;
        return (
          <button
            key={label}
            role="radio"
            aria-checked={active}
            disabled={disabled}
            onClick={() => onChange(i)}
            className={`flex flex-col items-center gap-1 py-2 rounded-lg text-[12px] transition-colors disabled:opacity-40 ${
              active ? "bg-white/10 text-white" : "text-white/55 hover:text-white/85 hover:bg-white/5"
            }`}
          >
            <Icon size={18} color={active ? color : "currentColor"} />
            {label}
          </button>
        );
      })}
    </div>
  );
}
