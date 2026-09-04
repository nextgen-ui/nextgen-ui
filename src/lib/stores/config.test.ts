import { test } from 'node:test';
import assert from 'node:assert/strict';
import { appConfig, defaults, loadConfig, saveConfig } from './config.ts';
import { get } from 'svelte/store';

test('appConfig defaults with valid initial structure', () => {
  const cfg = get(appConfig);
  assert.equal(cfg.theme, 'default');
  assert.equal(cfg.fontSize, 14);
  assert.equal(cfg.statsWidth, 230);
  assert.equal(cfg.audioEnabled, true);
  assert.equal(cfg.audioVolume, 0.6);
});

test('loadConfig and saveConfig run safely in headless node environment', async () => {
  const loaded = await loadConfig();
  assert.equal(loaded.theme, defaults.theme);

  const saved = await saveConfig(defaults);
  assert.equal(typeof saved, 'boolean');
});
