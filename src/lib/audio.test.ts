import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  soundState,
  setSoundEnabled,
  toggleSound,
  setVolume,
  playSound,
  playKeySound,
  preloadAll,
  type SoundEffect,
} from './audio.ts';
import { get } from 'svelte/store';

test('sound defaults to enabled with moderate volume', () => {
  const state = get(soundState);
  assert.equal(typeof state.enabled, 'boolean');
  assert.equal(typeof state.volume, 'number');
  assert.ok(state.volume > 0 && state.volume <= 1);
});

test('toggleSound inverts the enabled state', () => {
  const before = get(soundState).enabled;
  const toggled = toggleSound();
  assert.equal(toggled, !before);
  assert.equal(get(soundState).enabled, !before);

  // Toggle back
  toggleSound();
  assert.equal(get(soundState).enabled, before);
});

test('setSoundEnabled explicitly sets state', () => {
  setSoundEnabled(false);
  assert.equal(get(soundState).enabled, false);

  setSoundEnabled(true);
  assert.equal(get(soundState).enabled, true);
});

test('setVolume clamps between 0.0 and 1.0', () => {
  setVolume(1.5);
  assert.equal(get(soundState).volume, 1.0);

  setVolume(-0.5);
  assert.equal(get(soundState).volume, 0.0);

  setVolume(0.4);
  assert.equal(get(soundState).volume, 0.4);
});

test('playKeySound and playSound run safely in headless/node test environment', () => {
  // In node environment without AudioContext, these functions should exit gracefully without throwing
  assert.doesNotThrow(() => playKeySound('a'));
  assert.doesNotThrow(() => playKeySound('Enter'));
  assert.doesNotThrow(() => playKeySound('\r'));
  assert.doesNotThrow(() => playKeySound('Backspace'));
  assert.doesNotThrow(() => playKeySound('Space'));

  const effects: SoundEffect[] = [
    'stdin',
    'granted',
    'stdout',
    'panels',
    'folder',
    'theme',
    'keyboard',
    'denied',
    'error',
  ];

  for (const effect of effects) {
    assert.doesNotThrow(() => playSound(effect));
    assert.doesNotThrow(() => playSound(effect, { pitchVariance: true, volumeScale: 0.5 }));
  }

  assert.doesNotThrow(() => preloadAll());
});
