/**
 * Keyboard shortcuts the HUD claims globally.
 *
 * xterm.js calls stopPropagation() on every key it handles, so a plain
 * window-level keydown listener never sees these while the terminal has focus
 * — which is almost always, since Terminal.svelte focuses on mount. Terminal
 * therefore consults isAppShortcut() from attachCustomKeyEventHandler and
 * declines those keys, letting them bubble up to the window handler instead.
 *
 * Both sides must agree, so this is the single source of truth.
 */
export const APP_SHORTCUT_KEYS = ['t', 'w', 'b', 'p', 'k', 'f'] as const;

export function isAppShortcut(e: KeyboardEvent): boolean {
  if (!e.ctrlKey || e.altKey || e.metaKey) return false;
  return (APP_SHORTCUT_KEYS as readonly string[]).includes(e.key.toLowerCase());
}
