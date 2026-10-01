/*
 * The colour theme.
 *
 * The palette lives in the stylesheet as two sets of the same variables; the
 * choice is made by setting `data-theme` on the document element. Storing it
 * the same way as the language means the two are remembered together and a
 * reader who chooses once does not choose again.
 *
 * The choice is not a preference only: a dark page in a bright room is hard
 * to read, and so is a bright one at night. The reader decides.
 */

import { writable } from 'svelte/store';

export type Theme = 'dark' | 'light';

const STORAGE_KEY = 'apsis.theme';

function isTheme(value: string | null): value is Theme {
  return value === 'dark' || value === 'light';
}

/**
 * The stored choice, the operating system's preference, or the default.
 *
 * The system preference is only consulted when nothing has been stored, so
 * that a reader who has chosen once keeps that choice on a device whose
 * system setting later changes.
 */
function initialTheme(): Theme {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (isTheme(stored)) return stored;
  } catch {
    // A blocked or absent store is not an error; the system preference or
    // the default applies.
  }
  try {
    if (window.matchMedia('(prefers-color-scheme: light)').matches) return 'light';
  } catch {
    // A browser without the query is rare; the default applies.
  }
  return 'dark';
}

/** The current theme. */
export const theme = writable<Theme>(initialTheme());

/** Applies a theme to the store, the storage, and the document. */
export function setTheme(next: Theme): void {
  theme.set(next);
  try {
    window.localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // Persisting is best effort.
  }
  if (typeof document !== 'undefined') {
    document.documentElement.dataset.theme = next;
  }
}

theme.subscribe((current) => {
  // The subscription keeps the document attribute in step with the store
  // even when the store is set directly.
  if (typeof document !== 'undefined') {
    document.documentElement.dataset.theme = current;
  }
});

/**
 * Switches between the two themes and reports the one now in force.
 *
 * The result is returned so that a caller can describe the button in the
 * language of the theme it switches to.
 */
export function toggleTheme(current: Theme): Theme {
  const next: Theme = current === 'dark' ? 'light' : 'dark';
  setTheme(next);
  return next;
}