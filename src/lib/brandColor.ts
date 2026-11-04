/**
 * Derives the app's accent tokens from the company color set by the opsi
 * property `brand_color`. Mirrors how Windows uses its own accent: the color
 * itself on light surfaces, a lighter tint on dark ones (a saturated mid-tone
 * fill looks muddy on #282828), and a dark shade for the title bar in dark mode.
 */

export interface BrandShades {
  /** --primary / --ring: buttons, selection, links */
  primary: string;
  primaryForeground: string;
  /** --titlebar: the Company-Portal-style header */
  titlebar: string;
  titlebarForeground: string;
}

/** How far the dark-mode fill moves toward white, and the dark title bar toward black. */
const DARK_FILL_TINT = 0.4;
const DARK_TITLEBAR_SHADE = 0.25;
const TEXT_ON_LIGHT = "#1a1a1a";
const TEXT_ON_DARK = "#ffffff";

type Rgb = [number, number, number];

function parse(hex: string): Rgb {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 0xff, (n >> 8) & 0xff, n & 0xff];
}

function format(rgb: Rgb): string {
  return `#${rgb.map((c) => Math.round(c).toString(16).padStart(2, "0")).join("")}`;
}

function mix(hex: string, target: Rgb, amount: number): string {
  const rgb = parse(hex);
  return format(rgb.map((c, i) => c + (target[i] - c) * amount) as Rgb);
}

/** Relative luminance (WCAG) of a #rrggbb color → 0..1. */
function luminance(hex: string): number {
  const [r, g, b] = parse(hex).map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

/** White or near-black, whichever reads better on `fill` (yellow → black text). */
export function readableOn(fill: string): string {
  return contrast(fill, TEXT_ON_DARK) >= contrast(fill, TEXT_ON_LIGHT) ? TEXT_ON_DARK : TEXT_ON_LIGHT;
}

function shades(primary: string, titlebar: string): BrandShades {
  return {
    primary,
    primaryForeground: readableOn(primary),
    titlebar,
    titlebarForeground: readableOn(titlebar),
  };
}

/** `brand` must be a `#rrggbb` color (the backend validates the property). */
export function brandPalette(brand: string): Record<"light" | "dark", BrandShades> {
  return {
    light: shades(brand, brand),
    dark: shades(mix(brand, [255, 255, 255], DARK_FILL_TINT), mix(brand, [0, 0, 0], DARK_TITLEBAR_SHADE)),
  };
}
