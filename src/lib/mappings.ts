// UI <-> Gree protocol values (see greeclimate/device.py).
import { Droplet, Fan, Lightbulb, Moon, Rocket, Snowflake, Sparkles, Sun, VolumeX, Wind, type LucideIcon } from "lucide-react";

export type Props = Record<string, number>;
export type Snapshot = { props: Props; room: number | null };
export type Info = { name: string; ip: string; mac: string };
export type Found = {
  ip: string;
  mac: string;
  name: string;
  brand: string | null;
  model: string | null;
  ver: string | null;
  version: "v1" | "v2";
};

export const TEMP_MIN = 16;
export const TEMP_MAX = 30;

// SetTem is always Celsius, TemUn 0 keeps the unit's display in °C.
export const tempProps = (t: number): Props => ({ TemUn: 0, SetTem: t });

// Mod 0–4
export const MODES: { label: string; icon: LucideIcon; color: string }[] = [
  { label: "Auto", icon: Sparkles, color: "#a78bfa" },
  { label: "Frio", icon: Snowflake, color: "#3b8cff" },
  { label: "Seco", icon: Droplet, color: "#2dd4bf" },
  { label: "Ventilar", icon: Fan, color: "#94a3b8" },
  { label: "Quente", icon: Sun, color: "#ff8a3d" },
];

// WdSpd 0–5
export const FAN_SPEEDS = ["Auto", "Baixo", "Médio-baixo", "Médio", "Médio-alto", "Alto"];

// Index bits: 1 = vertical swing, 2 = horizontal swing.
export const DIRECTIONS = ["Fixo", "Oscilar vertical", "Oscilar horizontal", "Ambos"];

// SwUpDn: 1 full swing, 2–6 fixed positions, 7–11 partial swing. SwingLfRig: 1 full swing, 2–6 fixed.
const vSwing = (v = 0) => v === 1 || (v >= 7 && v <= 11);
const fixed = (v = 0) => (v >= 2 && v <= 6 ? v : 0);

export const directionOf = (p: Props) => (vSwing(p.SwUpDn) ? 1 : 0) | (p.SwingLfRig === 1 ? 2 : 0);

// Turning a swing off keeps a fixed blade position if the unit had one.
export const directionProps = (d: number, p: Props): Props => ({
  SwUpDn: d & 1 ? 1 : fixed(p.SwUpDn),
  SwingLfRig: d & 2 ? 1 : fixed(p.SwingLfRig),
});

export const TOGGLES: { label: string; icon: LucideIcon; get: (p: Props) => boolean; set: (on: boolean) => Props }[] = [
  // Turbo and Quiet are mutually exclusive: send both in the same cmd.
  { label: "Turbo", icon: Rocket, get: (p) => p.Tur === 1, set: (on): Props => (on ? { Tur: 1, Quiet: 0 } : { Tur: 0 }) },
  { label: "Sono", icon: Moon, get: (p) => !!p.SwhSlp, set: (on) => ({ SwhSlp: +on, SlpMod: +on }) },
  // greeclimate issue 87: quiet on is 2, not 1
  { label: "Silencioso", icon: VolumeX, get: (p) => !!p.Quiet, set: (on): Props => (on ? { Quiet: 2, Tur: 0 } : { Quiet: 0 }) },
  { label: "X-Fan", icon: Wind, get: (p) => !!p.Blo, set: (on) => ({ Blo: +on }) },
  { label: "Luz do display", icon: Lightbulb, get: (p) => !!p.Lig, set: (on) => ({ Lig: +on }) },
];
