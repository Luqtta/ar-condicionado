import { Check, ChevronDown } from "lucide-react";
import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";

// In-page listbox instead of <select>: WebView2's native select popup takes
// activation from the flyout, which then closes on blur.
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
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const id = useId();

  useEffect(() => {
    if (!open) return;
    const items = list.current!.querySelectorAll<HTMLButtonElement>("[role=option]");
    items[value ?? 0]?.focus();
    const outside = (e: PointerEvent) => !root.current?.contains(e.target as Node) && setOpen(false);
    document.addEventListener("pointerdown", outside);
    return () => document.removeEventListener("pointerdown", outside);
  }, [open, value]);

  const close = () => {
    setOpen(false);
    root.current?.querySelector<HTMLButtonElement>("[aria-haspopup]")?.focus();
  };

  const onKey = (e: KeyboardEvent) => {
    const items = [...list.current!.querySelectorAll<HTMLButtonElement>("[role=option]")];
    const i = items.indexOf(document.activeElement as HTMLButtonElement);
    if (e.key === "Escape") close();
    else if (e.key === "Tab") setOpen(false);
    else if (e.key === "ArrowDown") items[Math.min(i + 1, items.length - 1)]?.focus();
    else if (e.key === "ArrowUp") items[Math.max(i - 1, 0)]?.focus();
    else return;
    e.preventDefault();
  };

  return (
    <div ref={root} className="relative flex flex-col gap-1">
      <span id={id} className="text-[12px] text-white/50">
        {label}
      </span>
      <button
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-labelledby={id}
        disabled={disabled}
        onClick={() => setOpen(!open)}
        className="h-9 flex items-center rounded-lg bg-white/6 hover:bg-white/9 pl-3 pr-2.5 text-[13px] text-left outline-none focus-visible:ring-2 focus-visible:ring-accent disabled:opacity-40"
      >
        <span className="flex-1 truncate">{value === undefined ? "--" : options[value]}</span>
        <ChevronDown size={16} className={`text-white/50 transition-transform ${open ? "rotate-180" : ""}`} />
      </button>
      {open && (
        // opens upward: the dropdowns sit low in the 600px flyout
        <div
          ref={list}
          role="listbox"
          aria-labelledby={id}
          onKeyDown={onKey}
          className="absolute bottom-full mb-1 left-0 right-0 z-10 p-1 rounded-lg bg-[#2a2e35] border border-white/10 shadow-xl"
        >
          {options.map((o, i) => (
            <button
              key={o}
              role="option"
              aria-selected={i === value}
              onClick={() => {
                onChange(i);
                close();
              }}
              className="w-full h-8 flex items-center gap-2 px-2.5 rounded-md text-[13px] text-left outline-none hover:bg-white/8 focus-visible:bg-white/8"
            >
              <span className="flex-1">{o}</span>
              {i === value && <Check size={14} className="text-accent" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
