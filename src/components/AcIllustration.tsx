// Wall split unit with an animated airflow; color follows the mode, stops when off.
export function AcIllustration({ on, color, fan, light }: { on: boolean; color: string; fan: number; light: boolean }) {
  // faster flow on higher fan speeds (Auto counts as medium)
  const duration = `${1.8 - (fan || 3) * 0.2}s`;
  return (
    <svg viewBox="0 0 360 120" className="w-full h-[104px]" aria-hidden>
      <defs>
        <linearGradient id="body" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#3a3f48" />
          <stop offset="1" stopColor="#2a2e35" />
        </linearGradient>
      </defs>
      <rect x="40" y="8" width="280" height="54" rx="14" fill="url(#body)" stroke="#ffffff14" />
      <rect x="56" y="48" width="248" height="6" rx="3" fill="#15171b" />
      <circle cx="292" cy="24" r="3" fill={on && light ? color : "#4b515c"} />
      <g
        fill="none"
        stroke={color}
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeDasharray="10 30"
        style={{ opacity: on ? 0.9 : 0, transition: "opacity .4s, stroke .4s" }}
      >
        {[80, 130, 180, 230, 280].map((x, i) => (
          <path
            key={x}
            d={`M${x} 64 q -8 14 0 26 q 8 12 0 26`}
            style={{ animation: on ? `airflow ${duration} linear ${-i * 0.23}s infinite` : "none" }}
          />
        ))}
      </g>
    </svg>
  );
}
