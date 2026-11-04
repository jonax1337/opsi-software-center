import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { atLeast } from "./async";

describe("atLeast", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("holds a fast result back until the minimum duration has passed", async () => {
    let settled = false;
    const done = atLeast(Promise.resolve("ok"), 2000).then((v) => {
      settled = true;
      return v;
    });

    await vi.advanceTimersByTimeAsync(1999);
    expect(settled).toBe(false);

    await vi.advanceTimersByTimeAsync(1);
    await expect(done).resolves.toBe("ok");
  });

  it("does not add time to work that is slower than the minimum", async () => {
    const slow = new Promise<string>((resolve) => setTimeout(() => resolve("slow"), 5000));
    let settled = false;
    const done = atLeast(slow, 2000).then((v) => {
      settled = true;
      return v;
    });

    await vi.advanceTimersByTimeAsync(5000);
    expect(settled).toBe(true);
    await expect(done).resolves.toBe("slow");
  });

  it("rejects with the original error, but only after the minimum duration", async () => {
    const error = new Error("offline");
    const done = atLeast(Promise.reject(error), 2000);
    // Attach the assertion before the timers run, so the rejection is handled
    // the moment it happens (otherwise Vitest reports an unhandled rejection).
    const assertion = expect(done).rejects.toBe(error);
    let rejected = false;
    done.catch(() => (rejected = true));

    await vi.advanceTimersByTimeAsync(1999);
    expect(rejected).toBe(false);

    await vi.advanceTimersByTimeAsync(1);
    await assertion;
    expect(rejected).toBe(true);
  });
});
