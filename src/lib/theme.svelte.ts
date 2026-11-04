import { getAccentColor, isTauri, type AccentColor } from "./kiosk/bridge";

const media =
  typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: dark)") : null;

function apply(resolved: "light" | "dark") {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-theme", resolved);
}

/** Relative luminance (sRGB) of a #rrggbb color → 0..1. */
function luminance(hex: string): number {
  const n = parseInt(hex.slice(1), 16);
  const ch = [(n >> 16) & 0xff, (n >> 8) & 0xff, n & 0xff].map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * ch[0] + 0.7152 * ch[1] + 0.0722 * ch[2];
}

/** Apply the native accent to --primary/--ring for the given theme. Windows
 *  uses Dark1 for accent fills on light backgrounds and Light2 on dark ones —
 *  mirror that so the fill keeps enough contrast against the surface. */
function applyAccent(accent: AccentColor, resolved: "light" | "dark") {
  const root = document.documentElement.style;
  const pick =
    resolved === "dark"
      ? (accent.light2 ?? accent.light1 ?? accent.base)
      : (accent.dark1 ?? accent.base);
  if (!pick) return;
  root.setProperty("--primary", pick);
  root.setProperty("--ring", pick);
  // Text/icon color on top of the accent fill: white unless the accent is light
  // (e.g. yellow/lime), then near-black for legibility.
  root.setProperty("--primary-foreground", luminance(pick) > 0.5 ? "#1a1a1a" : "#ffffff");

  // Title bar: a dark accent shade in both themes, like Company Portal's
  // brand-colored header — the light dark-mode fill would wash out white text.
  const bar = resolved === "dark" ? (accent.dark2 ?? accent.dark1) : (accent.dark1 ?? accent.base);
  if (bar) {
    root.setProperty("--titlebar", bar);
    root.setProperty("--titlebar-foreground", luminance(bar) > 0.5 ? "#1a1a1a" : "#ffffff");
  }
}

/** Follows the OS for light/dark and overrides the accent with the user's
 *  native Windows accent color (falls back to the static CSS blue in the
 *  browser preview, where the registry isn't reachable). */
class ThemeStore {
  systemPrefersDark = $state(media?.matches ?? false);
  #accent: AccentColor | null = null;

  get resolved(): "light" | "dark" {
    return this.systemPrefersDark ? "dark" : "light";
  }

  constructor() {
    // No Mica outside the desktop app — app.css paints a fallback backdrop.
    if (!isTauri()) document.documentElement.toggleAttribute("data-browser-preview", true);
    apply(this.resolved);
    void this.#loadAccent();
    media?.addEventListener("change", (e) => {
      this.systemPrefersDark = e.matches;
      apply(this.resolved);
      if (this.#accent) applyAccent(this.#accent, this.resolved);
    });
  }

  async #loadAccent() {
    if (!isTauri()) return;
    try {
      this.#accent = await getAccentColor();
      applyAccent(this.#accent, this.resolved);
    } catch {
      // Keep the CSS fallback accent.
    }
  }
}

export const theme = new ThemeStore();
