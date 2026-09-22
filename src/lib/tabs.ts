/**
 * Terminal tab bookkeeping, kept pure so it can be tested without a DOM.
 * Each tab's id is also its PTY session id, so a fresh id means a fresh shell.
 */

export interface Tab {
  id: string;
  label: string;
}

export interface TabState {
  tabs: Tab[];
  activeTab: string;
}

/** Close a tab the user asked to close. The last tab cannot be closed. */
export function closeTab(state: TabState, id: string): TabState {
  const idx = state.tabs.findIndex((t) => t.id === id);
  if (idx < 0 || state.tabs.length === 1) return state;
  const tabs = state.tabs.filter((t) => t.id !== id);
  // Only move the selection if the closed tab was the one being viewed —
  // closing a background tab used to switch the user away from theirs.
  const activeTab = state.activeTab === id ? tabs[Math.max(0, idx - 1)].id : state.activeTab;
  return { tabs, activeTab };
}

/**
 * A tab's shell exited on its own (`exit`, Ctrl+D). Close the tab; if it was
 * the last one, replace it with a fresh shell rather than leave a dead
 * terminal on screen. `freshId` is only used in that case.
 */
export function shellExited(state: TabState, id: string, freshId: string): TabState {
  const idx = state.tabs.findIndex((t) => t.id === id);
  if (idx < 0) return state;
  if (state.tabs.length > 1) return closeTab(state, id);
  const tabs = [{ id: freshId, label: state.tabs[0].label }];
  return { tabs, activeTab: freshId };
}
