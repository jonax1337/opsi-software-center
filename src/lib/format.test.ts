import { describe, expect, it } from "vitest";
import {
  describePasswordExpiry,
  formatBytes,
  formatDateTime,
  formatUptime,
  initials,
  presentOrUndefined,
} from "./format";

const now = new Date("2026-01-10T12:00:00Z");
const hoursAgo = (h: number) => new Date(now.getTime() - h * 3_600_000).toISOString();
const daysFromNow = (d: number) => new Date(now.getTime() + d * 86_400_000).toISOString();

describe("presentOrUndefined", () => {
  it.each([undefined, null, "", "   "])("maps %j to undefined", (s) => {
    expect(presentOrUndefined(s)).toBeUndefined();
  });

  it("trims real values", () => {
    expect(presentOrUndefined("  IT  ")).toBe("IT");
  });
});

describe("formatDateTime", () => {
  it("shows a dash for unparsable input", () => {
    expect(formatDateTime("not a date")).toBe("—");
  });
});

describe("formatUptime", () => {
  it("shows hours below one day", () => {
    expect(formatUptime(hoursAgo(5), now)).toBe("5 h");
  });

  it("shows days and remaining hours from one day on", () => {
    expect(formatUptime(hoursAgo(24 * 3 + 4), now)).toBe("3 d 4 h");
  });

  it("is undefined for a boot time in the future", () => {
    expect(formatUptime(hoursAgo(-1), now)).toBeUndefined();
  });

  it("is undefined for unparsable input", () => {
    expect(formatUptime("garbage", now)).toBeUndefined();
  });
});

describe("describePasswordExpiry", () => {
  it("reports a password that never expires", () => {
    expect(describePasswordExpiry(null, now)).toEqual({ text: "Läuft nicht ab", tone: "muted" });
  });

  it("is null for an unparsable date", () => {
    expect(describePasswordExpiry("garbage", now)).toBeNull();
  });

  it("flags an expired password", () => {
    expect(describePasswordExpiry(daysFromNow(-2), now)?.tone).toBe("bad");
  });

  it("warns on the expiry day", () => {
    expect(describePasswordExpiry(now.toISOString(), now)).toEqual({
      text: "Läuft heute ab",
      tone: "warn",
    });
  });

  it("uses the singular for one day", () => {
    expect(describePasswordExpiry(daysFromNow(1), now)?.text).toMatch(/^In 1 Tag \(/);
  });

  it("warns up to and including 14 days", () => {
    expect(describePasswordExpiry(daysFromNow(14), now)?.tone).toBe("warn");
  });

  it("is ok from 15 days on", () => {
    expect(describePasswordExpiry(daysFromNow(15), now)?.tone).toBe("ok");
  });
});

describe("initials", () => {
  it("uses the first and last word of the display name", () => {
    expect(initials("Erika Anna Mustermann", "emuster")).toBe("EM");
  });

  it("uses one letter for a single-word name", () => {
    expect(initials("erika", "emuster")).toBe("E");
  });

  it("falls back to the account name", () => {
    expect(initials("  ", "emuster")).toBe("E");
  });

  it("is empty when nothing is known", () => {
    expect(initials("", "")).toBe("");
  });
});

describe("formatBytes", () => {
  it.each([
    [0, "0 Bytes"],
    [1023, "1.023 Bytes"],
    [1024, "1 KB"],
    [1536, "1,5 KB"],
    [5 * 1024 ** 2, "5 MB"],
    [1.25 * 1024 ** 3, "1,3 GB"],
    [3 * 1024 ** 5, "3.072 TB"],
  ])("formats %d as %s", (bytes, text) => {
    expect(formatBytes(bytes)).toBe(text);
  });
});
