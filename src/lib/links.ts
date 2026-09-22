/**
 * Opening links from terminal output.
 *
 * Terminal output is untrusted — any program, remote host or `cat`ed file can
 * print a link, and OSC 8 hyperlinks can carry any URI behind innocent text.
 * xterm's default handlers call `window.open()`, which inside the app webview
 * has no sensible meaning. Links go to the system browser via the opener
 * plugin instead, and only for http(s): `file:`, `javascript:` and custom
 * scheme handlers never leave the terminal.
 */

export function isSafeExternalUrl(uri: string): boolean {
  let url: URL;
  try {
    url = new URL(uri);
  } catch {
    return false;
  }
  return url.protocol === 'http:' || url.protocol === 'https:';
}

export async function openExternal(uri: string): Promise<void> {
  if (!isSafeExternalUrl(uri)) return;
  const { openUrl } = await import('@tauri-apps/plugin-opener');
  await openUrl(uri);
}
