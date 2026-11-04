/**
 * Settle `work`, but not before `ms` have passed. Used where an operation
 * finishes so fast that the UI would only flicker — a spinner that stays for a
 * moment reads as "something happened". Failures also wait, so success and
 * error feel the same.
 */
export async function atLeast<T>(work: Promise<T>, ms: number): Promise<T> {
  const [result] = await Promise.allSettled([
    work,
    new Promise<void>((resolve) => setTimeout(resolve, ms)),
  ]);
  if (result.status === "rejected") throw result.reason;
  return result.value;
}
