import { describe, expect, it } from "vitest";
import { brandPalette, readableOn } from "./brandColor";

describe("brandPalette", () => {
  it("uses the company color itself on light surfaces", () => {
    const { light } = brandPalette("#e30613");
    expect(light.primary).toBe("#e30613");
    expect(light.titlebar).toBe("#e30613");
    expect(light.primaryForeground).toBe("#ffffff");
  });

  it("lightens the fill and darkens the title bar in dark mode", () => {
    const { dark } = brandPalette("#185abd");
    // 40 % toward white / 25 % toward black, per channel
    expect(dark.primary).toBe("#749cd7");
    expect(dark.titlebar).toBe("#12448e");
  });

  it("puts dark text on the light dark-mode fill and white text on the dark title bar", () => {
    const { dark } = brandPalette("#185abd");
    expect(dark.primaryForeground).toBe("#1a1a1a");
    expect(dark.titlebarForeground).toBe("#ffffff");
  });

  it("keeps a light company color legible with dark text", () => {
    const { light } = brandPalette("#ffd200");
    expect(light.primaryForeground).toBe("#1a1a1a");
    expect(light.titlebarForeground).toBe("#1a1a1a");
  });
});

describe("readableOn", () => {
  it("picks white on dark and near-black on light fills", () => {
    expect(readableOn("#000000")).toBe("#ffffff");
    expect(readableOn("#ffffff")).toBe("#1a1a1a");
  });
});
