/**
 * Formats values for display.
 *
 * The table renderer and the scalar renderer share these helpers so that a
 * number is written the same way everywhere.
 */

/** Formats a number with a sensible number of decimals for its magnitude. */
export function formatNumber(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return '—';
  }
  const magnitude = Math.abs(value);
  if (magnitude === 0) return '0';
  if (magnitude >= 1e6 || magnitude < 1e-4) {
    return value.toExponential(6);
  }
  if (magnitude >= 1e3) return value.toFixed(3);
  if (magnitude >= 1) return value.toFixed(6);
  return value.toFixed(9);
}

/** Formats a table cell according to the column type. */
export function formatCell(value: string | number | null, numeric: boolean): string {
  if (value === null || value === undefined) return '—';
  if (typeof value === 'number') {
    return numeric ? formatNumber(value) : String(value);
  }
  return value;
}

/** Returns the CSS colour variable for a graph group. */
export function groupColor(group: string): string {
  return `var(--group-${group}, var(--accent))`;
}

/** Truncates text to a maximum length. */
export function truncate(text: string, length: number): string {
  if (text.length <= length) return text;
  return `${text.slice(0, length - 1)}…`;
}
