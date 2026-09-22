import test from 'node:test';
import assert from 'node:assert/strict';
import { isSafeExternalUrl } from './links.ts';

test('http and https links may be opened', () => {
  assert.equal(isSafeExternalUrl('https://example.com/path?q=1'), true);
  assert.equal(isSafeExternalUrl('http://localhost:8080'), true);
  assert.equal(isSafeExternalUrl('HTTPS://EXAMPLE.COM'), true);
});

test('non-web schemes are refused', () => {
  for (const uri of [
    'file:///etc/passwd',
    'javascript:alert(1)',
    'JaVaScRiPt:alert(1)',
    'data:text/html,<script>alert(1)</script>',
    'vbscript:msgbox',
    'ssh://host',
    'smb://host/share',
    'tauri://localhost',
  ]) {
    assert.equal(isSafeExternalUrl(uri), false, uri);
  }
});

test('malformed and relative input is refused', () => {
  for (const uri of ['', 'not a url', '/relative/path', '//example.com']) {
    assert.equal(isSafeExternalUrl(uri), false, JSON.stringify(uri));
  }
});
