import { test } from 'node:test';
import assert from 'node:assert/strict';
import { networkState, initNetworkService, refreshGeo, setGeoEnabled, isGeoEnabled } from './network.ts';
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

test('geolocation is off by default and makes no request', async () => {
  let calls = 0;
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async () => { calls++; throw new Error('no network in tests'); }) as typeof fetch;
  try {
    assert.equal(isGeoEnabled(), false);
    assert.equal(await refreshGeo(), null);
    assert.equal(calls, 0, 'nothing may be sent to a third party until the user opts in');
    const s = get(networkState);
    assert.equal(s.publicIp, null);
    assert.equal(s.geo, null);
  } finally {
    globalThis.fetch = realFetch;
  }
});

test('geolocation lookup uses HTTPS only', async () => {
  const urls: string[] = [];
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    urls.push(String(input));
    return new Response(JSON.stringify({ success: false }), { status: 200 });
  }) as typeof fetch;
  try {
    setGeoEnabled(true);
    await refreshGeo();
    assert.ok(urls.length > 0);
    for (const u of urls) assert.ok(u.startsWith('https://'), u);
  } finally {
    setGeoEnabled(false);
    globalThis.fetch = realFetch;
  }
});

test('disabling geolocation forgets the fetched location', async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async () =>
    new Response(JSON.stringify({ ip: '203.0.113.7', city: 'X', latitude: 10, longitude: 20 }), { status: 200 })) as typeof fetch;
  try {
    setGeoEnabled(true);
    await refreshGeo();
    assert.equal(get(networkState).publicIp, '203.0.113.7');
    setGeoEnabled(false);
    assert.equal(get(networkState).publicIp, null);
    assert.equal(get(networkState).geo, null);
  } finally {
    globalThis.fetch = realFetch;
  }
});
