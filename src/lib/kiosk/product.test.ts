import { describe, expect, it } from "vitest";
import {
  categoriesOf,
  compareProducts,
  displayName,
  displayVersion,
  featuredProducts,
  isPending,
  joinNames,
  matchesCategory,
  matchesSearch,
  productState,
  publisherOf,
  selectProducts,
  sortProducts,
} from "./product";
import type { Catalog, KioskProduct } from "./types";

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

describe("displayVersion", () => {
  it("shows the available version when not installed", () => {
    expect(displayVersion(product())).toBe("3.0.21-1");
  });

  it("shows the installed version, even when an update is available", () => {
    const p = installed({ installedVerStr: "3.0.20-1", updatePossible: true });
    expect(displayVersion(p)).toBe("3.0.20-1");
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

  it("matches the publisher from the catalog", () => {
    const catalog: Catalog = { vlc: { publisher: "VideoLAN", category: "", featured: false } };
    expect(matchesSearch(product(), "videolan", catalog)).toBe(true);
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

describe("catalog metadata", () => {
  const entry = (publisher: string, category = "", featured = false) => ({
    publisher,
    category,
    featured,
  });
  const vlc = product();
  const gimp = product({ productId: "gimp", productName: "GIMP" });
  const zip = product({ productId: "7zip", productName: "7-Zip" });
  const catalog: Catalog = {
    vlc: entry("VideoLAN", "Multimedia", true),
    gimp: entry("The GIMP Team", "Grafik"),
  };
  const ids = (list: KioskProduct[]) => list.map((p) => p.productId);

  it("has no publisher for products without an entry", () => {
    expect(publisherOf(zip, catalog)).toBe("");
    expect(publisherOf(vlc, catalog)).toBe("VideoLAN");
  });

  it("matches every product for the empty category", () => {
    expect(matchesCategory(zip, "", catalog)).toBe(true);
    expect(matchesCategory(vlc, "Grafik", catalog)).toBe(false);
    expect(matchesCategory(gimp, "Grafik", catalog)).toBe(true);
  });

  it.each([
    ["name-asc", ["7zip", "gimp", "vlc"]],
    ["name-desc", ["vlc", "gimp", "7zip"]],
    ["publisher-asc", ["gimp", "vlc", "7zip"]],
    ["publisher-desc", ["vlc", "gimp", "7zip"]],
  ] as const)("sorts by %s, products without publisher last", (sort, expected) => {
    expect(ids(sortProducts([vlc, zip, gimp], sort, catalog))).toEqual(expected);
  });

  it("does not reorder the input when sorting", () => {
    const input = [vlc, zip];
    sortProducts(input, "name-asc", catalog);
    expect(input).toEqual([vlc, zip]);
  });

  it("groups products into alphabetical categories, skipping uncategorized ones", () => {
    const groups = categoriesOf([vlc, zip, gimp], catalog);
    expect(groups.map((g) => [g.name, ids(g.products)])).toEqual([
      ["Grafik", ["gimp"]],
      ["Multimedia", ["vlc"]],
    ]);
  });

  it("lists only featured products", () => {
    expect(ids(featuredProducts([vlc, zip, gimp], catalog))).toEqual(["vlc"]);
  });
});
