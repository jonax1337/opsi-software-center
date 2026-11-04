import { describe, expect, it } from "vitest";
import {
  compareProducts,
  displayName,
  isPending,
  joinNames,
  matchesSearch,
  productState,
  selectProducts,
} from "./product";
import type { KioskProduct } from "./types";

function product(overrides: Partial<KioskProduct> = {}): KioskProduct {
  return {
    productId: "vlc",
    productName: "VLC Media Player",
    description: "Medienwiedergabe",
    productVersion: "3.0.21",
    packageVersion: "1",
    versionStr: "3.0.21-1",
    installedVerStr: "",
    installedProdVer: "",
    installedPackVer: "",
    updatePossible: false,
    possibleAction: "setup",
    installationStatus: "not_installed",
    actionRequest: "none",
    actionResult: "",
    ...overrides,
  };
}

const installed = (o: Partial<KioskProduct> = {}) =>
  product({ installationStatus: "installed", possibleAction: "uninstall", ...o });

describe("displayName", () => {
  it("prefers the product name", () => {
    expect(displayName(product())).toBe("VLC Media Player");
  });

  it("falls back to the product id when the name is empty", () => {
    expect(displayName(product({ productName: "" }))).toBe("vlc");
  });
});

describe("isPending", () => {
  it.each(["", "none"] as const)("is false for actionRequest %j", (actionRequest) => {
    expect(isPending(product({ actionRequest }))).toBe(false);
  });

  it.each(["setup", "uninstall", "update", "once"] as const)(
    "is true for actionRequest %j",
    (actionRequest) => {
      expect(isPending(product({ actionRequest }))).toBe(true);
    },
  );
});

describe("productState", () => {
  it("is running while a pending request is being processed", () => {
    expect(productState(product({ actionRequest: "setup" }), true)).toBe("running");
  });

  it("is pending when a request is set but nothing is processing", () => {
    expect(productState(product({ actionRequest: "setup" }), false)).toBe("pending");
  });

  it("is update for an installed product with a newer version", () => {
    expect(productState(installed({ updatePossible: true }), false)).toBe("update");
  });

  it("is installed for an up-to-date installed product", () => {
    expect(productState(installed(), false)).toBe("installed");
  });

  it("is available otherwise", () => {
    expect(productState(product(), false)).toBe("available");
  });

  it("ignores updatePossible on a product that is not installed", () => {
    expect(productState(product({ updatePossible: true }), false)).toBe("available");
  });
});

describe("matchesSearch", () => {
  it("matches every product for a blank query", () => {
    expect(matchesSearch(product(), "   ")).toBe(true);
  });

  it.each(["vlc", "MEDIA", "wiedergabe"])("matches id, name or description (%j)", (q) => {
    expect(matchesSearch(product(), q)).toBe(true);
  });

  it("does not match unrelated text", () => {
    expect(matchesSearch(product(), "firefox")).toBe(false);
  });

  it("tolerates a missing name and description", () => {
    expect(matchesSearch(product({ productName: undefined, description: undefined }), "x")).toBe(
      false,
    );
  });
});

describe("compareProducts", () => {
  it("puts products with an update before the rest", () => {
    const update = installed({ productName: "Zip", updatePossible: true });
    const plain = product({ productName: "Alpha" });
    expect([plain, update].sort(compareProducts)).toEqual([update, plain]);
  });

  it("sorts alphabetically with German collation", () => {
    const names = ["Zoom", "Ärzte-App", "Adobe"].map((productName) => product({ productName }));
    expect(names.sort(compareProducts).map(displayName)).toEqual(["Adobe", "Ärzte-App", "Zoom"]);
  });
});

describe("selectProducts", () => {
  const catalog = [
    product({ productId: "vlc", productName: "VLC" }),
    installed({ productId: "7zip", productName: "7-Zip", updatePossible: true }),
    installed({ productId: "firefox", productName: "Firefox" }),
  ];
  const ids = (list: KioskProduct[]) => list.map((p) => p.productId);

  it("returns all products for the 'all' filter, updates first", () => {
    expect(ids(selectProducts(catalog, "all", ""))).toEqual(["7zip", "firefox", "vlc"]);
  });

  it("returns installed products for the 'installed' filter", () => {
    expect(ids(selectProducts(catalog, "installed", ""))).toEqual(["7zip", "firefox"]);
  });

  it("returns not-installed products for the 'available' filter", () => {
    expect(ids(selectProducts(catalog, "available", ""))).toEqual(["vlc"]);
  });

  it("returns installed products with an update for the 'updates' filter", () => {
    expect(ids(selectProducts(catalog, "updates", ""))).toEqual(["7zip"]);
  });

  it("combines filter and search", () => {
    expect(ids(selectProducts(catalog, "installed", "fire"))).toEqual(["firefox"]);
  });

  it("does not reorder the input array", () => {
    const input = [...catalog];
    selectProducts(input, "all", "");
    expect(input).toEqual(catalog);
  });
});

describe("joinNames", () => {
  it("lists display names comma-separated", () => {
    expect(joinNames([product(), product({ productName: "", productId: "7zip" })])).toBe(
      "VLC Media Player, 7zip",
    );
  });
});
