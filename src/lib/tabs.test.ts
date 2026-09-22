import test from 'node:test';
import assert from 'node:assert/strict';
import { closeTab, shellExited, type TabState } from './tabs.ts';

const three = (active: string): TabState => ({
  tabs: [
    { id: 'a', label: 'SHELL 1' },
    { id: 'b', label: 'SHELL 2' },
    { id: 'c', label: 'SHELL 3' },
  ],
  activeTab: active,
});

test('closing the active tab selects its left neighbour', () => {
  const s = closeTab(three('b'), 'b');
  assert.deepEqual(s.tabs.map((t) => t.id), ['a', 'c']);
  assert.equal(s.activeTab, 'a');
});

test('closing the first tab while active selects the next one', () => {
  assert.equal(closeTab(three('a'), 'a').activeTab, 'b');
});

test('closing a background tab keeps the current selection', () => {
  const s = closeTab(three('c'), 'a');
  assert.deepEqual(s.tabs.map((t) => t.id), ['b', 'c']);
  assert.equal(s.activeTab, 'c');
});

test('the last tab cannot be closed by the user', () => {
  const one: TabState = { tabs: [{ id: 'a', label: 'SHELL 1' }], activeTab: 'a' };
  assert.equal(closeTab(one, 'a'), one);
});

test('a shell exiting closes its tab when others remain', () => {
  const s = shellExited(three('b'), 'b', 'fresh');
  assert.deepEqual(s.tabs.map((t) => t.id), ['a', 'c']);
});

test('the last shell exiting is replaced by a fresh one, keeping its label', () => {
  const one: TabState = { tabs: [{ id: 'a', label: 'SHELL 1' }], activeTab: 'a' };
  const s = shellExited(one, 'a', 'fresh');
  assert.deepEqual(s.tabs, [{ id: 'fresh', label: 'SHELL 1' }]);
  assert.equal(s.activeTab, 'fresh');
});

test('an exit for an unknown or already-closed tab changes nothing', () => {
  const s = three('a');
  assert.equal(shellExited(s, 'gone', 'fresh'), s);
  assert.equal(closeTab(s, 'gone'), s);
});
