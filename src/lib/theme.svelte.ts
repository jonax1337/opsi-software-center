const media =
  typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: dark)") : null;

function apply(resolved: "light" | "dark") {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-theme", resolved);
}

/** Follows the OS for light/dark. The accent stays the static brand magenta
 *  from app.css so it always matches the app icon. */
class ThemeStore {
  systemPrefersDark = $state(media?.matches ?? false);

  get resolved(): "light" | "dark" {
    return this.systemPrefersDark ? "dark" : "light";
  }

  constructor() {
    apply(this.resolved);
    media?.addEventListener("change", (e) => {
      this.systemPrefersDark = e.matches;
      apply(this.resolved);
    });
  }
}

export const theme = new ThemeStore();
