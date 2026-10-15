import { useId } from "react";

import { cn } from "../lib/utils";

// Geometry in the 64x64 viewBox, shared with public/favicon.svg.
const V_PATH = "M15.5 17.5h10.2L32 36.5l6.3-19h10.2L37 47.5H27z";
const ORB_CENTER = 32;
const ORB_RADIUS = 32;
const V_STROKE_WIDTH = 3;
const V_SHADOW_OFFSET_Y = 1.6;
const V_SHADOW_OPACITY = 0.35;
const PINK_GLOW = { cx: 14, cy: 16, r: 30, opacity: 0.9 };
const LIME_GLOW = { cx: 52, cy: 54, r: 26, opacity: 0.95 };

const color = (token: string) => `var(--logo-${token})`;

type LogoMarkProps = {
  className?: string;
  /** Accessible name. Omit for a purely decorative mark. */
  title?: string;
};

/** The Vitarchy orb with a bold V. Same artwork as public/favicon.svg. */
export function LogoMark({ className, title }: LogoMarkProps) {
  const uid = useId().replace(/:/g, "");
  const base = `${uid}-base`;
  const pink = `${uid}-pink`;
  const lime = `${uid}-lime`;
  const titleId = `${uid}-title`;

  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 64 64"
      className={cn("size-7 shrink-0", className)}
      role={title ? "img" : undefined}
      aria-hidden={title ? undefined : true}
      aria-labelledby={title ? titleId : undefined}
    >
      {title ? <title id={titleId}>{title}</title> : null}
      <defs>
        <linearGradient id={base} x1="8" y1="6" x2="56" y2="58" gradientUnits="userSpaceOnUse">
          <stop offset="0" style={{ stopColor: color("pink") }} />
          <stop offset="0.5" style={{ stopColor: color("violet") }} />
          <stop offset="1" style={{ stopColor: color("lime") }} />
        </linearGradient>
        <GlowGradient id={pink} token="pink" {...PINK_GLOW} />
        <GlowGradient id={lime} token="lime" {...LIME_GLOW} />
      </defs>
      {[base, pink, lime].map((gradient) => (
        <circle
          key={gradient}
          cx={ORB_CENTER}
          cy={ORB_CENTER}
          r={ORB_RADIUS}
          fill={`url(#${gradient})`}
        />
      ))}
      <path
        d={V_PATH}
        transform={`translate(0 ${String(V_SHADOW_OFFSET_Y)})`}
        strokeWidth={V_STROKE_WIDTH}
        strokeLinejoin="round"
        fillOpacity={V_SHADOW_OPACITY}
        strokeOpacity={V_SHADOW_OPACITY}
        style={{ fill: color("shadow"), stroke: color("shadow") }}
      />
      <path
        d={V_PATH}
        strokeWidth={V_STROKE_WIDTH}
        strokeLinejoin="round"
        style={{ fill: color("letter"), stroke: color("letter") }}
      />
    </svg>
  );
}

interface GlowGradientProps {
  id: string;
  token: string;
  cx: number;
  cy: number;
  r: number;
  opacity: number;
}

/** A radial glow that fades from `token` at `opacity` to transparent. */
function GlowGradient({ id, token, cx, cy, r, opacity }: GlowGradientProps) {
  return (
    <radialGradient id={id} cx={cx} cy={cy} r={r} gradientUnits="userSpaceOnUse">
      <stop offset="0" stopOpacity={opacity} style={{ stopColor: color(token) }} />
      <stop offset="1" stopOpacity="0" style={{ stopColor: color(token) }} />
    </radialGradient>
  );
}
