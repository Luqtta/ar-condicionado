import { TOGGLES, type Props } from "../lib/mappings";

export function Toggles({ props, disabled, onChange }: { props: Props; disabled: boolean; onChange: (change: Props) => void }) {
  return (
    <div className="grid grid-cols-3 gap-2">
      {TOGGLES.map(({ label, icon: Icon, get, set }) => {
        const on = get(props);
        return (
          <button
            key={label}
            role="switch"
            aria-checked={on}
            disabled={disabled}
            onClick={() => onChange(set(!on))}
            className={`h-10 flex items-center gap-2 px-3 rounded-lg text-[12.5px] whitespace-nowrap transition-colors disabled:opacity-40 ${
              on ? "bg-accent text-white" : "bg-white/6 text-white/75 hover:bg-white/10"
            }`}
          >
            <Icon size={16} className="shrink-0" />
            {label}
          </button>
        );
      })}
    </div>
  );
}
