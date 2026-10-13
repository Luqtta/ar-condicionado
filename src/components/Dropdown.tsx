import { ChevronDown } from "lucide-react";

// Native <select>: keyboard, a11y and the dark popup list come for free.
export function Dropdown({
  label,
  options,
  value,
  disabled,
  onChange,
}: {
  label: string;
  options: string[];
  value?: number;
  disabled: boolean;
  onChange: (i: number) => void;
}) {
  return (
    <label className="flex flex-col gap-1 text-[12px] text-white/50">
      {label}
      <span className="relative">
        <select
          value={value ?? ""}
          disabled={disabled}
          onChange={(e) => onChange(+e.target.value)}
          className="w-full h-9 appearance-none rounded-lg bg-white/6 hover:bg-white/9 pl-3 pr-8 text-[13px] text-white outline-none focus-visible:ring-2 focus-visible:ring-accent disabled:opacity-40"
        >
          {value === undefined && <option value="" disabled />}
          {options.map((o, i) => (
            <option key={o} value={i} className="bg-panel">
              {o}
            </option>
          ))}
        </select>
        <ChevronDown size={16} className="absolute right-2.5 top-1/2 -translate-y-1/2 pointer-events-none text-white/50" />
      </span>
    </label>
  );
}
