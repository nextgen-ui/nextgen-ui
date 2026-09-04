import { test } from 'node:test';
import assert from 'node:assert/strict';
import { networkState, initNetworkService } from './network.ts';
import { get } from 'svelte/store';

test('networkState defaults with valid initial structure', () => {
  const state = get(networkState);
  assert.equal(typeof state.online, 'boolean');
  assert.ok('localIp' in state);
  assert.ok('interfaceName' in state);
  assert.ok('publicIp' in state);
  assert.ok('pingMs' in state);
  assert.ok('geo' in state);
});

test('initNetworkService is safe in headless test environment', () => {
  assert.doesNotThrow(() => initNetworkService());
});
