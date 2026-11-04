/** German display formatting (device, account, title bar). Pure — `now` is a parameter. */

const LOCALE = "de-DE";
const HOUR_MS = 3_600_000;
const DAY_MS = 86_400_000;
/** Password-expiry warning threshold, matching the usual Windows logon prompt. */
const PASSWORD_WARN_DAYS = 14;

export const EM_DASH = "—";

/** Empty or whitespace-only directory strings → undefined, so DataField shows a dash. */
export function presentOrUndefined(s?: string | null): string | undefined {
  const trimmed = s?.trim();
  return trimmed ? trimmed : undefined;
}

/** Avatar initials like Windows: first + last word of the display name
 *  ("Erika Mustermann" → "EM"), else the first letter of the account name. */
export function initials(displayName: string, username: string): string {
  const words = displayName.trim().split(/\s+/).filter(Boolean);
  if (words.length >= 2) return (words[0][0] + words[words.length - 1][0]).toUpperCase();
  return (words[0] ?? username.trim()).slice(0, 1).toUpperCase();
}

export function formatDateTime(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? EM_DASH : d.toLocaleString(LOCALE);
}

export function formatDate(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? EM_DASH : d.toLocaleDateString(LOCALE);
}

export function formatTime(date: Date): string {
  return date.toLocaleTimeString(LOCALE, { hour: "2-digit", minute: "2-digit" });
}

/** "3 d 4 h" / "5 h" since `bootIso`; undefined for unparsable or future values. */
export function formatUptime(bootIso: string, now: Date): string | undefined {
  const ms = now.getTime() - new Date(bootIso).getTime();
  if (!(ms > 0)) return undefined;
  const hours = Math.floor(ms / HOUR_MS);
  const days = Math.floor(hours / 24);
  return days > 0 ? `${days} d ${hours % 24} h` : `${hours} h`;
}

const BYTE_UNITS = ["Bytes", "KB", "MB", "GB", "TB"];

/** "0 Bytes", "512 Bytes", "1,5 MB" — binary steps like Explorer, one decimal from KB on. */
export function formatBytes(bytes: number): string {
  let value = Math.max(0, bytes);
  let unit = 0;
  while (value >= 1024 && unit < BYTE_UNITS.length - 1) {
    value /= 1024;
    unit++;
  }
  const digits = unit === 0 ? 0 : 1;
  return `${value.toLocaleString(LOCALE, { maximumFractionDigits: digits })} ${BYTE_UNITS[unit]}`;
}

export type PasswordTone ="muted" | "ok" | "warn" | "bad";
export type PasswordExpiry = { text: string; tone: PasswordTone };

/** `null` expiry = the password never expires. Returns null for unparsable dates. */
export function describePasswordExpiry(expiresIso: string | null, now: Date): PasswordExpiry | null {
  if (!expiresIso) return { text: "Läuft nicht ab", tone: "muted" };
  const expires = new Date(expiresIso);
  if (Number.isNaN(expires.getTime())) return null;
  const days = Math.ceil((expires.getTime() - now.getTime()) / DAY_MS);
  const date = expires.toLocaleDateString(LOCALE);
  if (days < 0) return { text: `Abgelaufen (${date})`, tone: "bad" };
  if (days === 0) return { text: "Läuft heute ab", tone: "warn" };
  const text = `In ${days} ${days === 1 ? "Tag" : "Tagen"} (${date})`;
  return { text, tone: days <= PASSWORD_WARN_DAYS ? "warn" : "ok" };
}
