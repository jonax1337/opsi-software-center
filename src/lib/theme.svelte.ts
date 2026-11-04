import { isTauri } from "./kiosk/bridge";
import { brandPalette, type BrandShades } from "./brandColor";

const media =
  typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: dark)") : null;

/** CSS custom properties the company color overrides (defaults live in app.css). */
const BRAND_PROPERTIES: Record<keyof BrandShades, string[]> = {
  primary: ["--primary", "--ring"],
  primaryForeground: ["--primary-foreground"],
  titlebar: ["--titlebar"],
  titlebarForeground: ["--titlebar-foreground"],
};

function apply(resolved: "light" | "dark") {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-theme", resolved);
}

/** Override the accent tokens with the company color, or drop the overrides so
 *  app.css's Soft Stack blue applies again (`brand` empty). */
function applyBrand(brand: string, resolved: "light" | "dark") {
  const root = document.documentElement.style;
  const shades = brand ? brandPalette(brand)[resolved] : null;
  for (const [key, properties] of Object.entries(BRAND_PROPERTIES)) {
    for (const property of properties) {
      if (shades) root.setProperty(property, shades[key as keyof BrandShades]);
      else root.removeProperty(property);
    }
  }
}

/** Follows the OS for light/dark. Accent colors come from the company color
 *  (opsi property `brand_color`), falling back to the static Soft Stack blue. */
class ThemeStore {
  systemPrefersDark = $state(media?.matches ?? false);
  #brand = "";

  get resolved(): "light" | "dark" {
    return this.systemPrefersDark ? "dark" : "light";
  }

  constructor() {
    // No Mica outside the desktop app — app.css paints a fallback backdrop.
    if (!isTauri()) document.documentElement.toggleAttribute("data-browser-preview", true);
    apply(this.resolved);
    media?.addEventListener("change", (e) => {
      this.systemPrefersDark = e.matches;
      apply(this.resolved);
      applyBrand(this.#brand, this.resolved);
    });
  }

  /** `#rrggbb` from the branding, or "" for the default color. */
  setBrandColor(brand: string) {
    if (brand === this.#brand) return;
    this.#brand = brand;
    applyBrand(brand, this.resolved);
  }
}

export const theme = new ThemeStore();
